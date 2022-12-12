#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    attr, from_binary, to_binary, Addr, Binary, CosmosMsg, Decimal, Deps, DepsMut, DistributionMsg,
    Env, MessageInfo, QueryRequest, Response, StakingMsg, StdError, StdResult, SubMsg, Uint128,
    WasmMsg, WasmQuery,
};

use crate::config::{
    execute_deregister_validator, execute_register_validator, execute_update_config,
    execute_update_params,
};

use crate::state::{
    all_unbond_history, get_unbond_requests, query_get_finished_amount, read_validators, ADMIN,
    CONFIG, CURRENT_BATCH, PARAMETERS, PAUSE, STATE,
};
use crate::unbond::{execute_unbond, execute_withdraw_unbonded};

use crate::autho_compounding::execute_update_exchange_rate;
use crate::bond::execute_bond;
use crate::utility::{is_contract_paused, unwrap_assert_admin, validate_params};
use basset::hub::{
    AllHistoryResponse, Config, ConfigResponse, CurrentBatch, CurrentBatchResponse, Cw20HookMsg,
    ExecuteMsg, InstantiateMsg, Parameters, QueryMsg, State, StateResponse, UnbondRequestsResponse,
    WhitelistedValidatorsResponse, WithdrawableUnbondedResponse,
};
use cw20::{Cw20QueryMsg, Cw20ReceiveMsg, TokenInfoResponse};
use cw_controllers::AdminError;
use basset::gov::MsgVoteWeighted;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    let sender = info.sender.clone();
    let _sndr_raw = deps.api.addr_canonicalize(sender.as_str())?;

    // keep pause false
    PAUSE.save(deps.storage, &false)?;

    let payment = info
        .funds
        .iter()
        .find(|x| x.denom == msg.underlying_coin_denom && x.amount > Uint128::zero())
        .ok_or_else(|| {
            StdError::generic_err(format!("No {} assets are provided to bond", "uluna"))
        })?;

    //set the admin
    let admin = deps.api.addr_validate(info.sender.as_str())?;
    ADMIN.set(deps.branch(), Some(admin))?;

    // store config
    let data = Config {
        token_contract_registered: false,
        token_contract: None,
        protocol_fee_collector: None,
    };
    CONFIG.save(deps.storage, &data)?;

    // store state
    let state = State {
        exchange_rate: Decimal::one(),
        second_exchange_rate: Decimal::one(),
        last_index_modification: env.block.time.seconds(),
        last_unbonded_time: env.block.time.seconds(),
        last_processed_batch: 0u64,
        total_bond_amount: payment.amount,
        ..Default::default()
    };

    STATE.save(deps.storage, &state)?;

    //validate the params
    validate_params(msg.clone())?;

    // instantiate parameters
    let params = Parameters {
        epoch_period: msg.epoch_period,
        underlying_coin_denom: msg.underlying_coin_denom,
        unbonding_period: msg.unbonding_period,
        peg_recovery_fee: msg.peg_recovery_fee,
        er_threshold: msg.er_threshold,
        protocol_fee: msg.protocol_fee,
    };

    PARAMETERS.save(deps.storage, &params)?;

    let batch = CurrentBatch {
        id: 1,
        requested_with_fee: Default::default(),
    };
    CURRENT_BATCH.save(deps.storage, &batch)?;

    let mut messages = vec![];

    // register the given validator
    let register_validator = ExecuteMsg::RegisterValidator {
        validator: msg.validator.clone(),
    };
    messages.push(SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: env.contract.address.to_string(),
        msg: to_binary(&register_validator).unwrap(),
        funds: vec![],
    })));

    // send the delegate message
    messages.push(SubMsg::new(CosmosMsg::Staking(StakingMsg::Delegate {
        validator: msg.validator.to_string(),
        amount: payment.clone(),
    })));

    Ok(Response::new()
        .add_submessages(messages)
        .add_attributes(vec![
            attr("register-validator", msg.validator),
            attr("bond", payment.amount),
        ]))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> StdResult<Response> {
    match msg {
        ExecuteMsg::Pause {} => {
            unwrap_assert_admin(deps.as_ref(), ADMIN, &info.sender)?;

            PAUSE.save(deps.storage, &true)?;
            Ok(Response::new())
        }
        ExecuteMsg::Unpause {} => {
            unwrap_assert_admin(deps.as_ref(), ADMIN, &info.sender)?;

            PAUSE.save(deps.storage, &false)?;
            Ok(Response::new())
        }
        ExecuteMsg::Receive(msg) => {
            is_contract_paused(deps.as_ref())?;
            receive_cw20(deps, env, info, msg)
        }
        ExecuteMsg::Bond { validator } => {
            is_contract_paused(deps.as_ref())?;
            execute_bond(deps, env, info, validator)
        }
        ExecuteMsg::UpdateGlobalIndex {} => {
            is_contract_paused(deps.as_ref())?;
            execute_update_global(deps, env)
        }
        ExecuteMsg::UpdateExchangeRate {} => {
            is_contract_paused(deps.as_ref())?;
            execute_update_exchange_rate(deps, env, info)
        }
        ExecuteMsg::WithdrawUnbonded {} => {
            is_contract_paused(deps.as_ref())?;
            execute_withdraw_unbonded(deps, env, info)
        }
        ExecuteMsg::RegisterValidator { validator } => {
            is_contract_paused(deps.as_ref())?;
            execute_register_validator(deps, env, info, validator)
        }
        ExecuteMsg::DeregisterValidator { validator } => {
            is_contract_paused(deps.as_ref())?;
            execute_deregister_validator(deps, env, info, validator)
        }
        ExecuteMsg::CheckSlashing {} => {
            is_contract_paused(deps.as_ref())?;
            execute_slashing(deps, env)
        }
        ExecuteMsg::UpdateParams {
            epoch_period,
            unbonding_period,
            peg_recovery_fee,
            er_threshold,
            protocol_fee,
        } => {
            is_contract_paused(deps.as_ref())?;
            execute_update_params(
                deps,
                env,
                info,
                epoch_period,
                unbonding_period,
                peg_recovery_fee,
                er_threshold,
                protocol_fee,
            )
        }
        ExecuteMsg::UpdateConfig {
            token_contract,
            protocol_fee_collector,
        } => {
            is_contract_paused(deps.as_ref())?;
            execute_update_config(deps, env, info, token_contract, protocol_fee_collector)
        }
        ExecuteMsg::UpdateAdmin { admin } => {
            is_contract_paused(deps.as_ref())?;
            let admin = deps.api.addr_validate(&admin)?;
            match ADMIN.execute_update_admin(deps, info, Some(admin)) {
                Ok(r) => Ok(r),
                Err(e) => match e {
                    AdminError::NotAdmin {} => Err(StdError::generic_err("Caller is not admin")),
                    AdminError::Std(std_error) => Err(std_error),
                },
            }
        }
        // TODO vote should be permissioned: only prism_gov contract can execute vote
        ExecuteMsg::Vote(vote_msg) => {
            let stargate_msg = CosmosMsg::Stargate {
                type_url: "/cosmos.gov.v1.MsgVoteWeighted".to_string(),
                value: MsgVoteWeighted {
                    proposal_id: vote_msg.proposal,
                    voter: env.contract.address.to_string(),
                    options: vote_msg.options,
                }.into(),
            };
            Ok(
                Response::new().add_submessage(SubMsg::new(stargate_msg)) // TODO add attributes
            )
        }
    }
}

/// CW20 token receive handler.
pub fn receive_cw20(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    cw20_msg: Cw20ReceiveMsg,
) -> StdResult<Response> {
    let contract_addr = info.sender.clone();

    match from_binary(&cw20_msg.msg) {
        Ok(Cw20HookMsg::Unbond {}) => {
            // only token contract can execute this message
            let conf = CONFIG.load(deps.storage)?;
            if deps.api.addr_canonicalize(contract_addr.as_str())?
                != conf
                    .token_contract
                    .expect("the token contract must have been registered")
            {
                return Err(StdError::generic_err("unauthorized"));
            }
            execute_unbond(deps, env, info, cw20_msg.amount, cw20_msg.sender)
        }
        Err(err) => Err(err),
    }
}

/// Update general parameters
/// Permissionless
pub fn execute_update_global(deps: DepsMut, env: Env) -> StdResult<Response> {
    let mut messages: Vec<SubMsg> = vec![];

    let contract_addr = env.contract.address.clone();

    let param = PARAMETERS.load(deps.storage)?;

    // Send withdraw message
    let mut withdraw_msgs = withdraw_all_rewards(&deps, contract_addr.clone())?;
    messages.append(&mut withdraw_msgs);

    let balances = deps.querier.query_all_balances(contract_addr.to_string())?;
    let principle_balances_before_update = balances
        .iter()
        .find(|x| x.denom == param.underlying_coin_denom)
        .unwrap()
        .amount;

    messages.push(SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: contract_addr.to_string(),
        msg: to_binary(&ExecuteMsg::UpdateExchangeRate {}).unwrap(),
        funds: vec![],
    })));

    //update state last modified
    STATE.update(deps.storage, |mut last_state| -> StdResult<State> {
        last_state.last_index_modification = env.block.time.seconds();
        last_state.principle_balance_before_exchange_update = principle_balances_before_update;
        Ok(last_state)
    })?;

    Ok(Response::new()
        .add_submessages(messages)
        .add_attributes(vec![attr("action", "update_global_index")]))
}

/// Create withdraw requests for all validators
fn withdraw_all_rewards(deps: &DepsMut, delegator: Addr) -> StdResult<Vec<SubMsg>> {
    let mut messages: Vec<SubMsg> = vec![];
    let delegations = deps.querier.query_all_delegations(delegator);

    if let Ok(delegations) = delegations {
        for delegation in delegations {
            let msg: CosmosMsg =
                CosmosMsg::Distribution(DistributionMsg::WithdrawDelegatorReward {
                    validator: delegation.validator,
                });
            messages.push(SubMsg::new(msg));
        }
    }

    Ok(messages)
}

/// Check whether slashing has happened
/// This is used for checking slashing while bonding or unbonding
pub fn slashing(deps: &mut DepsMut, env: Env) -> StdResult<()> {
    //read params
    let params = PARAMETERS.load(deps.storage)?;
    let coin_denom = params.underlying_coin_denom;

    // Check the amount that contract thinks is bonded
    let state_total_bonded = STATE.load(deps.storage)?.total_bond_amount;

    // Check the actual bonded amount
    let delegations = deps.querier.query_all_delegations(env.contract.address)?;
    if delegations.is_empty() {
        Ok(())
    } else {
        let mut actual_total_bonded = Uint128::zero();
        for delegation in delegations {
            if delegation.amount.denom == coin_denom {
                actual_total_bonded += delegation.amount.amount
            }
        }

        // Need total issued for updating the exchange rate
        let total_issued = query_total_issued(deps.as_ref())?;
        let current_requested_fee = CURRENT_BATCH.load(deps.storage)?.requested_with_fee;

        // Slashing happens if the expected amount is less than stored amount
        if state_total_bonded.u128() > actual_total_bonded.u128() {
            STATE.update(deps.storage, |mut state| -> StdResult<State> {
                state.total_bond_amount = actual_total_bonded;
                state.update_exchange_rate(total_issued, current_requested_fee);
                Ok(state)
            })?;
        }

        Ok(())
    }
}

/// Handler for tracking slashing
pub fn execute_slashing(mut deps: DepsMut, env: Env) -> StdResult<Response> {
    // call slashing
    slashing(&mut deps, env)?;
    // read state for log
    let state = STATE.load(deps.storage)?;
    Ok(Response::new().add_attributes(vec![
        attr("action", "check_slashing"),
        attr("new_exchange_rate", state.exchange_rate.to_string()),
    ]))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query_config(deps)?),
        QueryMsg::State {} => to_binary(&query_state(deps)?),
        QueryMsg::CurrentBatch {} => to_binary(&query_current_batch(deps)?),
        QueryMsg::WhitelistedValidators {} => to_binary(&query_white_validators(deps)?),
        QueryMsg::WithdrawableUnbonded { address } => {
            to_binary(&query_withdrawable_unbonded(deps, address, env)?)
        }
        QueryMsg::Parameters {} => to_binary(&query_params(deps)?),
        QueryMsg::UnbondRequests { address } => to_binary(&query_unbond_requests(deps, address)?),
        QueryMsg::AllHistory { start_from, limit } => {
            to_binary(&query_unbond_requests_limitation(deps, start_from, limit)?)
        }
        QueryMsg::Admin {} => to_binary(&ADMIN.query_admin(deps)?),
    }
}

fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config = CONFIG.load(deps.storage)?;

    let token: Option<String> = if config.token_contract.is_some() {
        Some(
            deps.api
                .addr_humanize(&config.token_contract.unwrap())
                .unwrap()
                .to_string(),
        )
    } else {
        None
    };

    let fee_collector: Option<String> = if config.protocol_fee_collector.is_some() {
        Some(
            deps.api
                .addr_humanize(&config.protocol_fee_collector.unwrap())
                .unwrap()
                .to_string(),
        )
    } else {
        None
    };

    Ok(ConfigResponse {
        token_contract: token,
        protocol_fee_collector: fee_collector,
    })
}

fn query_state(deps: Deps) -> StdResult<StateResponse> {
    let state = STATE.load(deps.storage)?;

    let res = StateResponse {
        exchange_rate: state.exchange_rate,
        total_bond_amount: state.total_bond_amount,
        last_index_modification: state.last_index_modification,
        principle_balance_before_exchange_update: state.principle_balance_before_exchange_update,
        prev_hub_balance: state.prev_hub_balance,
        actual_unbonded_amount: state.actual_unbonded_amount,
        last_unbonded_time: state.last_unbonded_time,
        last_processed_batch: state.last_processed_batch,
    };
    Ok(res)
}

fn query_white_validators(deps: Deps) -> StdResult<WhitelistedValidatorsResponse> {
    let validators = read_validators(deps.storage)?;
    let response = WhitelistedValidatorsResponse { validators };
    Ok(response)
}

fn query_current_batch(deps: Deps) -> StdResult<CurrentBatchResponse> {
    let current_batch = CURRENT_BATCH.load(deps.storage)?;
    Ok(CurrentBatchResponse {
        id: current_batch.id,
        requested_with_fee: current_batch.requested_with_fee,
    })
}

fn query_withdrawable_unbonded(
    deps: Deps,
    address: String,
    env: Env,
) -> StdResult<WithdrawableUnbondedResponse> {
    let params = PARAMETERS.load(deps.storage)?;
    let historical_time = env.block.time.seconds() - params.unbonding_period;
    let all_requests = query_get_finished_amount(deps.storage, address, historical_time)?;

    let withdrawable = WithdrawableUnbondedResponse {
        withdrawable: all_requests,
    };
    Ok(withdrawable)
}

fn query_params(deps: Deps) -> StdResult<Parameters> {
    PARAMETERS.load(deps.storage)
}

pub(crate) fn query_total_issued(deps: Deps) -> StdResult<Uint128> {
    let token_address = deps
        .api
        .addr_humanize(
            &CONFIG
                .load(deps.storage)?
                .token_contract
                .expect("token contract must have been registered"),
        )?
        .to_string();
    let token_info: TokenInfoResponse =
        deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: token_address,
            msg: to_binary(&Cw20QueryMsg::TokenInfo {})?,
        }))?;

    Ok(token_info.total_supply)
}

fn query_unbond_requests(deps: Deps, address: String) -> StdResult<UnbondRequestsResponse> {
    if deps.api.addr_validate(address.as_str()).is_err() {
        return Err(StdError::generic_err("invalid address"));
    }
    let requests = get_unbond_requests(deps.storage, address.clone())?;
    let res = UnbondRequestsResponse { address, requests };
    Ok(res)
}

fn query_unbond_requests_limitation(
    deps: Deps,
    start: Option<u64>,
    limit: Option<u32>,
) -> StdResult<AllHistoryResponse> {
    let requests = all_unbond_history(deps.storage, start, limit)?;
    let res = AllHistoryResponse { history: requests };
    Ok(res)
}
