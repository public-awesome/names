use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Decimal, Uint128};

#[cw_serde]
pub enum SgNameMinterExecuteMsg {
    /// Mint a name and list on Stargaze Name Marketplace
    MintAndList { name: String },
    /// Change the admin that manages the whitelist
    /// Will be set to null after go-to-market
    UpdateAdmin { admin: Option<String> },
    /// Add a whiltelist address
    AddWhitelist { address: String },
    /// Remove a whiltelist address
    RemoveWhitelist { address: String },
}

#[cw_serde]
pub enum SgNameMinterQueryMsg {
    Admin {},
    Whitelists {},
    Collection {},
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
    pub fair_burn_percent: Decimal,
}
