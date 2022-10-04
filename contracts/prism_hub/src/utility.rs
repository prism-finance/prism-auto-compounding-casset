use crate::state::PAUSE;
use basset::hub::InstantiateMsg;
use cosmwasm_std::{Addr, CustomQuery, Decimal, Deps, Response, StdError, StdResult};
use cw_controllers::{Admin, AdminError};

const MAINNET_UNDELEGATION_TIME: u64 = 1814400;
const COIN_DENOM: &str = "uluna";

pub fn unwrap_assert_admin<Q: CustomQuery>(
    deps: Deps<Q>,
    admin: Admin,
    sender: &Addr,
) -> Result<(), StdError> {
    match admin.assert_admin(deps, sender) {
        Ok(_) => Ok(()),
        Err(e) => match e {
            AdminError::NotAdmin {} => Err(StdError::generic_err("Caller is not admin")),
            AdminError::Std(std_error) => Err(std_error),
        },
    }
}

pub fn validate_params(msg: InstantiateMsg) -> Result<(), StdError> {
    if msg.epoch_period > MAINNET_UNDELEGATION_TIME {
        StdError::generic_err("epoch period cannot be more than mainnet undelegation period");
    }

    if msg.er_threshold < Decimal::one() {
        StdError::generic_err("exchange rate threshold should be more than one");
    }

    if msg.protocol_fee > Decimal::one() {
        StdError::generic_err("Protocol fee should not be more than 1");
    }

    if msg.unbonding_period > MAINNET_UNDELEGATION_TIME {
        StdError::generic_err("unbonding period cannot be more than mainnet undelegation period");
    }

    if msg.underlying_coin_denom != COIN_DENOM {
        StdError::generic_err("underlying coin denom should be uluna");
    }
    Ok(())
}

pub fn is_contract_paused<Q: CustomQuery>(deps: Deps<Q>) -> StdResult<Response> {
    let is_paused = PAUSE.load(deps.storage)?;

    if is_paused {
        return Err(StdError::generic_err(
            "Contract is paused cannot perform the tx",
        ));
    }

    Ok(Response::new())
}
