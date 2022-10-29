use cosmwasm_schema::cw_serde;

#[cw_serde]
pub enum SgWhitelistExecuteMsg {
    /// Update admin of the list
    UpdateAdmin { admin: String },
    /// Add addresses to the list
    AddAddresses { addresses: Vec<String> },
    /// Remove an address from the list
    RemoveAddresses { addresses: Vec<String> },
    /// Called by another contract to process an address
    /// Returns true if the address is whitelisted and processed
    ProcessAddress { address: String },
    /// Update per address limit
    UpdatePerAddressLimit { limit: u32 },
}

#[cw_serde]
pub enum SgWhitelistQueryMsg {
    /// Query the current contract admin
    Admin {},
    /// Query the number of addresses
    AddressCount {},
    /// Query the per address limit
    PerAddressLimit { limit: u64 },
    /// Query if address is included
    IncludesAddress { address: String },
    /// Query if address is included and under per address limit
    IsProcessable { address: String },
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
pub struct IncludesAddressResponse {
    /// Whether the address is included in the whitelist
    pub included: bool,
}

#[cw_serde]
pub struct IsProcessableResponse {
    pub processable: bool,
}
