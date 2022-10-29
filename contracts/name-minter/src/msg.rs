use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Uint128};

#[cw_serde]
pub struct InstantiateMsg {
    /// Temporary admin for managing whitelists
    pub admin: Option<String>,
    pub collection_code_id: u64,
    pub marketplace_addr: String,
    pub min_name_length: u32,
    pub max_name_length: u32,
    pub base_price: Uint128,
    pub whitelists: Vec<String>,
}

#[cw_serde]
pub enum ExecuteMsg {
    /// Mint a name and list on Stargaze Name Marketplace
    MintAndList { name: String },
    /// Change the admin that manages the whitelist
    /// Will be set to null after go-to-market
    UpdateAdmin { admin: Option<String> },
    /// Admin can pause minting during whitelist switching
    Pause { pause: bool },
    /// Add a whitelist address
    AddWhitelist { address: String },
    /// Remove a whitelist address
    RemoveWhitelist { address: String },
}

#[cw_serde]
pub enum SudoMsg {
    UpdateParams {
        min_name_length: u32,
        max_name_length: u32,
        base_price: Uint128,
    },
    UpdateNameCollection {
        collection: String,
    },
    UpdateNameMarketplace {
        marketplace: String,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(cw_controllers::AdminResponse)]
    Admin {},
    #[returns(WhitelistsResponse)]
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
pub struct WhitelistsResponse {
    pub whitelists: Vec<Addr>,
}

#[cw_serde]
pub struct ParamsResponse {
    pub min_name_length: u32,
    pub max_name_length: u32,
    pub base_price: Uint128,
}
