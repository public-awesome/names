use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Binary, Timestamp};
use cw721::{
    AllNftInfoResponse, ApprovalResponse, ApprovalsResponse, ContractInfoResponse, Expiration,
    NftInfoResponse, NumTokensResponse, OperatorsResponse, OwnerOfResponse, TokensResponse,
};
use cw721_base::{MintMsg, MinterResponse};
use sg721::{ExecuteMsg as Sg721ExecuteMsg, RoyaltyInfoResponse, UpdateCollectionInfoMsg};
use sg721_base::msg::{CollectionInfoResponse, QueryMsg as Sg721QueryMsg};
use sg_name::{Metadata, NameMarketplaceResponse, NameResponse, TextRecord, NFT};

// Add execute msgs related to bio, profile, text records
// The rest are inherited from sg721 and impl to properly convert the msgs.
#[cw_serde]
pub enum ExecuteMsg<T> {
    /// Set name marketplace contract address
    SetNameMarketplace { address: String },
    /// Set an address for name reverse lookup
    /// Can be an EOA or a contract address
    AssociateAddress {
        name: String,
        address: Option<String>,
    },
    /// Update metadata
    UpdateMetadata {
        name: String,
        metadata: Option<Metadata>,
    },
    /// Update image NFT
    UpdateImageNft { name: String, nft: Option<NFT> },
    /// Update profile
    UpdateProfileNft {
        name: String,
        token_id: Option<String>,
    },
    /// Add text record ex: twitter handle, discord name, etc
    AddTextRecord { name: String, record: TextRecord },
    /// Remove text record ex: twitter handle, discord name, etc
    RemoveTextRecord { name: String, record_name: String },
    /// Update text record ex: twitter handle, discord name, etc
    UpdateTextRecord { name: String, record: TextRecord },
    /// Transfer is a base message to move a token to another account without triggering actions
    TransferNft { recipient: String, token_id: String },
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
    Revoke { spender: String, token_id: String },
    /// Allows operator to transfer / send any token from the owner's account.
    /// If expiration is set, then this allowance has a time/height limit
    ApproveAll {
        operator: String,
        expires: Option<Expiration>,
    },
    /// Remove previously granted ApproveAll permission
    RevokeAll { operator: String },
    /// Mint a new NFT, can only be called by the contract minter
    Mint(MintMsg<T>),
    /// Burn an NFT the sender has access to
    Burn { token_id: String },
    /// Update specific collection info fields
    UpdateCollectionInfo {
        collection_info: UpdateCollectionInfoMsg<RoyaltyInfoResponse>,
    },
    /// Called by the minter to update trading start time
    UpdateStartTradingTime(Option<Timestamp>),
    /// Freeze collection info from further updates
    FreezeCollectionInfo {},
}

impl<T, E> From<ExecuteMsg<T>> for Sg721ExecuteMsg<T, E> {
    fn from(msg: ExecuteMsg<T>) -> Sg721ExecuteMsg<T, E> {
        match msg {
            ExecuteMsg::TransferNft {
                recipient,
                token_id,
            } => Sg721ExecuteMsg::TransferNft {
                recipient,
                token_id,
            },
            ExecuteMsg::SendNft {
                contract,
                token_id,
                msg,
            } => Sg721ExecuteMsg::SendNft {
                contract,
                token_id,
                msg,
            },
            ExecuteMsg::Approve {
                spender,
                token_id,
                expires,
            } => Sg721ExecuteMsg::Approve {
                spender,
                token_id,
                expires,
            },
            ExecuteMsg::ApproveAll { operator, expires } => {
                Sg721ExecuteMsg::ApproveAll { operator, expires }
            }
            ExecuteMsg::Revoke { spender, token_id } => {
                Sg721ExecuteMsg::Revoke { spender, token_id }
            }
            ExecuteMsg::RevokeAll { operator } => Sg721ExecuteMsg::RevokeAll { operator },
            ExecuteMsg::Burn { token_id } => Sg721ExecuteMsg::Burn { token_id },
            ExecuteMsg::UpdateCollectionInfo { collection_info } => {
                Sg721ExecuteMsg::UpdateCollectionInfo { collection_info }
            }
            ExecuteMsg::UpdateStartTradingTime(start_time) => {
                Sg721ExecuteMsg::UpdateStartTradingTime(start_time)
            }
            ExecuteMsg::FreezeCollectionInfo {} => Sg721ExecuteMsg::FreezeCollectionInfo {},
            ExecuteMsg::Mint(MintMsg {
                token_id,
                owner,
                token_uri,
                extension,
            }) => Sg721ExecuteMsg::Mint(MintMsg {
                token_id,
                owner,
                token_uri,
                extension,
            }),
            _ => unreachable!("Invalid ExecuteMsg"),
        }
    }
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    /// Returns max record count
    #[returns(u32)]
    MaxRecordCount {},
    /// Reverse lookup of name for address
    #[returns(NameResponse)]
    Name { address: String },
    /// Returns the marketplace contract address
    #[returns(NameMarketplaceResponse)]
    NameMarketplace {},
    #[returns(OwnerOfResponse)]
    OwnerOf {
        token_id: String,
        include_expired: Option<bool>,
    },
    #[returns(ApprovalResponse)]
    Approval {
        token_id: String,
        spender: String,
        include_expired: Option<bool>,
    },
    #[returns(ApprovalsResponse)]
    Approvals {
        token_id: String,
        include_expired: Option<bool>,
    },
    #[returns(OperatorsResponse)]
    AllOperators {
        owner: String,
        include_expired: Option<bool>,
        start_after: Option<String>,
        limit: Option<u32>,
    },
    #[returns(NumTokensResponse)]
    NumTokens {},
    #[returns(ContractInfoResponse)]
    ContractInfo {},
    #[returns(NftInfoResponse<Metadata>)]
    NftInfo { token_id: String },
    #[returns(AllNftInfoResponse<Metadata>)]
    AllNftInfo {
        token_id: String,
        include_expired: Option<bool>,
    },
    #[returns(TokensResponse)]
    Tokens {
        owner: String,
        start_after: Option<String>,
        limit: Option<u32>,
    },
    #[returns(TokensResponse)]
    AllTokens {
        start_after: Option<String>,
        limit: Option<u32>,
    },
    #[returns(MinterResponse)]
    Minter {},
    #[returns(CollectionInfoResponse)]
    CollectionInfo {},
}

impl From<QueryMsg> for Sg721QueryMsg {
    fn from(msg: QueryMsg) -> Sg721QueryMsg {
        match msg {
            QueryMsg::OwnerOf {
                token_id,
                include_expired,
            } => Sg721QueryMsg::OwnerOf {
                token_id,
                include_expired,
            },
            QueryMsg::Approval {
                token_id,
                spender,
                include_expired,
            } => Sg721QueryMsg::Approval {
                token_id,
                spender,
                include_expired,
            },
            QueryMsg::Approvals {
                token_id,
                include_expired,
            } => Sg721QueryMsg::Approvals {
                token_id,
                include_expired,
            },
            QueryMsg::AllOperators {
                owner,
                include_expired,
                start_after,
                limit,
            } => Sg721QueryMsg::AllOperators {
                owner,
                include_expired,
                start_after,
                limit,
            },
            QueryMsg::NumTokens {} => Sg721QueryMsg::NumTokens {},
            QueryMsg::ContractInfo {} => Sg721QueryMsg::ContractInfo {},
            QueryMsg::NftInfo { token_id } => Sg721QueryMsg::NftInfo { token_id },
            QueryMsg::AllNftInfo {
                token_id,
                include_expired,
            } => Sg721QueryMsg::AllNftInfo {
                token_id,
                include_expired,
            },
            QueryMsg::Tokens {
                owner,
                start_after,
                limit,
            } => Sg721QueryMsg::Tokens {
                owner,
                start_after,
                limit,
            },
            QueryMsg::AllTokens { start_after, limit } => {
                Sg721QueryMsg::AllTokens { start_after, limit }
            }
            QueryMsg::Minter {} => Sg721QueryMsg::Minter {},
            _ => unreachable!("cannot convert {:?} to Cw721QueryMsg", msg),
        }
    }
}
