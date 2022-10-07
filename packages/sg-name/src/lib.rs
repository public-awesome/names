use cosmwasm_std::{Addr, Timestamp};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub const MAX_TEXT_LENGTH: u64 = 512;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct NFT {
    pub collection: Addr,
    pub token_id: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct TextRecord {
    pub name: String,  // "twitter"
    pub value: String, // "shan3v"
    pub verified_at: Option<Timestamp>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Metadata<T> {
    pub bio: Option<String>,
    pub profile: Option<NFT>,
    pub records: Vec<TextRecord>,
    pub extension: T,
}
