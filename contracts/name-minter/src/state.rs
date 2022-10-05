use cosmwasm_std::Addr;
use cw_storage_plus::Item;

pub const NAME_COLLECTION: Item<Addr> = Item::new("name-collection");
