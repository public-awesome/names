use cosmwasm_std::{Addr, Decimal, StdResult, Storage, Uint128};
use cw_storage_plus::{Index, IndexList, IndexedMap, Item, Map, MultiIndex, UniqueIndex};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use sg_controllers::Hooks;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct SudoParams {
    /// Fair Burn fee for winning bids
    pub trading_fee_percent: Decimal,
    /// Min value for a bid
    pub min_price: Uint128,
}

pub const SUDO_PARAMS: Item<SudoParams> = Item::new("sudo-params");

pub const ASK_HOOKS: Hooks = Hooks::new("ask-hooks");
pub const BID_HOOKS: Hooks = Hooks::new("bid-hooks");
pub const SALE_HOOKS: Hooks = Hooks::new("sale-hooks");

pub const NAME_MINTER: Item<Addr> = Item::new("name-minter");
pub const NAME_COLLECTION: Item<Addr> = Item::new("name-collection");

/// height -> [name1, name2, etc.]
pub const RENEWAL_QUEUE: Map<u64, Vec<TokenId>> = Map::new("rq");

pub const ASK_COUNT: Item<u64> = Item::new("ask-count");

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
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
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
pub struct AskIndicies<'a> {
    /// Unique incrementing id for each ask
    /// This allows pagination when `token_id`s are strings
    pub id: UniqueIndex<'a, u64, Ask, AskKey>,
    /// Index by seller
    pub seller: MultiIndex<'a, Addr, Ask, AskKey>,
    /// Keeps track of whene renewal has to happen
    pub height: MultiIndex<'a, u64, Ask, AskKey>,
}

impl<'a> IndexList<Ask> for AskIndicies<'a> {
    fn get_indexes(&'_ self) -> Box<dyn Iterator<Item = &'_ dyn Index<Ask>> + '_> {
        let v: Vec<&dyn Index<Ask>> = vec![&self.id, &self.seller, &self.height];
        Box::new(v.into_iter())
    }
}

pub fn asks<'a>() -> IndexedMap<'a, AskKey, Ask, AskIndicies<'a>> {
    let indexes = AskIndicies {
        id: UniqueIndex::new(|d| d.id, "ask__id"),
        seller: MultiIndex::new(|d: &Ask| d.seller.clone(), "asks", "asks__seller"),
        height: MultiIndex::new(|d: &Ask| d.height, "asks", "asks__height"),
    };
    IndexedMap::new("asks", indexes)
}

/// Represents a bid (offer) on the marketplace
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
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
pub struct BidIndicies<'a> {
    pub token_id: MultiIndex<'a, TokenId, Bid, BidKey>,
    pub price: MultiIndex<'a, u128, Bid, BidKey>,
    pub bidder: MultiIndex<'a, Addr, Bid, BidKey>,
    pub height: MultiIndex<'a, u64, Bid, BidKey>,
}

impl<'a> IndexList<Bid> for BidIndicies<'a> {
    fn get_indexes(&'_ self) -> Box<dyn Iterator<Item = &'_ dyn Index<Bid>> + '_> {
        let v: Vec<&dyn Index<Bid>> = vec![&self.token_id, &self.price, &self.bidder];
        Box::new(v.into_iter())
    }
}

pub fn bids<'a>() -> IndexedMap<'a, BidKey, Bid, BidIndicies<'a>> {
    let indexes = BidIndicies {
        token_id: MultiIndex::new(
            |d: &Bid| d.token_id.clone(),
            "bids",
            "bids__collection_token_id",
        ),
        price: MultiIndex::new(|d: &Bid| d.amount.u128(), "bids", "bids__collection_price"),
        bidder: MultiIndex::new(|d: &Bid| d.bidder.clone(), "bids", "bids__bidder"),
        height: MultiIndex::new(|d: &Bid| d.height, "bids", "bids__height"),
    };
    IndexedMap::new("bids", indexes)
}
