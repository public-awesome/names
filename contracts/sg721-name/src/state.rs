use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};

pub const NAME_MARKETPLACE: Item<Addr> = Item::new("name-marketplace");

/// Address (bech32) -> name
/// `token_uri` -> `token_id`
pub const ADDRESS_MAP: Map<&Addr, String> = Map::new("am");
pub const NAME_MAP: Map<&Addr, String> = Map::new("am");
