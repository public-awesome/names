use cosmwasm_schema::cw_serde;
use cosmwasm_std::Addr;
use cw_controllers::Admin;
use cw_storage_plus::{Item, Map};

#[cw_serde]
pub struct SudoParams {
    pub max_record_count: u32,
}
pub const SUDO_PARAMS: Item<SudoParams> = Item::new("params");
pub const NAME_MARKETPLACE: Item<Addr> = Item::new("name-marketplace");

pub type TokenUri = Addr;
pub type TokenId = String;

/// Address (bech32) -> name
pub const REVERSE_MAP: Map<&TokenUri, TokenId> = Map::new("rm");

/// Address of the text record verification oracle
pub const VERIFIER: Admin = Admin::new("verifier");
