use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Decimal};
use cw_controllers::Admin;
use cw_storage_plus::Item;
use whitelist_updatable::helpers::WhitelistUpdatableContract;

#[cw_serde]
pub struct SudoParams {
    /// 3 (same as DNS)
    pub min_name_length: u32,
    /// 63 (same as DNS)
    pub max_name_length: u32,
    /// 100_000_000 (5+ ASCII char price)
    pub base_price: u128,
    /// Fair Burn fee (rest goes to Community Pool)
    pub fair_burn_percent: Decimal,
}

pub const SUDO_PARAMS: Item<SudoParams> = Item::new("params");

pub const NAME_COLLECTION: Item<Addr> = Item::new("name-collection");

pub const NAME_MARKETPLACE: Item<Addr> = Item::new("name-marketplace");

pub const ADMIN: Admin = Admin::new("admin");

/// Can only be updated by admin
pub const WHITELISTS: Item<Vec<WhitelistUpdatableContract>> = Item::new("whitelists");

/// Controls if minting is paused or not by admin
pub const PAUSED: Item<bool> = Item::new("paused");
