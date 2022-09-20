use cosmwasm_std::{Addr, CustomQuery, Deps, StdError};
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
