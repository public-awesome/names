use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Timestamp};

pub const MAX_TEXT_LENGTH: u64 = 512;

#[cw_serde]
pub struct NFT {
    pub collection: Addr,
    pub token_id: String,
}

#[cw_serde]
pub struct TextRecord {
    pub name: String,  // "twitter"
    pub value: String, // "shan3v"
    pub verified_at: Option<Timestamp>,
}

/// Note that the address mapped to the name is stored in `token_uri`.
#[cw_serde]
pub struct Metadata<T> {
    pub bio: Option<String>,
    pub profile: Option<NFT>,
    pub records: Vec<TextRecord>,
    pub extension: T,
}

#[cw_serde]
pub enum SgNameExecuteMsg {
    /// Set name marketplace contract address
    SetNameMarketplace { address: String },
    /// Update bio
    UpdateBio { name: String, bio: Option<String> },
    /// Update profile
    UpdateProfile { name: String, profile: Option<NFT> },
    /// Add text record ex: twitter handle, discord name, etc
    AddTextRecord { name: String, record: TextRecord },
    /// Remove text record ex: twitter handle, discord name, etc
    RemoveTextRecord { name: String, record_name: String },
    /// Update text record ex: twitter handle, discord name, etc
    UpdateTextRecord { name: String, record: TextRecord },
}

#[cw_serde]
pub enum SgNameQueryMsg {
    /// Returns the name for the given address (`NameResponse`).
    /// `address` can be any Bech32 encoded address. It will be
    /// converted to a stars address for internal mapping.
    Name { address: String },
    /// Returns NameMarketplaceResponse
    NameMarketplace {},
}

#[cw_serde]
pub struct NameMarketplaceResponse {
    pub address: String,
}

/// Returns the `token_id`
#[cw_serde]
pub struct NameResponse {
    pub name: String,
}
