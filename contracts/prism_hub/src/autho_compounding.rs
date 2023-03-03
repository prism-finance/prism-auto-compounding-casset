use std::ops::Mul;

use crate::contract::query_total_issued;
use crate::state::{CONFIG, CURRENT_BATCH, PARAMETERS, STATE};
use basset::hub::{Parameters, State};
use cosmwasm_std::{
    BankMsg, Coin, CosmosMsg, Decimal, DepsMut, Env, MessageInfo, Response, StakingMsg, StdError,
    StdResult, Uint128,
};
use rand::{Rng, SeedableRng, XorShiftRng};

/// Increase exchange rate according to claimed rewards amount
/// Only hub_contract is allowed to execute
pub fn execute_update_exchange_rate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> StdResult<Response> {
    let mut state: State = STATE.load(deps.storage)?;
    let contract_address = env.contract.address;

    let config = CONFIG.load(deps.storage)?;
    let rewards_contract = deps.api.addr_humanize(&config.rewards_contract.unwrap())?;

    // Permission check
    if rewards_contract != info.sender {
        return Err(StdError::generic_err("Unauthorized"));
    }

    let params: Parameters = PARAMETERS.load(deps.storage)?;
    let coin_denom = params.underlying_coin_denom;
    let payment = info
        .funds
        .iter()
        .find(|x| x.denom == coin_denom && x.amount > Uint128::zero())
        .ok_or_else(|| {
            StdError::generic_err(format!(
                "No {} assets are provided to redelegate",
                coin_denom
            ))
        })?;

    // claimed_rewards = current_balance - prev_balance;
    let claimed_rewards = payment.amount;

    let protocol_fee = if params.protocol_fee != Decimal::zero() {
        claimed_rewards.mul(params.protocol_fee)
    } else {
        Uint128::zero()
    };

    let user_rewards = claimed_rewards.checked_sub(protocol_fee as Uint128)?;

    let current_batch = CURRENT_BATCH.load(deps.storage)?;
    let requested_with_fee = current_batch.requested_with_fee;
    let total_issued = query_total_issued(deps.as_ref())?;

    // exchange_rate += user_rewards / total_balance;
    state.exchange_rate += Decimal::from_ratio(user_rewards, total_issued + requested_with_fee);
    state.total_bond_amount += user_rewards;

    STATE.save(deps.storage, &state)?;

    let all_delegations = deps
        .querier
        .query_all_delegations(contract_address)
        .expect("There must be at least one delegation");

    let mut rng = XorShiftRng::seed_from_u64(env.block.height);

    let random_index = rng.gen_range(0, all_delegations.len());

    let mut messages: Vec<CosmosMsg> = vec![];

    if protocol_fee as Uint128 != Uint128::zero() {
        match config.protocol_fee_collector {
            Some(fee_collector) => {
                messages.push(CosmosMsg::Bank(BankMsg::Send {
                    to_address: deps.api.addr_humanize(&fee_collector)?.to_string(),
                    amount: vec![Coin::new(protocol_fee.u128(), &coin_denom)],
                }));
            }
            None => {
                return Err(StdError::generic_err(
                    "protocol fee collector address has not been set",
                ));
            }
        }
    };

    if user_rewards != Uint128::zero() {
        messages.push(
            // send the delegate message
            CosmosMsg::Staking(StakingMsg::Delegate {
                validator: all_delegations
                    .get(random_index)
                    .unwrap()
                    .validator
                    .to_string(),
                amount: Coin::new(user_rewards.u128(), coin_denom),
            }),
        );
    }

    Ok(Response::new()
        .add_messages(messages)
        .add_attribute("action", "update_exchange_rate")
        .add_attribute("reward_collected", claimed_rewards.to_string())
        .add_attribute("protocol_fee", protocol_fee.to_string()))
}
