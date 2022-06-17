use cosmwasm_std::{CanonicalAddr, Decimal, Uint128};
use cw20::Cw20ReceiveMsg;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub type UnbondRequest = Vec<(u64, Uint128)>;

#[derive(Serialize, Deserialize, Clone, Debug,PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub epoch_period: u64,
    pub underlying_coin_denom: String,
    pub unbonding_period: u64,
    pub peg_recovery_fee: Decimal,
    pub er_threshold: Decimal,
    pub reward_denom: String,
    pub validator: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq,JsonSchema)]
pub struct Parameters {
    pub epoch_period: u64,
    pub underlying_coin_denom: String,
    pub unbonding_period: u64,
    pub peg_recovery_fee: Decimal,
    pub er_threshold: Decimal,
    pub reward_denom: String,
}

#[derive(Serialize, Deserialize, Clone, Debug,PartialEq, JsonSchema)]
pub struct CurrentBatch {
    pub id: u64,
    pub requested_with_fee: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema, Default)]
pub struct State {
    pub exchange_rate: Decimal,
    pub second_exchange_rate: Decimal,
    pub total_bond_amount: Uint128,
    pub last_index_modification: u64,
    pub prev_hub_balance: Uint128,
    pub actual_unbonded_amount: Uint128,
    pub principle_balance_before_exchange_update: Uint128,
    pub last_unbonded_time: u64,
    pub last_processed_batch: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema)]
pub struct Config {
    pub creator: CanonicalAddr,
    pub token_contract: Option<CanonicalAddr>,
}

impl State {
    pub fn update_exchange_rate(&mut self, total_issued: Uint128, requested_with_fee: Uint128) {
        let actual_supply = total_issued + requested_with_fee;
        if self.total_bond_amount.is_zero() || actual_supply.is_zero() {
            self.exchange_rate = Decimal::one()
        } else {
            self.exchange_rate = Decimal::from_ratio(self.total_bond_amount, actual_supply);
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    ////////////////////
    /// Owner's operations
    ////////////////////

    /// Set the owener
    UpdateConfig {
        owner: Option<String>,
        token_contract: Option<String>,
    },

    // Update the exchange rate
    UpdateExchangeRate {},

    /// Register receives the reward contract address
    RegisterValidator {
        validator: String,
    },

    // Remove the validator from validators whitelist
    DeregisterValidator {
        validator: String,
    },

    /// update the parameters that is needed for the contract
    UpdateParams {
        epoch_period: Option<u64>,
        unbonding_period: Option<u64>,
        peg_recovery_fee: Option<Decimal>,
        er_threshold: Option<Decimal>,
    },

    ////////////////////
    /// User's operations
    ////////////////////

    /// Receives `amount` in underlying coin denom from sender.
    /// Delegate `amount` to a specific `validator`.
    /// Issue `amount` / exchange_rate for the user.
    Bond {
        validator: String,
    },

    /// Update global index
    UpdateGlobalIndex {},

    /// Send back unbonded coin to the user
    WithdrawUnbonded {},

    /// Check whether the slashing has happened or not
    CheckSlashing {},

    ////////////////////
    /// bAsset's operations
    ///////////////////

    /// Receive interface for send token.
    /// Unbond the underlying coin denom.
    /// Burn the received basset token.
    Receive(Cw20ReceiveMsg),
}

#[derive(Serialize, Deserialize, Clone, Debug,PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Config {},
    State {},
    WhitelistedValidators {},
    CurrentBatch {},
    WithdrawableUnbonded {
        address: String,
    },
    Parameters {},
    UnbondRequests {
        address: String,
    },
    AllHistory {
        start_from: Option<u64>,
        limit: Option<u32>,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq,JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Cw20HookMsg {
    Unbond {},
}

#[derive(Serialize, Deserialize, Clone, Debug,PartialEq, JsonSchema)]
pub struct UnbondHistory {
    pub batch_id: u64,
    pub time: u64,
    pub amount: Uint128,
    pub applied_exchange_rate: Decimal,
    pub withdraw_rate: Decimal,
    pub released: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct StateResponse {
    pub exchange_rate: Decimal,
    pub total_bond_amount: Uint128,
    pub last_index_modification: u64,
    pub prev_hub_balance: Uint128,
    pub actual_unbonded_amount: Uint128,
    pub last_unbonded_time: u64,
    pub last_processed_batch: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ConfigResponse {
    pub owner: String,
    pub token_contract: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq,JsonSchema)]
pub struct WhitelistedValidatorsResponse {
    pub validators: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug,PartialEq, JsonSchema)]
pub struct CurrentBatchResponse {
    pub id: u64,
    pub requested_with_fee: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug,PartialEq, JsonSchema)]
pub struct WithdrawableUnbondedResponse {
    pub withdrawable: Uint128,
}
#[derive(Serialize, Deserialize, Clone, Debug,PartialEq, JsonSchema)]
pub struct UnbondRequestsResponse {
    pub address: String,
    pub requests: UnbondRequest,
}

#[derive(Serialize, Deserialize, Clone, Debug,PartialEq,JsonSchema)]
pub struct AllHistoryResponse {
    pub history: Vec<UnbondHistory>,
}
