use crate::state::CONFIG;
use basset::hub::Config;
use cosmwasm_std::{CanonicalAddr, StdResult, Storage};
use cw_storage_plus::Item;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct LegacyConfig {
    pub token_contract: Option<CanonicalAddr>,
    pub protocol_fee_collector: Option<CanonicalAddr>,
}

fn read_legacy_config(storage: &dyn Storage) -> StdResult<LegacyConfig> {
    let config: Item<LegacyConfig> = Item::new("\u{0}\u{6}config");
    config.load(storage)
}

pub fn migrate_config(
    storage: &mut dyn Storage,
    rewards_contract: Option<CanonicalAddr>,
) -> StdResult<()> {
    let legacy_config = read_legacy_config(storage)?;

    CONFIG.save(
        storage,
        &Config {
            token_contract: legacy_config.token_contract,
            protocol_fee_collector: legacy_config.protocol_fee_collector,
            rewards_contract,
        },
    )?;

    Ok(())
}
