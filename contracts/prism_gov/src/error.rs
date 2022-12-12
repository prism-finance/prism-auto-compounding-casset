


use cosmwasm_std::StdError;

use thiserror::Error;



/// Never is a placeholder to ensure we don't return any errors
#[derive(Error, Debug)]
pub enum Never {}

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Channel doesn't exist: {id}")]
    NoSuchChannel { id: String },

    #[error("Only supports channel with ibc version ics20-1, got {version}")]
    InvalidIbcVersion { version: String },

    #[error("Only supports unordered channel")]
    OnlyUnorderedChannel {},

    #[error("Only supports receiving packet")]
    PacketSendNotSupported {},

    #[error("Cannot migrate from different contract type: {previous_contract}")]
    CannotMigrate { previous_contract: String },

    #[error("Cannot migrate from unsupported version: {previous_version}")]
    CannotMigrateVersion { previous_version: String },

    #[error("Got a submessage reply with unknown id: {id}")]
    UnknownReplyId { id: u64 },

    #[error("You cannot lower the gas limit for a contract on the allow list")]
    CannotLowerGas,

    #[error("Only the governance contract can do this")]
    Unauthorized,

    #[error("You can only send cw20 tokens that have been explicitly allowed by governance")]
    NotOnAllowList,
}
