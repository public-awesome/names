#[cfg(not(feature = "library"))]
use cosmwasm_std::{Addr, Deps, DepsMut, MessageInfo};

use crate::error::ContractError;
use cw721_base::Extension;
use cw_utils::nonpayable;
use sg721_base::ContractError::Unauthorized;
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
    if is_sender_token_id_owner(deps.as_ref(), info.sender, &token_id) == false {
        return Err(ContractError::Base(Unauthorized {}));
    }
    // TODO check NFT owner is address

    Sg721NameContract::default()
        .tokens
        .update(deps.storage, &token_id, |token| match token {
            Some(mut token_info) => {
                token_info.extension.profile = profile.clone();
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
    record: TextRecord,
) -> Result<Response, ContractError> {
    nonpayable(&info)?;
    let token_id = name;
    if is_sender_token_id_owner(deps.as_ref(), info.sender, &token_id) == false {
        return Err(ContractError::Base(Unauthorized {}));
    }
    // TODO validate record

    Sg721NameContract::default()
        .tokens
        .update(deps.storage, &token_id, |token| match token {
            Some(mut token_info) => {
                // TODO throw error if record name already exists
                token_info.extension.records.push(record.clone());
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
    record: TextRecord,
) -> Result<Response, ContractError> {
    nonpayable(&info)?;
    let token_id = name;
    if is_sender_token_id_owner(deps.as_ref(), info.sender, &token_id) == false {
        return Err(ContractError::Base(Unauthorized {}));
    }
    // TODO validate record

    Sg721NameContract::default()
        .tokens
        .update(deps.storage, &token_id, |token| match token {
            Some(mut token_info) => {
                token_info
                    .extension
                    .records
                    .retain(|r| r.name != record.name);
                token_info.extension.records.push(record.clone());
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
