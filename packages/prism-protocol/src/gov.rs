use cosmwasm_schema::cw_serde;
use cosmwasm_std::Binary;
use prost::{Enumeration, Message};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// WeightedVoteOption defines a unit of vote for vote split.
#[derive(Clone, PartialEq, Eq, Message, Serialize, Deserialize, JsonSchema)]
pub struct WeightedVoteOption {
    #[prost(enumeration = "VoteOption", tag = "1")]
    pub option: i32,
    #[prost(string, tag = "2")]
    pub weight: ::prost::alloc::string::String,
}

/// VoteOption enumerates the valid vote options for a given governance proposal.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Enumeration)]
#[repr(i32)]
pub enum VoteOption {
    /// VOTE_OPTION_UNSPECIFIED defines a no-op vote option.
    Unspecified = 0,
    /// VOTE_OPTION_YES defines a yes vote option.
    Yes = 1,
    /// VOTE_OPTION_ABSTAIN defines an abstain vote option.
    Abstain = 2,
    /// VOTE_OPTION_NO defines a no vote option.
    No = 3,
    /// VOTE_OPTION_NO_WITH_VETO defines a no with veto vote option.
    NoWithVeto = 4,
}

/// MsgVoteWeighted defines a message to cast a vote.
#[derive(Clone, PartialEq, Eq, Message, Serialize, Deserialize, JsonSchema)]
pub struct MsgVoteWeighted {
    #[prost(uint64, tag = "1")]
    // #[serde(
    //     serialize_with = "crate::serde::as_str::serialize",
    //     deserialize_with = "crate::serde::as_str::deserialize"
    // )]
    pub proposal_id: u64,
    #[prost(string, tag = "2")]
    pub voter: ::prost::alloc::string::String,
    #[prost(message, repeated, tag = "3")]
    pub options: ::prost::alloc::vec::Vec<WeightedVoteOption>,
}

#[cw_serde]
pub struct VoteMsg {
    pub proposal: u64,
    pub options: Vec<WeightedVoteOption>,
}

impl From<MsgVoteWeighted> for Binary {
    fn from(msg: MsgVoteWeighted) -> Self {
        let mut bytes = Vec::new();
        Message::encode(&msg, &mut bytes)
            .expect("Message encoding must be infallible");
        Binary(bytes)
    }
}
