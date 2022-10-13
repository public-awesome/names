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
    #[returns(bool)]
    IncludesAddress { address: String },
    #[returns(Option<String>)]
    Admin {},
    #[returns(u64)]
    Count {},
    #[returns(u64)]
    PerAddressLimit {},
}
