use crate::state::{Ask, Bid, Id, SudoParams, TokenId};
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{to_binary, Addr, Binary, StdResult, Uint128};

#[cw_serde]
pub struct InstantiateMsg {
    /// Community pool fee for winning bids
    /// 0.25% = 25, 0.5% = 50, 1% = 100, 2.5% = 250
    pub trading_fee_bps: u64,
    /// Min value for bids and asks
    pub min_price: Uint128,
    /// Blocks per year
    pub blocks_per_year: u64,
}

#[cw_serde]
pub enum ExecuteMsg {
    /// List name NFT on the marketplace by creating a new ask
    /// Only the name minter can call this.
    SetAsk { token_id: TokenId, seller: String },
    /// Place a bid on an existing ask
    SetBid { token_id: TokenId },
    /// Remove an existing bid from an ask
    RemoveBid { token_id: TokenId },
    /// Accept a bid on an existing ask
    AcceptBid { token_id: TokenId, bidder: String },
    /// Fund renewal of a name
    FundRenewal { token_id: TokenId },
    /// Refund a renewal of a name
    RefundRenewal { token_id: TokenId },
    /// Check if expired names have been paid for, and collect fees.
    /// If not paid, transfer ownership to the highest bidder.
    ProcessRenewals { height: u64 },
    /// Setup contract with minter and collection addresses
    /// Can only be run once
    Setup { minter: String, collection: String },
}

#[cw_serde]
pub enum SudoMsg {
    /// Update the contract parameters
    /// Can only be called by governance
    UpdateParams {
        trading_fee_bps: Option<u64>,
        min_price: Option<Uint128>,
        blocks_per_year: Option<u64>,
    },
    /// Update the contract address of the name minter
    UpdateNameMinter { minter: String },
    /// Update the contract address of the name collection
    UpdateNameCollection { collection: String },
    /// Add a new hook to be informed of all asks
    AddAskHook { hook: String },
    /// Remove a ask hook
    RemoveAskHook { hook: String },
    /// Add a new hook to be informed of all bids
    AddBidHook { hook: String },
    /// Remove a bid hook
    RemoveBidHook { hook: String },
    /// Add a new hook to be informed of all trades
    AddSaleHook { hook: String },
    /// Remove a trade hook
    RemoveSaleHook { hook: String },
}

pub type Collection = String;
pub type Bidder = String;
pub type Seller = String;

/// Offset for ask pagination
#[cw_serde]
pub struct AskOffset {
    pub price: Uint128,
    pub token_id: TokenId,
}

impl AskOffset {
    pub fn new(price: Uint128, token_id: TokenId) -> Self {
        AskOffset { price, token_id }
    }
}

/// Offset for bid pagination
#[cw_serde]
pub struct BidOffset {
    pub price: Uint128,
    pub token_id: TokenId,
    pub bidder: Addr,
}

impl BidOffset {
    pub fn new(price: Uint128, token_id: TokenId, bidder: Addr) -> Self {
        BidOffset {
            price,
            token_id,
            bidder,
        }
    }
}

#[cw_serde]
pub enum QueryMsg {
    /// Get the current ask for specific name
    /// Return type: `CurrentAskResponse`
    Ask { token_id: TokenId },
    /// Get all asks for a collection
    /// Return type: `AsksResponse`
    Asks {
        start_after: Option<Id>,
        limit: Option<u32>,
    },
    /// Get all asks in reverse
    /// Return type: `AsksResponse`
    ReverseAsks {
        start_before: Option<Id>,
        limit: Option<u32>,
    },
    /// Count of all asks
    /// Return type: `AskCountResponse`
    AskCount {},
    /// Get all asks by seller
    /// Return type: `AsksResponse`
    AsksBySeller {
        seller: Seller,
        start_after: Option<TokenId>,
        limit: Option<u32>,
    },
    /// Get data for a specific bid
    /// Return type: `BidResponse`
    Bid { token_id: TokenId, bidder: Bidder },
    /// Get all bids by a bidder
    /// Return type: `BidsResponse`
    BidsByBidder {
        bidder: Bidder,
        start_after: Option<TokenId>,
        limit: Option<u32>,
    },
    /// Get all bids for a specific NFT
    /// Return type: `BidsResponse`
    Bids {
        token_id: TokenId,
        start_after: Option<Bidder>,
        limit: Option<u32>,
    },
    /// Get all bids for a collection, sorted by price
    /// Return type: `BidsResponse`
    BidsSortedByPrice {
        start_after: Option<BidOffset>,
        limit: Option<u32>,
    },
    /// Get all bids for a collection, sorted by price in reverse
    /// Return type: `BidsResponse`
    ReverseBidsSortedByPrice {
        start_before: Option<BidOffset>,
        limit: Option<u32>,
    },
    /// Show all registered ask hooks
    /// Return type: `HooksResponse`
    AskHooks {},
    /// Show all registered bid hooks
    /// Return type: `HooksResponse`
    BidHooks {},
    /// Show all registered sale hooks
    /// Return type: `HooksResponse`
    SaleHooks {},
    /// Get the config for the contract
    /// Return type: `ParamsResponse`
    Params {},
    /// Get the renewal queue for a specific height
    RenewalQueue { height: u64 },
    /// Get the minter and collection
    Config {},
}

#[cw_serde]
pub struct ConfigResponse {
    pub minter: Addr,
    pub collection: Addr,
}

#[cw_serde]
pub struct RenewalQueueResponse {
    pub queue: Vec<TokenId>,
}

#[cw_serde]
pub struct AskResponse {
    pub ask: Option<Ask>,
}

#[cw_serde]
pub struct AsksResponse {
    pub asks: Vec<Ask>,
}

#[cw_serde]
pub struct AskCountResponse {
    pub count: u32,
}

#[cw_serde]
pub struct BidResponse {
    pub bid: Option<Bid>,
}

#[cw_serde]
pub struct BidsResponse {
    pub bids: Vec<Bid>,
}

#[cw_serde]
pub struct ParamsResponse {
    pub params: SudoParams,
}

#[cw_serde]
pub struct SaleHookMsg {
    pub token_id: String,
    pub seller: String,
    pub buyer: String,
}

impl SaleHookMsg {
    pub fn new(token_id: &str, seller: String, buyer: String) -> Self {
        SaleHookMsg {
            token_id: token_id.to_string(),
            seller,
            buyer,
        }
    }

    /// serializes the message
    pub fn into_binary(self) -> StdResult<Binary> {
        let msg = SaleExecuteMsg::SaleHook(self);
        to_binary(&msg)
    }
}

// This is just a helper to properly serialize the above message
#[cw_serde]
pub enum SaleExecuteMsg {
    SaleHook(SaleHookMsg),
}

#[cw_serde]
pub enum HookAction {
    Create,
    Update,
    Delete,
}

#[cw_serde]
pub struct AskHookMsg {
    pub ask: Ask,
}

impl AskHookMsg {
    pub fn new(ask: Ask) -> Self {
        AskHookMsg { ask }
    }

    /// serializes the message
    pub fn into_binary(self, action: HookAction) -> StdResult<Binary> {
        let msg = match action {
            HookAction::Create => AskHookExecuteMsg::AskCreatedHook(self),
            HookAction::Update => AskHookExecuteMsg::AskUpdatedHook(self),
            HookAction::Delete => AskHookExecuteMsg::AskDeletedHook(self),
        };
        to_binary(&msg)
    }
}

// This is just a helper to properly serialize the above message
#[cw_serde]
pub enum AskHookExecuteMsg {
    AskCreatedHook(AskHookMsg),
    AskUpdatedHook(AskHookMsg),
    AskDeletedHook(AskHookMsg),
}

#[cw_serde]
pub struct BidHookMsg {
    pub bid: Bid,
}

impl BidHookMsg {
    pub fn new(bid: Bid) -> Self {
        BidHookMsg { bid }
    }

    /// serializes the message
    pub fn into_binary(self, action: HookAction) -> StdResult<Binary> {
        let msg = match action {
            HookAction::Create => BidExecuteMsg::BidCreatedHook(self),
            HookAction::Update => BidExecuteMsg::BidUpdatedHook(self),
            HookAction::Delete => BidExecuteMsg::BidDeletedHook(self),
        };
        to_binary(&msg)
    }
}

// This is just a helper to properly serialize the above message
#[cw_serde]
pub enum BidExecuteMsg {
    BidCreatedHook(BidHookMsg),
    BidUpdatedHook(BidHookMsg),
    BidDeletedHook(BidHookMsg),
}
