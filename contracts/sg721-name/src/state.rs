use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};

pub const NAME_MARKETPLACE: Item<Addr> = Item::new("name-marketplace");

pub type TokenUri = Addr;
pub type TokenId = String;

/// Address (bech32) -> name
pub const REVERSE_MAP: Map<&TokenUri, TokenId> = Map::new("rm");
