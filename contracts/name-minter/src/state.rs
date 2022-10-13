use cosmwasm_schema::cw_serde;
use cosmwasm_std::Addr;
use cw_controllers::Admin;
use cw_storage_plus::Item;

#[cw_serde]
pub struct SudoParams {
    /// 3
    pub min_name_length: u64,
    /// 63
    pub max_name_length: u64,
    /// 100_000_000
    pub base_price: u128,
}

pub const SUDO_PARAMS: Item<SudoParams> = Item::new("params");

pub const NAME_COLLECTION: Item<Addr> = Item::new("name-collection");

pub const NAME_MARKETPLACE: Item<Addr> = Item::new("name-marketplace");

pub const ADMIN: Admin = Admin::new("admin");

/// The currently active whitelist
/// Can only be updated by admin
pub const WHITELIST: Item<Option<Addr>> = Item::new("whitelist");
