use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Uint128;

#[cw_serde]
pub struct InstantiateMsg {
    /// Temporary admin for managing whitelists
    pub admin: Option<String>,
    pub collection_code_id: u64,
    pub marketplace_addr: String,
    pub min_name_length: u64,
    pub max_name_length: u64,
    pub base_price: Uint128,
}

#[cw_serde]
pub enum ExecuteMsg {
    /// Change the admin that manages the whitelist
    /// Will be set to null after go-to-market
    UpdateAdmin {
        admin: Option<String>,
    },
    UpdateWhitelist {
        whitelist: Option<String>,
    },
    MintAndList {
        name: String,
    },
}

#[cw_serde]
pub enum SudoMsg {
    UpdateParams {
        min_name_length: u64,
        max_name_length: u64,
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
    #[returns(ConfigResponse)]
    Config {},
    #[returns(ParamsResponse)]
    Params {},
}

// We define a custom struct for each query response
#[cw_serde]
pub struct ConfigResponse {
    pub collection_addr: String,
}

#[cw_serde]
pub struct ParamsResponse {
    pub min_name_length: u64,
    pub max_name_length: u64,
    pub base_price: Uint128,
}
