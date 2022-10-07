use cosmwasm_std::{Binary, Timestamp};
use cw721::Expiration;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use sg721::{MintMsg, RoyaltyInfoResponse, UpdateCollectionInfoMsg};
use sg_name::{TextRecord, NFT};

// Add execute msgs related to bio, profile, text records
// The rest are inherited from sg721 and impl to properly convert the msgs.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub enum ExecuteMsg<T> {
    /// Update bio
    UpdateBio {
        name: String,
        bio: Option<String>,
    },
    /// Update profile
    UpdateProfile {
        name: String,
        profile: Option<NFT>,
    },
    /// Add text record ex: twitter handle, discord name, etc
    AddTextRecord {
        name: String,
        record: TextRecord,
    },
    /// Remove text record ex: twitter handle, discord name, etc
    RemoveTextRecord {
        name: String,
        record_name: String,
    },
    /// Update text record ex: twitter handle, discord name, etc
    UpdateTextRecord {
        name: String,
        record: TextRecord,
    },
    /// Transfer is a base message to move a token to another account without triggering actions
    TransferNft {
        recipient: String,
        token_id: String,
    },
    /// Send is a base message to transfer a token to a contract and trigger an action
    /// on the receiving contract.
    SendNft {
        contract: String,
        token_id: String,
        msg: Binary,
    },
    /// Allows operator to transfer / send the token from the owner's account.
    /// If expiration is set, then this allowance has a time/height limit
    Approve {
        spender: String,
        token_id: String,
        expires: Option<Expiration>,
    },
    /// Remove previously granted Approval
    Revoke {
        spender: String,
        token_id: String,
    },
    /// Allows operator to transfer / send any token from the owner's account.
    /// If expiration is set, then this allowance has a time/height limit
    ApproveAll {
        operator: String,
        expires: Option<Expiration>,
    },
    /// Remove previously granted ApproveAll permission
    RevokeAll {
        operator: String,
    },
    /// Mint a new NFT, can only be called by the contract minter
    Mint(MintMsg<T>),
    /// Burn an NFT the sender has access to
    Burn {
        token_id: String,
    },
    /// Update specific collection info fields
    UpdateCollectionInfo {
        collection_info: UpdateCollectionInfoMsg<RoyaltyInfoResponse>,
    },
    /// Called by the minter to update trading start time
    UpdateTradingStartTime(Option<Timestamp>),
    // Freeze collection info from further updates
    FreezeCollectionInfo,
}
