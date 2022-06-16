use crate::math::decimal_summation_in_256;

use crate::state::{Parameters, PARAMETERS, STATE};
use basset::hub::State;
use cosmwasm_std::{
    Coin, CosmosMsg, Decimal, DepsMut, Env, MessageInfo, Response, StakingMsg, StdError, StdResult,
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

    let messages: Vec<CosmosMsg> = vec![
        // send the delegate message
        CosmosMsg::Staking(StakingMsg::Delegate {
            validator: all_delegations
                .get(random_index)
                .unwrap()
                .validator
                .to_string(),
            amount: Coin::new(claimed_rewards.u128(), "uluna"),
        }),
    ];

    let res = Response::new()
        .add_messages(messages)
        .add_attribute("action", "update_exchange_rate")
        .add_attribute("reward_collected", claimed_rewards.to_string());

    Ok(res)
}
