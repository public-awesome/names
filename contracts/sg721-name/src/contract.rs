use crate::error::ContractError;

use cosmwasm_std::{to_binary, Addr, Deps, DepsMut, Env, MessageInfo, WasmQuery};

use cw721::OwnerOfResponse;
use cw721_base::Extension;
use cw_utils::nonpayable;

use sg721::ExecuteMsg as Sg721ExecuteMsg;
use sg721_base::{msg::QueryMsg as Sg721QueryMsg, ContractError::Unauthorized};
use sg_name::{Metadata, TextRecord, MAX_TEXT_LENGTH, NFT};
use sg_std::Response;

pub type Sg721NameContract<'a> = sg721_base::Sg721Contract<'a, Metadata<Extension>>;

pub fn execute_update_bio(
    deps: DepsMut,
    info: MessageInfo,
    name: String,
    bio: Option<String>,
) -> Result<Response, ContractError> {
    nonpayable(&info)?;
    let token_id = name;
    if !is_sender_token_id_owner(deps.as_ref(), info.sender, &token_id) {
        return Err(ContractError::Base(Unauthorized {}));
    }
    validate_bio(bio.clone())?;

    Sg721NameContract::default()
        .tokens
        .update(deps.storage, &token_id, |token| match token {
            Some(mut token_info) => {
                token_info.extension.bio = bio;
                Ok(token_info)
            }
            None => Err(ContractError::NameNotFound {}),
        })?;
    Ok(Response::new())
}

pub fn execute_update_profile(
    deps: DepsMut,
    info: MessageInfo,
    name: String,
    profile: Option<NFT>,
) -> Result<Response, ContractError> {
    nonpayable(&info)?;
    let token_id = name;
    if !is_sender_token_id_owner(deps.as_ref(), info.sender.clone(), &token_id) {
        return Err(ContractError::Base(Unauthorized {}));
    }

    if let Some(profile) = profile.clone() {
        let req = WasmQuery::Smart {
            contract_addr: profile.collection.to_string(),
            msg: to_binary(&Sg721QueryMsg::OwnerOf {
                token_id: profile.token_id,
                include_expired: None,
            })?,
        }
        .into();
        let res: OwnerOfResponse = deps.querier.query(&req)?;
        if res.owner != info.sender {
            return Err(ContractError::NotNFTOwner {});
        }
    }

    Sg721NameContract::default()
        .tokens
        .update(deps.storage, &token_id, |token| match token {
            Some(mut token_info) => {
                token_info.extension.profile = profile;
                Ok(token_info)
            }
            None => Err(ContractError::NameNotFound {}),
        })?;
    Ok(Response::new())
}

pub fn execute_add_text_record(
    deps: DepsMut,
    info: MessageInfo,
    name: String,
    mut record: TextRecord,
) -> Result<Response, ContractError> {
    nonpayable(&info)?;
    let token_id = name;
    if !is_sender_token_id_owner(deps.as_ref(), info.sender, &token_id) {
        return Err(ContractError::Base(Unauthorized {}));
    }
    validate_and_sanitize_record(&mut record)?;

    Sg721NameContract::default()
        .tokens
        .update(deps.storage, &token_id, |token| match token {
            Some(mut token_info) => {
                // can not add a record with existing name
                for r in token_info.extension.records.iter() {
                    if r.name == record.name {
                        return Err(ContractError::RecordNameAlreadyExists {});
                    }
                }
                token_info.extension.records.push(record);
                Ok(token_info)
            }
            None => Err(ContractError::NameNotFound {}),
        })?;
    Ok(Response::new())
}

pub fn execute_remove_text_record(
    deps: DepsMut,
    info: MessageInfo,
    name: String,
    record_name: String,
) -> Result<Response, ContractError> {
    nonpayable(&info)?;
    let token_id = name;
    if !is_sender_token_id_owner(deps.as_ref(), info.sender, &token_id) {
        return Err(ContractError::Base(Unauthorized {}));
    }

    Sg721NameContract::default()
        .tokens
        .update(deps.storage, &token_id, |token| match token {
            Some(mut token_info) => {
                token_info
                    .extension
                    .records
                    .retain(|r| r.name != record_name);
                Ok(token_info)
            }
            None => Err(ContractError::NameNotFound {}),
        })?;
    Ok(Response::new())
}

pub fn execute_update_text_record(
    deps: DepsMut,
    info: MessageInfo,
    name: String,
    mut record: TextRecord,
) -> Result<Response, ContractError> {
    nonpayable(&info)?;
    let token_id = name;
    if !is_sender_token_id_owner(deps.as_ref(), info.sender, &token_id) {
        return Err(ContractError::Base(Unauthorized {}));
    }
    validate_and_sanitize_record(&mut record)?;

    Sg721NameContract::default()
        .tokens
        .update(deps.storage, &token_id, |token| match token {
            Some(mut token_info) => {
                token_info
                    .extension
                    .records
                    .retain(|r| r.name != record.name);
                token_info.extension.records.push(record);
                Ok(token_info)
            }
            None => Err(ContractError::NameNotFound {}),
        })?;
    Ok(Response::new())
}

pub fn execute_transfer_nft(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    recipient: String,
    token_id: String,
) -> Result<Response, ContractError> {
    // sender can transfer check done on sg721 transfer tx
    nonpayable(&info)?;
    let mut token = Sg721NameContract::default()
        .tokens
        .may_load(deps.storage, &token_id)?
        .ok_or(ContractError::NameNotFound {})?;

    // reset bio, profile, records
    token.extension.bio = None;
    token.extension.profile = None;
    token.extension.records = vec![];
    Sg721NameContract::default()
        .tokens
        .save(deps.storage, &token_id, &token)?;

    let msg = Sg721ExecuteMsg::TransferNft {
        recipient,
        token_id,
    };
    Sg721NameContract::default().execute(deps, env, info, msg)?;

    Ok(Response::new())
}

fn is_sender_token_id_owner(deps: Deps, sender: Addr, token_id: &str) -> bool {
    let owner = match Sg721NameContract::default()
        .tokens
        .load(deps.storage, token_id)
    {
        Ok(token_info) => token_info.owner,
        Err(_) => return false,
    };
    owner == sender
}

fn validate_bio(bio: Option<String>) -> Result<(), ContractError> {
    if let Some(bio) = bio {
        let len = bio.len() as u64;
        if len > MAX_TEXT_LENGTH {
            return Err(ContractError::BioTooLong {});
        }
    }
    Ok(())
}

fn validate_and_sanitize_record(record: &mut TextRecord) -> Result<(), ContractError> {
    let len = record.name.len() as u64;
    if len > MAX_TEXT_LENGTH {
        return Err(ContractError::RecordNameTooLong {});
    }
    if len == 0 {
        return Err(ContractError::RecordNameEmpty {});
    }
    let len = record.value.len() as u64;
    if len > MAX_TEXT_LENGTH {
        return Err(ContractError::RecordValueTooLong {});
    }
    // new or updated records need to be verified
    record.verified_at = None;
    Ok(())
}
