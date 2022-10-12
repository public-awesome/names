use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Decimal, StdResult, Storage, Uint128};
use cw_storage_macro::index_list;
use cw_storage_plus::{IndexedMap, Item, Map, MultiIndex, UniqueIndex};
use sg_controllers::Hooks;

#[cw_serde]
pub struct SudoParams {
    /// Fair Burn fee for winning bids
    pub trading_fee_percent: Decimal,
    /// Min value for a bid
    pub min_price: Uint128,
    /// Blocks per year
    pub blocks_per_year: u64,
}

pub const SUDO_PARAMS: Item<SudoParams> = Item::new("sudo-params");

pub const ASK_HOOKS: Hooks = Hooks::new("ask-hooks");
pub const BID_HOOKS: Hooks = Hooks::new("bid-hooks");
pub const SALE_HOOKS: Hooks = Hooks::new("sale-hooks");

pub const NAME_MINTER: Item<Addr> = Item::new("name-minter");
pub const NAME_COLLECTION: Item<Addr> = Item::new("name-collection");

/// (renewal_time, id) -> [token_id]
pub const RENEWAL_QUEUE: Map<(u64, u64), TokenId> = Map::new("rq");

pub const ASK_COUNT: Item<u64> = Item::new("ask-count");

pub const IS_SETUP: Item<bool> = Item::new("is-setup");

pub fn ask_count(storage: &dyn Storage) -> StdResult<u64> {
    Ok(ASK_COUNT.may_load(storage)?.unwrap_or_default())
}

pub fn increment_asks(storage: &mut dyn Storage) -> StdResult<u64> {
    let val = ask_count(storage)? + 1;
    ASK_COUNT.save(storage, &val)?;
    Ok(val)
}

pub fn decrement_asks(storage: &mut dyn Storage) -> StdResult<u64> {
    let val = ask_count(storage)? - 1;
    ASK_COUNT.save(storage, &val)?;
    Ok(val)
}

/// Type for storing the `ask`
pub type TokenId = String;

/// Type for `ask` unique secondary index
pub type Id = u64;

/// Represents an ask on the marketplace
#[cw_serde]
pub struct Ask {
    pub token_id: TokenId,
    pub id: u64,
    pub seller: Addr,
    pub height: u64,
    pub renewal_fund: Uint128,
}

/// Primary key for asks: token_id
/// Name reverse lookup can happen in O(1) time
pub type AskKey = TokenId;
/// Convenience ask key constructor
pub fn ask_key(token_id: &str) -> AskKey {
    token_id.to_string()
}

/// Defines indices for accessing Asks
#[index_list(Ask)]
pub struct AskIndicies<'a> {
    /// Unique incrementing id for each ask
    /// This allows pagination when `token_id`s are strings
    pub id: UniqueIndex<'a, u64, Ask, AskKey>,
    /// Index by seller
    pub seller: MultiIndex<'a, Addr, Ask, AskKey>,
}

pub fn asks<'a>() -> IndexedMap<'a, AskKey, Ask, AskIndicies<'a>> {
    let indexes = AskIndicies {
        id: UniqueIndex::new(|d| d.id, "ask__id"),
        seller: MultiIndex::new(
            |_pk: &[u8], d: &Ask| d.seller.clone(),
            "asks",
            "asks__seller",
        ),
    };
    IndexedMap::new("asks", indexes)
}

/// Represents a bid (offer) on the marketplace
#[cw_serde]
pub struct Bid {
    pub token_id: TokenId,
    pub bidder: Addr,
    pub amount: Uint128,
    pub height: u64,
}

impl Bid {
    pub fn new(token_id: &str, bidder: Addr, amount: Uint128, height: u64) -> Self {
        Bid {
            token_id: token_id.to_string(),
            bidder,
            amount,
            height,
        }
    }
}

/// Primary key for bids: (token_id, bidder)
pub type BidKey = (TokenId, Addr);
/// Convenience bid key constructor
pub fn bid_key(token_id: &str, bidder: &Addr) -> BidKey {
    (token_id.to_string(), bidder.clone())
}

/// Defines incides for accessing bids
#[index_list(Bid)]
pub struct BidIndicies<'a> {
    pub price: MultiIndex<'a, u128, Bid, BidKey>,
    pub bidder: MultiIndex<'a, Addr, Bid, BidKey>,
}

pub fn bids<'a>() -> IndexedMap<'a, BidKey, Bid, BidIndicies<'a>> {
    let indexes = BidIndicies {
        price: MultiIndex::new(
            |_pk: &[u8], d: &Bid| d.amount.u128(),
            "bids",
            "bids__collection_price",
        ),
        bidder: MultiIndex::new(
            |_pk: &[u8], d: &Bid| d.bidder.clone(),
            "bids",
            "bids__bidder",
        ),
    };
    IndexedMap::new("bids", indexes)
}
