use crate::state::PAUSE;
use cosmwasm_std::{Addr, CustomQuery, Deps, Response, StdError, StdResult};
use cw_controllers::{Admin, AdminError};

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

pub fn is_contract_paused<Q: CustomQuery>(deps: Deps<Q>) -> StdResult<Response> {
    let is_paused = PAUSE.load(deps.storage)?;

    if is_paused {
        return Err(StdError::generic_err(
            "Contract is paused cannot perform the tx",
        ));
    }

    Ok(Response::new())
}
