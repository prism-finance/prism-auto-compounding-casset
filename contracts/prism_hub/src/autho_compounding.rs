use crate::math::decimal_summation_in_256;

use crate::state::{PARAMETERS, STATE, Parameters};
use basset::hub::State;
use cosmwasm_std::{DepsMut, Env, MessageInfo, Response, StdResult, Decimal, CosmosMsg, StakingMsg, StdError, Coin, Uint128};
use rand::{XorShiftRng, SeedableRng, Rng};

/// Swap all native tokens to reward_denom
/// Only hub_contract is allowed to execute
#[allow(clippy::if_same_then_else)]
#[allow(clippy::needless_collect)]
pub fn execute_swap(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> StdResult<Response> {
    // let config = read_config(deps.storage)?;
    // let param = PARAMETERS.load(deps.storage)?;
    //
    // let contr_addr = env.contract.address;
    //
    // if info.sender != contr_addr {
    //     return Err(StdError::generic_err("unauthorized"));
    // }
    //
    // let balance = deps.querier.query_all_balances(contr_addr)?;
    // let mut messages: Vec<CosmosMsg> = Vec::new();
    //
    // let principle_denom = param.underlying_coin_denom;
    //
    // let denoms: Vec<String> = balance.iter().map(|item| item.denom.clone()).collect();
    //
    // let exchange_rates = query_exchange_rates(&deps, principle_denom.clone(), denoms)?;
    // let known_denoms: Vec<String> = exchange_rates
    //     .exchange_rates
    //     .iter()
    //     .map(|item| item.quote_denom.clone())
    //     .collect();
    //
    // for coin in balance {
    //     if coin.denom == principle_denom.clone() || !known_denoms.contains(&coin.denom) {
    //         continue;
    //     }
    //
    //     messages.push(create_swap_msg(coin, principle_denom.to_string()));
    // }

    let res = Response::new()
        .add_attributes(vec![("action", "swap")]);

    Ok(res)
}

/// Increase exchange rate according to claimed rewards amount
/// Only hub_contract is allowed to execute
pub fn execute_update_exchange_rate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> StdResult<Response> {
    let mut state: State = STATE.load(deps.storage)?;
    let mut contract_address = env.contract.address;

    // Permission check
    if contract_address != info.sender {
        return Err(StdError::generic_err("Unauthorized"));
    }

    let params: Parameters = PARAMETERS.load(deps.storage)?;
    let new_balance: Coin = deps
        .querier
        .query_balance(contract_address.clone(), params.underlying_coin_denom)?;

    let previous_balance = state.principle_balance_before_exchange_update;

    // claimed_rewards = current_balance - prev_balance;
    let claimed_rewards = new_balance.amount.checked_sub(previous_balance)?;


    // exchange_rate += claimed_rewards / total_balance;
    state.second_exchange_rate = decimal_summation_in_256(
        state.second_exchange_rate,
        Decimal::from_ratio(claimed_rewards, state.total_bond_amount),
    );

    STATE.save(deps.storage, &state)?;

    let all_delegations = deps
        .querier
        .query_all_delegations(contract_address)
        .expect("There must be at least one delegation");

    let mut rng = XorShiftRng::seed_from_u64(env.block.height);
    let random_index = rng.gen_range(0, all_delegations.len());

    let mut messages: Vec<CosmosMsg> = vec![
        // send the delegate message
        CosmosMsg::Staking(StakingMsg::Delegate {
            validator: all_delegations.get(random_index).unwrap().validator.to_string(),
            amount: Coin::new(claimed_rewards.u128(), "uluna"),
        }),
    ];

    let attributes = vec![
        ("action", "update_exchange_rate"),
        ("reward_collected", claimed_rewards.to_string().as_str()),
    ];
    let res = Response::new().add_attributes(attributes);

    Ok(res)
}

// pub fn query_exchange_rates(
//     deps: &DepsMut,
//     base_denom: String,
//     quote_denoms: Vec<String>,
// ) -> StdResult<ExchangeRatesResponse> {
//     let querier = TerraQuerier::new(&deps.querier);
//     let res: ExchangeRatesResponse = querier.query_exchange_rates(base_denom, quote_denoms)?;
//     Ok(res)
// }
