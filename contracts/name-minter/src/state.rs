use cosmwasm_std::Addr;
use cw_storage_plus::Item;

pub const COLLECTION_ADDRESS: Item<Addr> = Item::new("collection");
