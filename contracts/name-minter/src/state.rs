use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::Item;

pub const NAME_COLLECTION: Item<Addr> = Item::new("name-collection");

pub const NAME_MARKETPLACE: Item<Addr> = Item::new("name-marketplace");

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

#[cw_serde]
pub struct ParamsResponse {
    pub min_name_length: u64,
    pub max_name_length: u64,
    pub base_price: Uint128,
}
