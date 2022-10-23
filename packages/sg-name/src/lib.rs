use cosmwasm_schema::cw_serde;
use cosmwasm_std::Addr;

pub const MAX_TEXT_LENGTH: u32 = 512;

#[cw_serde]
pub struct NFT {
    pub collection: Addr,
    pub token_id: String,
}

#[cw_serde]
pub struct TextRecord {
    pub name: String,  // "twitter"
    pub value: String, // "shan3v"
}

/// Note that the address mapped to the name is stored in `token_uri`.
#[cw_serde]
#[derive(Default)]
pub struct Metadata {
    pub image_nft: Option<NFT>,
    pub records: Vec<TextRecord>,
}

#[cw_serde]
pub enum SgNameExecuteMsg {
    /// Set name marketplace contract address
    SetNameMarketplace { address: String },
    /// Set an address for name reverse lookup
    /// Can be an EOA or a contract address
    AssociateAddress { name: String, address: String },
    /// Update profile
    UpdateProfileNft { name: String, nft: Option<NFT> },
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
