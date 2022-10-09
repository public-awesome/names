use cosmwasm_std::{Addr, Decimal, Uint128};
use cw_storage_plus::{Index, IndexList, IndexedMap, Item, MultiIndex};
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

pub type TokenId = String;

/// Represents an ask on the marketplace
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Ask {
    pub token_id: TokenId,
    pub seller: Addr,
    pub height: u64,
}

/// Primary key for asks: token_id
pub type AskKey = TokenId;
/// Convenience ask key constructor
pub fn ask_key(token_id: TokenId) -> AskKey {
    token_id
}

/// Defines indices for accessing Asks
pub struct AskIndicies<'a> {
    pub seller: MultiIndex<'a, Addr, Ask, AskKey>,
    pub height: MultiIndex<'a, u64, Ask, AskKey>,
}

impl<'a> IndexList<Ask> for AskIndicies<'a> {
    fn get_indexes(&'_ self) -> Box<dyn Iterator<Item = &'_ dyn Index<Ask>> + '_> {
        let v: Vec<&dyn Index<Ask>> = vec![&self.seller];
        Box::new(v.into_iter())
    }
}

pub fn asks<'a>() -> IndexedMap<'a, AskKey, Ask, AskIndicies<'a>> {
    let indexes = AskIndicies {
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
    pub fn new(token_id: TokenId, bidder: Addr, amount: Uint128, height: u64) -> Self {
        Bid {
            token_id,
            bidder,
            amount,
            height,
        }
    }
}

/// Primary key for bids: (token_id, bidder)
pub type BidKey = (TokenId, Addr);
/// Convenience bid key constructor
pub fn bid_key(token_id: TokenId, bidder: &Addr) -> BidKey {
    (token_id, bidder.clone())
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
