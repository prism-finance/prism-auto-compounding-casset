#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    attr, to_binary, Binary, Coin, CosmosMsg, Deps, DepsMut, Env, MessageInfo, Response, StdError,
    StdResult, SubMsg, WasmMsg,
};

use crate::state::{ADMIN, CONFIG, PAUSE};
use crate::utility::{is_contract_paused, unwrap_assert_admin};
use basset::hub::ExecuteMsg::UpdateExchangeRate;
use basset::rewards::{Config, ConfigResponse, ExecuteMsg, InstantiateMsg, QueryMsg};
use cw_controllers::AdminError;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    mut deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    let sender = info.sender.clone();
    let _sndr_raw = deps.api.addr_canonicalize(sender.as_str())?;

    // keep pause false
    PAUSE.save(deps.storage, &false)?;

    //set the admin
    let admin = deps.api.addr_validate(info.sender.as_str())?;
    ADMIN.set(deps.branch(), Some(admin))?;

    // store config
    let conf = Config {
        hub_contract: deps.api.addr_canonicalize(&msg.hub_addr)?,
        underlying_coin_denom: msg.underlying_coin_denom,
    };
    CONFIG.save(deps.storage, &conf)?;

    Ok(Response::new().add_attributes(vec![
        attr("hub_contract", msg.hub_addr),
        attr("admin", info.sender.to_string()),
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
        ExecuteMsg::ProcessRewards {} => {
            is_contract_paused(deps.as_ref())?;
            execute_process_rewards(deps, env, info)
        }
    }
}

pub fn execute_process_rewards(deps: DepsMut, env: Env, info: MessageInfo) -> StdResult<Response> {
    let config = CONFIG.load(deps.storage)?;
    let hub_contract = deps.api.addr_humanize(&config.hub_contract)?;

    if info.sender != hub_contract {
        return Err(StdError::generic_err("Caller is not hub contract"));
    }

    let contract_address = env.contract.address;
    let balance: Coin = deps
        .querier
        .query_balance(contract_address, &config.underlying_coin_denom)?;

    let messages: Vec<SubMsg> = vec![SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: hub_contract.to_string(),
        msg: to_binary(&UpdateExchangeRate {}).unwrap(),
        funds: vec![Coin::new(balance.amount.u128(), balance.denom)],
    }))];

    Ok(Response::new()
        .add_submessages(messages)
        .add_attributes(vec![attr("reward_accumulated", balance.amount)]))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query_config(deps)?),
        QueryMsg::Admin {} => to_binary(&ADMIN.query_admin(deps)?),
    }
}

fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config = CONFIG.load(deps.storage)?;

    let hub_addr: String = deps
        .api
        .addr_humanize(&config.hub_contract)
        .unwrap()
        .to_string();

    Ok(ConfigResponse {
        hub_contract: hub_addr,
    })
}
