use cosmwasm_schema::{cw_serde, QueryResponses};

#[cw_serde]
pub struct InstantiateMsg {
    pub addresses: Vec<String>,
    pub per_address_limit: u32,
}

#[cw_serde]
pub enum ExecuteMsg {
    UpdateAdmin { new_admin: String },
    AddAddresses { addresses: Vec<String> },
    RemoveAddresses { addresses: Vec<String> },
    // Add message to increment mint count on whitelist map. if mint succeeds, map increment will also succeed.
    ProcessAddress { address: String },
    UpdatePerAddressLimit { limit: u32 },
    Purge {},
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(IncludesAddressResponse)]
    IncludesAddress { address: String },
    #[returns(CountResponse)]
    MintCount { address: String },
    #[returns(cw_controllers::AdminResponse)]
    Admin {},
    #[returns(CountResponse)]
    Count {},
    #[returns(PerAddressLimitResponse)]
    PerAddressLimit {},
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
