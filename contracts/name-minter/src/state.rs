use cosmwasm_std::Addr;
use cw_controllers::Admin;
use cw_storage_plus::Item;

use serde::{Deserialize, Serialize};
use sg_name_minter::{Config, SudoParams};

use whitelist_updatable_flatrate::helpers::WhitelistUpdatableFlatrateContract;

#[derive(Serialize, Deserialize)]
pub struct WhitelistContract {
    pub contract_type: WhitelistContractType,
    pub addr: Addr,
}

#[derive(Serialize, Deserialize, PartialEq)]
pub enum WhitelistContractType {
    UpdatableFlatrateDiscount,
    UpdatablePercentDiscount,
}

pub const SUDO_PARAMS: Item<SudoParams> = Item::new("params");

pub const NAME_COLLECTION: Item<Addr> = Item::new("name-collection");

pub const NAME_MARKETPLACE: Item<Addr> = Item::new("name-marketplace");

pub const ADMIN: Admin = Admin::new("admin");

/// Can only be updated by admin
pub const WHITELISTS: Item<Vec<WhitelistContract>> = Item::new("whitelists");

/// Controls if minting is paused or not by admin
pub const PAUSED: Item<bool> = Item::new("paused");

pub const CONFIG: Item<Config> = Item::new("config");
