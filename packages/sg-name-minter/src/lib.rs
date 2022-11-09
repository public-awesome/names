use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Decimal, Timestamp, Uint128};

pub const PUBLIC_MINT_START_TIME_IN_SECONDS: Timestamp = Timestamp::from_seconds(1669406400);
#[cw_serde]
pub struct SudoParams {
    /// 3 (same as DNS)
    pub min_name_length: u32,
    /// 63 (same as DNS)
    pub max_name_length: u32,
    /// 100_000_000 (5+ ASCII char price)
    pub base_price: Uint128,
    /// Fair Burn fee (rest goes to Community Pool)
    pub fair_burn_percent: Decimal,
}

#[cw_serde]
pub enum SgNameMinterExecuteMsg {
    /// Mint a name and list on Stargaze Name Marketplace
    MintAndList { name: String },
    /// Change the admin that manages the whitelist
    /// Will be set to null after go-to-market
    UpdateAdmin { admin: Option<String> },
    /// Add a whiltelist address
    AddWhitelist { address: String },
    /// Remove a whitelist address
    RemoveWhitelist { address: String },
    /// Update config, only callable by admin
    /// will not be callable after admin is removed
    UpdateConfig { config: Config },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum SgNameMinterQueryMsg {
    #[returns(cw_controllers::AdminResponse)]
    Admin {},
    #[returns(Vec<Addr>)]
    Whitelists {},
    #[returns(CollectionResponse)]
    Collection {},
    #[returns(ParamsResponse)]
    Params {},
}

#[cw_serde]
pub struct CollectionResponse {
    pub collection: String,
}

#[cw_serde]
pub struct ParamsResponse {
    pub params: SudoParams,
}

#[cw_serde]
pub struct Config {
    pub public_mint_start_time: Timestamp,
}

#[cw_serde]
pub struct ConfigResponse {
    pub config: Config,
}
