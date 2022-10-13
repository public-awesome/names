use cosmwasm_schema::cw_serde;

#[cw_serde]
pub enum SgWhitelistExecuteMsg {
    /// Update admin of the list
    UpdateAdmin { admin: String },
    /// Add an address to the list
    AddAddress { address: String },
    /// Remove an address from the list
    RemoveAddress { address: String },
    /// Called by another contract to process an address
    /// Returns true if the address is whitelisted and processed
    ProcessAddress { address: String },
    /// Updatet the per address limit
    UpdatePerAddressLimit { limit: u64 },
}

#[cw_serde]
pub enum SgWhitelistQueryMsg {
    /// Query the current contract admin
    Admin {},
    /// Query the number of addresses
    Count {},
    /// Query the per address limit
    PerAddressLimit { limit: u64 },
    /// Query if address is included
    IncludesAddress { address: String },
    /// Query if address has been processed
    IsProcessed { address: String },
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
pub struct IsProcessedResponse {
    pub processed: bool,
}
