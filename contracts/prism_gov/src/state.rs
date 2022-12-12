use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, IbcEndpoint};
use cw_controllers::Admin;
use cw_storage_plus::{Item, Map};



pub const ADMIN: Admin = Admin::new("admin");

pub const CONFIG: Item<Config> = Item::new("pgov_config");

/// static info on one channel that doesn't change
pub const CHANNEL_INFO: Map<&str, ChannelInfo> = Map::new("channel_info");

#[cw_serde]
pub struct Config {
    pub hub_contract: Addr,
    pub gas_limit: Option<u64>,
}

#[cw_serde]
pub struct ChannelInfo {
    /// id of this channel
    pub id: String,
    /// the remote channel/port we connect to
    pub counterparty_endpoint: IbcEndpoint,
    /// the connection this exists on (you can use to query client/consensus info)
    pub connection_id: String,
}

#[cw_serde]
pub struct ReplyArgs {
    pub channel: String,
}
