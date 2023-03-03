use cw_controllers::Admin;
use cw_storage_plus::Item;

use basset::rewards::Config;

pub type LastBatch = u64;

pub const ADMIN: Admin = Admin::new("admin");
pub static PAUSE: Item<bool> = Item::new("pause");

pub const CONFIG: Item<Config> = Item::new("\u{0}\u{6}config");
