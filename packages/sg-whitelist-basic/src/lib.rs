use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Addr;

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
#[derive(QueryResponses)]
pub enum SgWhitelistQueryMsg {
    /// Query the current contract admin
    #[returns(Addr)]
    Admin {},
    /// Query the number of addresses
    #[returns(u64)]
    AddressCount {},
    /// Query the per address limit
    #[returns(u64)]
    PerAddressLimit { limit: u64 },
    /// Query if address is included
    #[returns(bool)]
    IncludesAddress { address: String },
    /// Query if address is included and under per address limit
    #[returns(bool)]
    IsProcessable { address: String },
}
