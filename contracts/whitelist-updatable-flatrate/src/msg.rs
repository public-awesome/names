use cosmwasm_schema::{cw_serde, QueryResponses};

use crate::state::Config;

#[cw_serde]
pub struct InstantiateMsg {
    pub addresses: Vec<String>,
    pub per_address_limit: u32,
    pub mint_discount_amount: Option<u64>,
    pub admin_list: Option<Vec<String>>,
}

#[cw_serde]
pub enum ExecuteMsg {
    UpdateAdmins {
        new_admin_list: Vec<String>,
    },
    AddAddresses {
        addresses: Vec<String>,
    },
    RemoveAddresses {
        addresses: Vec<String>,
    },
    /// Only callable by minter contract. Increment mint count on whitelist map.
    ProcessAddress {
        address: String,
    },
    UpdatePerAddressLimit {
        limit: u32,
    },
    Purge {},
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(Config)]
    Config {},
    #[returns(bool)]
    IncludesAddress { address: String },
    #[returns(u64)]
    MintCount { address: String },
    /// Avoid processing addresses that will fail. Includes address and under per address limit
    #[returns(bool)]
    IsProcessable { address: String },
    #[returns(cw_controllers::AdminResponse)]
    Admins {},
    #[returns(u64)]
    AddressCount {},
    #[returns(u64)]
    PerAddressLimit {},
    // Mint discount converts bps to decimal percentage
    #[returns(u64)]
    MintDiscountAmount {},
}
