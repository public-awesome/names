use cosmwasm_schema::{cw_serde, QueryResponses};

use crate::state::Config;

#[cw_serde]
pub struct InstantiateMsg {
    pub addresses: Vec<String>,
    pub per_address_limit: u32,
    pub mint_discount_bps: Option<u64>,
}

#[cw_serde]
pub enum ExecuteMsg {
    UpdateAdmin {
        new_admin: String,
    },
    AddAddresses {
        addresses: Vec<String>,
    },
    RemoveAddresses {
        addresses: Vec<String>,
    },
    /// Add message to increment mint count on whitelist map. if mint succeeds, map increment will also succeed.
    ProcessAddress {
        address: String,
    },
    UpdatePerAddressLimit {
        limit: u32,
    },
    UpdateMinterContract {
        minter_contract: String,
    },
    Purge {},
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(ConfigResponse)]
    Config {},
    #[returns(IncludesAddressResponse)]
    IncludesAddress { address: String },
    #[returns(CountResponse)]
    MintCount { address: String },
    /// Avoid processing addresses that will fail. Includes address and under per address limit
    #[returns(IsProcessableResponse)]
    IsProcessable { address: String },
    #[returns(cw_controllers::AdminResponse)]
    Admin {},
    #[returns(CountResponse)]
    Count {},
    #[returns(PerAddressLimitResponse)]
    PerAddressLimit {},
}

#[cw_serde]
pub struct ConfigResponse {
    pub config: Config,
}
#[cw_serde]
pub struct IncludesAddressResponse {
    pub includes: bool,
}

#[cw_serde]
pub struct CountResponse {
    pub count: u64,
}

#[cw_serde]
pub struct PerAddressLimitResponse {
    pub limit: u64,
}

#[cw_serde]
pub struct IsProcessableResponse {
    pub processable: bool,
}
