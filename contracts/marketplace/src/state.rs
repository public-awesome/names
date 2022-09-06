use cosmwasm_std::{Addr, BlockInfo, Decimal, Timestamp, Uint128};
use cw_storage_plus::{Index, IndexList, IndexedMap, Item, MultiIndex};
use cw_utils::Duration;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use sg_controllers::Hooks;

use crate::helpers::ExpiryRange;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct SudoParams {
    /// Fair Burn fee for winning bids
    pub trading_fee_percent: Decimal,
    /// Valid time range for Bids
    /// (min, max) in seconds
    pub bid_expiry: ExpiryRange,
    /// Operators are entites that are responsible for maintaining the active state of Asks
    /// They listen to NFT transfer events, and update the active state of Asks
    pub operators: Vec<Addr>,
    /// Min value for a bid
    pub min_price: Uint128,
    /// Duration after expiry when a bid becomes stale
    pub stale_bid_duration: Duration,
    /// Stale bid removal reward
    pub bid_removal_reward_percent: Decimal,
    /// Listing fee to reduce spam
    pub listing_fee: Uint128,
    /// Name collection
    pub collection: Addr,
}

pub const SUDO_PARAMS: Item<SudoParams> = Item::new("sudo-params");

pub const ASK_HOOKS: Hooks = Hooks::new("ask-hooks");
pub const BID_HOOKS: Hooks = Hooks::new("bid-hooks");
pub const SALE_HOOKS: Hooks = Hooks::new("sale-hooks");

pub type TokenId = u32;

pub trait Order {
    fn expires_at(&self) -> Timestamp;

    fn is_expired(&self, block: &BlockInfo) -> bool {
        self.expires_at() <= block.time
    }
}

/// Represents an ask on the marketplace
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Ask {
    pub token_id: TokenId,
    pub seller: Addr,
    pub funds_recipient: Option<Addr>,
    pub is_active: bool,
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
    };
    IndexedMap::new("asks", indexes)
}

/// Represents a bid (offer) on the marketplace
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Bid {
    pub token_id: TokenId,
    pub bidder: Addr,
    pub price: Uint128,
    pub expires_at: Timestamp,
}

impl Bid {
    pub fn new(token_id: TokenId, bidder: Addr, price: Uint128, expires: Timestamp) -> Self {
        Bid {
            token_id,
            bidder,
            price,
            expires_at: expires,
        }
    }
}

impl Order for Bid {
    fn expires_at(&self) -> Timestamp {
        self.expires_at
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
    // Cannot include `Timestamp` in index, converted `Timestamp` to `seconds` and stored as `u64`
    pub bidder_expires_at: MultiIndex<'a, (Addr, u64), Bid, BidKey>,
}

impl<'a> IndexList<Bid> for BidIndicies<'a> {
    fn get_indexes(&'_ self) -> Box<dyn Iterator<Item = &'_ dyn Index<Bid>> + '_> {
        let v: Vec<&dyn Index<Bid>> = vec![
            &self.token_id,
            &self.price,
            &self.bidder,
            &self.bidder_expires_at,
        ];
        Box::new(v.into_iter())
    }
}

pub fn bids<'a>() -> IndexedMap<'a, BidKey, Bid, BidIndicies<'a>> {
    let indexes = BidIndicies {
        token_id: MultiIndex::new(|d: &Bid| d.token_id, "bids", "bids__collection_token_id"),
        price: MultiIndex::new(|d: &Bid| d.price.u128(), "bids", "bids__collection_price"),
        bidder: MultiIndex::new(|d: &Bid| d.bidder.clone(), "bids", "bids__bidder"),
        bidder_expires_at: MultiIndex::new(
            |d: &Bid| (d.bidder.clone(), d.expires_at.seconds()),
            "bids",
            "bids__bidder_expires_at",
        ),
    };
    IndexedMap::new("bids", indexes)
}
