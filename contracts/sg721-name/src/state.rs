use cosmwasm_std::Addr;
use cw_storage_plus::Map;

// name, addr
pub const NAME_MAP: Map<&str, Addr> = Map::new("nm");
