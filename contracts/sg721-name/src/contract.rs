use cosmwasm_std::{to_binary, WasmQuery};
#[cfg(not(feature = "library"))]
use cosmwasm_std::{Addr, Deps, DepsMut, MessageInfo};
use cw721::OwnerOfResponse;

use crate::error::ContractError;
use cw721_base::Extension;
use cw_utils::nonpayable;
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
    if is_sender_token_id_owner(deps.as_ref(), info.sender, &token_id) == false {
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
    if is_sender_token_id_owner(deps.as_ref(), info.sender.clone(), &token_id) == false {
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
        if res.owner != info.sender.to_string() {
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
    if is_sender_token_id_owner(deps.as_ref(), info.sender, &token_id) == false {
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
    if is_sender_token_id_owner(deps.as_ref(), info.sender, &token_id) == false {
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
    if is_sender_token_id_owner(deps.as_ref(), info.sender, &token_id) == false {
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

fn is_sender_token_id_owner(deps: Deps, sender: Addr, token_id: &str) -> bool {
    let owner = match Sg721NameContract::default()
        .tokens
        .load(deps.storage, &token_id)
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
