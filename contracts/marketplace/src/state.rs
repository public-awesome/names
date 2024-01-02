use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Decimal, StdResult, Storage, Timestamp, Uint128};
use cw_storage_macro::index_list;
use cw_storage_plus::{IndexedMap, Item, Map, MultiIndex, UniqueIndex};
use sg_controllers::Hooks;

#[cw_serde]
pub struct SudoParams {
    /// Fair Burn + Community Pool fee for winning bids
    pub trading_fee_percent: Decimal,
    /// Min value for a bid
    pub min_price: Uint128,
    /// Interval to rate limit setting asks (in seconds)
    pub ask_interval: u64,
    /// The maximum number of renewals that can be processed in each block
    pub max_renewals_per_block: u32,
    /// The number of bids to query to when searching for the highest bid
    pub valid_bid_query_limit: u32,
    /// The number of seconds before the current block time that a
    /// bid must have been created in order to be considered valid
    pub renew_window: u64,
    /// The percentage of the winning bid that must be paid to renew a name
    pub renewal_bid_percentage: Decimal,
    /// The address with permission to invoke process_renewals
    pub operator: Addr,
}

pub const SUDO_PARAMS: Item<SudoParams> = Item::new("sudo-params");

pub const ASK_HOOKS: Hooks = Hooks::new("ask-hooks");
pub const BID_HOOKS: Hooks = Hooks::new("bid-hooks");
pub const SALE_HOOKS: Hooks = Hooks::new("sale-hooks");

pub const NAME_MINTER: Item<Addr> = Item::new("name-minter");
pub const NAME_COLLECTION: Item<Addr> = Item::new("name-collection");

/// (renewal_time (in seconds), id) -> [token_id]
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
    pub renewal_time: Timestamp,
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
    /// Index by renewal time
    pub renewal_time: MultiIndex<'a, u64, Ask, AskKey>,
}

pub fn asks<'a>() -> IndexedMap<'a, AskKey, Ask, AskIndicies<'a>> {
    let indexes = AskIndicies {
        id: UniqueIndex::new(|d| d.id, "ask__id"),
        seller: MultiIndex::new(
            |_pk: &[u8], d: &Ask| d.seller.clone(),
            "asks",
            "asks__seller",
        ),
        renewal_time: MultiIndex::new(
            |_pk: &[u8], d: &Ask| d.renewal_time.seconds(),
            "asks",
            "asks__renewal_time",
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
    pub created_time: Timestamp,
}

impl Bid {
    pub fn new(token_id: &str, bidder: Addr, amount: Uint128, created_time: Timestamp) -> Self {
        Bid {
            token_id: token_id.to_string(),
            bidder,
            amount,
            created_time,
        }
    }
}

/// Primary key for bids: (token_id, bidder)
pub type BidKey = (TokenId, Addr);

pub fn bid_key(token_id: &str, bidder: &Addr) -> BidKey {
    (token_id.to_string(), bidder.clone())
}

/// Defines indices for accessing bids
#[index_list(Bid)]
pub struct BidIndicies<'a> {
    pub bidder: MultiIndex<'a, Addr, Bid, BidKey>,
    pub price: MultiIndex<'a, (String, u128), Bid, BidKey>,
    pub created_time: MultiIndex<'a, (String, u64), Bid, BidKey>,
}

pub fn bids<'a>() -> IndexedMap<'a, BidKey, Bid, BidIndicies<'a>> {
    let indexes = BidIndicies {
        bidder: MultiIndex::new(|_pk: &[u8], b: &Bid| b.bidder.clone(), "b2", "b2__b"),
        price: MultiIndex::new(
            |_pk: &[u8], b: &Bid| (b.token_id.clone(), b.amount.u128()),
            "b2",
            "b2__p",
        ),
        created_time: MultiIndex::new(
            |_pk: &[u8], b: &Bid| (b.token_id.clone(), b.created_time.seconds()),
            "b2",
            "b2__ct",
        ),
    };
    IndexedMap::new("b2", indexes)
}

/// Defines indices for accessing bids
#[index_list(Bid)]
pub struct LegacyBidIndices<'a> {
    pub price: MultiIndex<'a, u128, Bid, BidKey>,
    pub bidder: MultiIndex<'a, Addr, Bid, BidKey>,
    pub created_time: MultiIndex<'a, u64, Bid, BidKey>,
}

pub fn legacy_bids<'a>() -> IndexedMap<'a, BidKey, Bid, LegacyBidIndices<'a>> {
    let indexes = LegacyBidIndices {
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
        created_time: MultiIndex::new(
            |_pk: &[u8], d: &Bid| d.created_time.seconds(),
            "bids",
            "bids__time",
        ),
    };
    IndexedMap::new("bids", indexes)
}
