use crate::state::{Ask, Bid, Id, SudoParams, TokenId};
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{to_binary, Addr, Binary, Coin, Decimal, StdResult, Timestamp, Uint128};
use sg_controllers::HooksResponse;

#[cw_serde]
pub struct InstantiateMsg {
    /// Community pool fee for winning bids
    /// 0.25% = 25, 0.5% = 50, 1% = 100, 2.5% = 250
    pub trading_fee_bps: u64,
    /// Min value for bids and asks
    pub min_price: Uint128,
    /// Interval to rate limit setting asks (in seconds)
    pub ask_interval: u64,
    /// The maximum number of renewals that can be processed in each block
    pub max_renewals_per_block: u32,
    /// The number of bids to query to when searching for the highest bid
    pub valid_bid_query_limit: u32,
    /// The number of seconds before the current block time that a
    /// bid must have been created in order to be considered valid.
    /// Also, the number of seconds before an ask expires where it can be renewed.
    pub renew_window: u64,
    /// The percentage of the winning bid that must be paid to renew a name
    pub renewal_bid_percentage: Decimal,
    /// The address with permission to invoke process_renewals
    pub operator: String,
}

#[cw_serde]
pub enum ExecuteMsg {
    /// List name NFT on the marketplace by creating a new ask.
    /// Only the name minter can call this.
    SetAsk { token_id: TokenId, seller: String },
    /// Remove name on the marketplace.
    /// Only the name collection can call this (i.e: when burned).
    RemoveAsk { token_id: TokenId },
    /// Update ask when an NFT is transferred
    /// Only the name collection can call this
    UpdateAsk { token_id: TokenId, seller: String },
    /// Place a bid on an existing ask
    SetBid { token_id: TokenId },
    /// Remove an existing bid from an ask
    RemoveBid { token_id: TokenId },
    /// Accept a bid on an existing ask
    AcceptBid { token_id: TokenId, bidder: String },
    /// Migrate bids from the old index to the new index
    MigrateBids { limit: u32 },
    /// Fund renewal of a name
    FundRenewal { token_id: TokenId },
    /// Refund a renewal of a name
    RefundRenewal { token_id: TokenId },
    /// Fully renew a name if within the renewal period
    Renew { token_id: TokenId },
    /// Check if expired names have been paid for, and collect fees.
    /// If not paid, transfer ownership to the highest bidder.
    ProcessRenewals { limit: u32 },
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
        ask_interval: Option<u64>,
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
    /// Is called by x/cron module EndBlocker,
    /// and is used to process name renewals.
    EndBlock {},
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
pub struct AskRenewPriceResponse {
    pub token_id: TokenId,
    pub price: Coin,
    pub bid: Option<Bid>,
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    /// Get the current ask for specific name
    #[returns(Option<Ask>)]
    Ask { token_id: TokenId },
    /// Get all asks for a collection
    #[returns(Vec<Ask>)]
    Asks {
        start_after: Option<Id>,
        limit: Option<u32>,
    },
    /// Count of all asks
    #[returns(u64)]
    AskCount {},
    /// Get all asks by seller
    #[returns(Vec<Ask>)]
    AsksBySeller {
        seller: Seller,
        start_after: Option<TokenId>,
        limit: Option<u32>,
    },
    /// Get all renewable Asks
    #[returns(Vec<Ask>)]
    AsksByRenewTime {
        max_time: Timestamp,
        start_after: Option<Timestamp>,
        limit: Option<u32>,
    },
    /// Get the renewal price for a specific name
    #[returns((Option<Coin>, Option<Bid>))]
    AskRenewPrice {
        current_time: Timestamp,
        token_id: TokenId,
    },
    /// Get renewal price for multiple names
    #[returns(Vec<AskRenewPriceResponse>)]
    AskRenewalPrices {
        current_time: Timestamp,
        token_ids: Vec<TokenId>,
    },
    /// Get data for a specific bid
    #[returns(Option<Bid>)]
    Bid { token_id: TokenId, bidder: Bidder },
    /// Get all bids by a bidder
    #[returns(Vec<Bid>)]
    BidsByBidder {
        bidder: Bidder,
        start_after: Option<TokenId>,
        limit: Option<u32>,
    },
    /// Get all bids for a specific NFT
    #[returns(Vec<Bid>)]
    Bids {
        token_id: TokenId,
        start_after: Option<Bidder>,
        limit: Option<u32>,
    },
    /// Get all legacy bids
    #[returns(Vec<Bid>)]
    LegacyBids {
        start_after: Option<BidOffset>,
        limit: Option<u32>,
    },
    /// Get all bids for a collection, sorted by price
    #[returns(Vec<Bid>)]
    BidsSortedByPrice {
        start_after: Option<BidOffset>,
        limit: Option<u32>,
    },
    /// Get all bids for a collection, sorted by price in reverse
    #[returns(Vec<Bid>)]
    ReverseBidsSortedByPrice {
        start_before: Option<BidOffset>,
        limit: Option<u32>,
    },
    /// Get all bids for a specific account
    #[returns(Vec<Bid>)]
    BidsForSeller {
        seller: String,
        start_after: Option<BidOffset>,
        limit: Option<u32>,
    },
    /// Get the highest bid for a name
    #[returns(Option<Bid>)]
    HighestBid { token_id: TokenId },
    /// Show all registered ask hooks
    #[returns(HooksResponse)]
    AskHooks {},
    /// Show all registered bid hooks
    #[returns(HooksResponse)]
    BidHooks {},
    /// Show all registered sale hooks
    #[returns(HooksResponse)]
    SaleHooks {},
    /// Get the config for the contract
    #[returns(SudoParams)]
    Params {},
    /// Get the renewal queue for a specific time
    #[returns(Vec<Ask>)]
    RenewalQueue { time: Timestamp },
    /// Get the minter and collection
    #[returns(ConfigResponse)]
    Config {},
}

#[cw_serde]
pub struct ConfigResponse {
    pub minter: Addr,
    pub collection: Addr,
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
