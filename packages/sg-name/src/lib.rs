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
    /// Returns NameMarketplaceResponse
    NameMarketplace {},
}

#[cw_serde]
pub struct NameMarketplaceResponse {
    pub address: String,
}
