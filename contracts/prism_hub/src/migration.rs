use cosmwasm_std::{CanonicalAddr, Decimal, StdResult, Storage};
use cw_storage_plus::Item;
use basset::hub::{Config, Parameters};
use crate::state::{CONFIG, PARAMETERS};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct LegacyParameters {
    pub epoch_period: u64,
    pub underlying_coin_denom: String,
    pub unbonding_period: u64,
    pub peg_recovery_fee: Decimal,
    pub er_threshold: Decimal,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct LegacyConfig {
    pub creator: CanonicalAddr,
    pub token_contract: Option<CanonicalAddr>,
}


fn read_legacy_config(storage: &dyn Storage) -> StdResult<LegacyConfig> {
    let config: Item<LegacyConfig> = Item::new("\u{0}\u{6}config");
    config.load(storage)
}

fn read_legacy_params(storage: &dyn Storage) -> StdResult<LegacyParameters> {
    let params: Item<LegacyParameters> = Item::new("\u{0}\u{b}parameteres");
    params.load(storage)
}

pub fn migrate_config(
    storage: &mut dyn Storage,
) -> StdResult<()> {
    let legacy_config = read_legacy_config(storage)?;

    CONFIG.save(storage, &Config{
        token_contract: legacy_config.token_contract,
        protocol_fee_collector: None
    })?;

    Ok(())
}

pub fn migrate_params(
    storage: &mut dyn Storage,
) -> StdResult<()> {
    let legacy_params = read_legacy_params(storage)?;

    PARAMETERS.save(storage, &Parameters{
        epoch_period: legacy_params.epoch_period,
        underlying_coin_denom: legacy_params.underlying_coin_denom,
        unbonding_period: legacy_params.unbonding_period,
        peg_recovery_fee: legacy_params.peg_recovery_fee,
        er_threshold: legacy_params.er_threshold,
        protocol_fee: Default::default()
    })?;

    Ok(())
}
