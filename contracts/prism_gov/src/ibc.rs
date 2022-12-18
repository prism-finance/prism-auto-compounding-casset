use std::str::FromStr;

use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Binary, Decimal, DepsMut, entry_point, Env, from_binary, IbcBasicResponse, IbcChannel, IbcChannelCloseMsg, IbcChannelConnectMsg, IbcChannelOpenMsg, IbcOrder, IbcPacket, IbcPacketAckMsg, IbcPacketReceiveMsg, IbcPacketTimeoutMsg, IbcReceiveResponse, Reply, Response, SubMsg, SubMsgResult, to_binary, Uint64, WasmMsg};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use basset::gov::{VoteMsg, VoteOption, WeightedVoteOption};
use basset::hub::ExecuteMsg;

use crate::error::{ContractError, Never};
use crate::state::{CHANNEL_INFO, ChannelInfo, CONFIG};

pub const PGOV_VERSION: &str = "pgov-1";
pub const PGOV_ORDERING: IbcOrder = IbcOrder::Unordered;

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, JsonSchema, Debug, Default)]
pub struct PGovPacketData {
    pub proposal_tally_result_packet: ProposalTallyResultPacketData,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, JsonSchema, Debug, Default)]
pub struct ProposalTallyResultPacketData {
    pub proposal_id: Uint64,
    pub asset: String,
    pub tally_result: TallyResult,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, JsonSchema, Debug, Default)]
pub struct TallyResult {
    yes_count: String,
    abstain_count: String,
    no_count: String,
    no_with_veto_count: String,
}

impl TallyResult {
    fn to_weighted_options(self) -> Vec<WeightedVoteOption> {
        let mut vec: Vec<WeightedVoteOption> = vec![];
        let yes = Decimal::from_str(self.yes_count.as_str());
        if yes.is_ok() && !yes.unwrap().is_zero() {
            vec.push(WeightedVoteOption {
                option: VoteOption::Yes as i32,
                weight: self.yes_count.to_string(),
            })
        }
        let abstain = Decimal::from_str(self.abstain_count.as_str());
        if abstain.is_ok() && !abstain.unwrap().is_zero() {
            vec.push(WeightedVoteOption {
                option: VoteOption::Abstain as i32,
                weight: self.abstain_count.to_string(),
            })
        }
        let no = Decimal::from_str(self.no_count.as_str());
        if no.is_ok() && !no.unwrap().is_zero() {
            vec.push(WeightedVoteOption {
                option: VoteOption::No as i32,
                weight: self.no_count.to_string(),
            })
        }
        let no_with_veto = Decimal::from_str(self.no_with_veto_count.as_str());
        if no_with_veto.is_ok() && !no_with_veto.unwrap().is_zero() {
            vec.push(WeightedVoteOption {
                option: VoteOption::NoWithVeto as i32,
                weight: self.no_with_veto_count.to_string(),
            })
        }
        return vec;
    }
}

#[cw_serde]
pub enum ProposalTallyResultPacketAck {
    Result(Binary),
    Error(String),
}

// create a serialized success message
fn ack_success() -> Binary {
    let res = ProposalTallyResultPacketAck::Result(Binary::default());
    to_binary(&res).unwrap()
}

// create a serialized error message
fn ack_fail(err: String) -> Binary {
    let res = ProposalTallyResultPacketAck::Error(err);
    to_binary(&res).unwrap()
}

#[cfg_attr(not(feature = "library"), entry_point)]
/// enforces ordering and versioning constraints
pub fn ibc_channel_open(
    _deps: DepsMut,
    _env: Env,
    msg: IbcChannelOpenMsg,
) -> Result<(), ContractError> {
    enforce_order_and_version(msg.channel(), msg.counterparty_version())?;
    Ok(())
}

#[cfg_attr(not(feature = "library"), entry_point)]
/// record the channel in CHANNEL_INFO
pub fn ibc_channel_connect(
    deps: DepsMut,
    _env: Env,
    msg: IbcChannelConnectMsg,
) -> Result<IbcBasicResponse, ContractError> {
    // we need to check the counter party version in try and ack (sometimes here)
    enforce_order_and_version(msg.channel(), msg.counterparty_version())?;

    let channel: IbcChannel = msg.into();
    let info = ChannelInfo {
        id: channel.endpoint.channel_id,
        counterparty_endpoint: channel.counterparty_endpoint,
        connection_id: channel.connection_id,
    };
    CHANNEL_INFO.save(deps.storage, &info.id, &info)?;

    Ok(IbcBasicResponse::default())
}

fn enforce_order_and_version(
    channel: &IbcChannel,
    counterparty_version: Option<&str>,
) -> Result<(), ContractError> {
    if channel.version != PGOV_VERSION {
        return Err(ContractError::InvalidIbcVersion {
            version: channel.version.clone(),
        });
    }
    if let Some(version) = counterparty_version {
        if version != PGOV_VERSION {
            return Err(ContractError::InvalidIbcVersion {
                version: version.to_string(),
            });
        }
    }
    if channel.order != PGOV_ORDERING {
        return Err(ContractError::OnlyUnorderedChannel {});
    }
    Ok(())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn ibc_channel_close(
    _deps: DepsMut,
    _env: Env,
    _channel: IbcChannelCloseMsg,
) -> Result<IbcBasicResponse, ContractError> {
    unimplemented!();
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn ibc_packet_receive(
    deps: DepsMut,
    _env: Env,
    msg: IbcPacketReceiveMsg,
) -> Result<IbcReceiveResponse, Never> {
    let packet = msg.packet;

    do_ibc_packet_receive(deps, &packet).or_else(|err| {
        Ok(IbcReceiveResponse::new().set_ack(ack_fail(err.to_string()))) // TODO add attributes
    })
}

const VOTE_ID: u64 = 1;

// this does the work of ibc_packet_receive, we wrap it to turn errors into acknowledgements
fn do_ibc_packet_receive(
    deps: DepsMut,
    packet: &IbcPacket,
) -> Result<IbcReceiveResponse, ContractError> {
    let packet_data: PGovPacketData = from_binary(&packet.data)?;

    let proposal = packet_data.proposal_tally_result_packet.proposal_id;
    let tally_result = packet_data.proposal_tally_result_packet.tally_result;
    let vote_msg = ExecuteMsg::Vote(VoteMsg {
        proposal: proposal.u64(),
        options: tally_result.to_weighted_options(),
    });
    let config = CONFIG.load(deps.storage)?;

    let wasm_msg = WasmMsg::Execute {
        contract_addr: config.hub_contract.to_string(),
        msg: to_binary(&vote_msg).unwrap(),
        funds: vec![], // FIXME ??
    };

    let mut sub_msg = SubMsg::reply_on_error(wasm_msg, VOTE_ID);
    let gas_limit = config.gas_limit;
    sub_msg.gas_limit = gas_limit;

    let res = IbcReceiveResponse::new()
        .set_ack(ack_success())
        .add_submessage(sub_msg); // TODO add attributes

    Ok(res)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(_deps: DepsMut, _env: Env, reply: Reply) -> Result<Response, ContractError> {
    match reply.id {
        VOTE_ID => match reply.result {
            SubMsgResult::Ok(_) => Ok(Response::new()),
            SubMsgResult::Err(err) => Ok(Response::new().set_data(ack_fail(err))),
        },
        _ => Err(ContractError::UnknownReplyId { id: reply.id }),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn ibc_packet_ack(
    _deps: DepsMut,
    _env: Env,
    _msg: IbcPacketAckMsg,
) -> Result<IbcBasicResponse, ContractError> {
    Err(ContractError::PacketSendNotSupported {})
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn ibc_packet_timeout(
    _deps: DepsMut,
    _env: Env,
    _msg: IbcPacketTimeoutMsg,
) -> Result<IbcBasicResponse, ContractError> {
    Err(ContractError::PacketSendNotSupported {})
}

#[cfg(test)]
mod test {
    use cosmwasm_std::{IbcEndpoint, Timestamp};
    use cosmwasm_std::testing::mock_env;

    use crate::test_helpers::*;

    use super::*;

    #[test]
    fn test_tally_result() {
        let tally_result = TallyResult {
            yes_count: "0.9".to_string(),
            abstain_count: "".to_string(),
            no_count: "0".to_string(),
            no_with_veto_count: "x".to_string(),
        };
        let weighted_options = tally_result.to_weighted_options();
        assert_eq!(1, weighted_options.len());
        assert_eq!(VoteOption::Yes as i32, weighted_options[0].option);
        assert_eq!("0.9", weighted_options[0].weight);
    }

    #[test]
    fn test_packet_receive() {
        let mut deps = setup(&["channel-0"]);
        let data = PGovPacketData {
            proposal_tally_result_packet: ProposalTallyResultPacketData {
                proposal_id: Uint64::new(1),
                asset: "luna".to_string(),
                tally_result: TallyResult {
                    yes_count: "1".to_string(),
                    abstain_count: "0".to_string(),
                    no_count: "0".to_string(),
                    no_with_veto_count: "0".to_string(),
                },
            },
        };
        let packet = IbcPacket::new(
            to_binary(&data).unwrap(),
            IbcEndpoint {
                port_id: REMOTE_PORT.to_string(),
                channel_id: "channel-0".to_string(),
            },
            IbcEndpoint {
                port_id: CONTRACT_PORT.to_string(),
                channel_id: "channel-0".to_string(),
            },
            3,
            Timestamp::from_seconds(1665321069).into(),
        );
        let msg = IbcPacketReceiveMsg::new(packet);
        let result = ibc_packet_receive(deps.as_mut(), mock_env(), msg).unwrap();
        assert_eq!(1, result.messages.len());
    }
}
