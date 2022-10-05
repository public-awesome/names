use cosmwasm_std::Addr;
use cw_storage_plus::Item;

pub const NAME_COLLECTION: Item<Addr> = Item::new("name-collection");

pub const NAME_MARKETPLACE: Item<Addr> = Item::new("name-marketplace");
