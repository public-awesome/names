use cosmwasm_schema::cw_serde;
use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};

#[cw_serde]
pub struct Config {
    pub admin: Addr,
    pub per_address_limit: u32,
    pub minter_contract: Option<Addr>,
}

pub const CONFIG: Item<Config> = Item::new("config");
pub const TOTAL_ADDRESS_COUNT: Item<u64> = Item::new("total_address_count");
// Holds all addresses and mint count
pub const WHITELIST: Map<Addr, u32> = Map::new("wl");
