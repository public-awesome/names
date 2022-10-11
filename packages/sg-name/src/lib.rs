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
