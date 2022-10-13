use crate::{
    error::ContractError,
    state::{ADDRESS_MAP, NAME_MARKETPLACE},
};

use cosmwasm_std::{to_binary, Addr, Deps, DepsMut, Env, MessageInfo, StdResult, WasmMsg};

use cw721_base::{state::TokenInfo, Extension, MintMsg};
use cw_utils::nonpayable;

use sg721::ExecuteMsg as Sg721ExecuteMsg;
use sg721_base::ContractError::{Claimed, Unauthorized};
use sg_name::{Metadata, NameMarketplaceResponse, NameResponse, TextRecord, MAX_TEXT_LENGTH, NFT};
use sg_name_market::SgNameMarketplaceExecuteMsg;
use sg_std::Response;

pub type Sg721NameContract<'a> = sg721_base::Sg721Contract<'a, Metadata<Extension>>;

pub fn execute_update_bio(
    deps: DepsMut,
    info: MessageInfo,
    name: String,
    bio: Option<String>,
) -> Result<Response, ContractError> {
    let token_id = name;

    nonpayable(&info)?;
    only_owner(deps.as_ref(), info.sender, &token_id)?;
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
    let token_id = name;

    nonpayable(&info)?;
    only_owner(deps.as_ref(), info.sender, &token_id)?;

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
    let token_id = name;

    nonpayable(&info)?;
    only_owner(deps.as_ref(), info.sender, &token_id)?;
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
    let token_id = name;

    nonpayable(&info)?;
    only_owner(deps.as_ref(), info.sender, &token_id)?;

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
    let token_id = name;

    nonpayable(&info)?;
    only_owner(deps.as_ref(), info.sender, &token_id)?;
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

pub fn execute_set_name_marketplace(
    deps: DepsMut,
    info: MessageInfo,
    address: String,
) -> Result<Response, ContractError> {
    nonpayable(&info)?;

    let minter = Sg721NameContract::default().minter.load(deps.storage)?;
    if minter != info.sender {
        return Err(ContractError::Base(Unauthorized {}));
    }

    NAME_MARKETPLACE.save(deps.storage, &deps.api.addr_validate(&address)?)?;

    Ok(Response::new())
}

pub fn execute_mint(
    deps: DepsMut,
    info: MessageInfo,
    msg: MintMsg<Metadata<Extension>>,
) -> Result<Response, ContractError> {
    let minter = Sg721NameContract::default().minter.load(deps.storage)?;
    if info.sender != minter {
        return Err(ContractError::Base(Unauthorized {}));
    }

    let token_uri = match msg.token_uri {
        Some(token_uri) => token_uri,
        None => return Err(ContractError::MissingTokenUri {}),
    };
    ADDRESS_MAP.save(
        deps.storage,
        &deps.api.addr_validate(&token_uri)?,
        &msg.token_id,
    )?;

    // create the token
    let token = TokenInfo {
        owner: deps.api.addr_validate(&msg.owner)?,
        approvals: vec![],          // TODO: set approval here?
        token_uri: Some(token_uri), // stars address
        extension: msg.extension,
    };
    Sg721NameContract::default()
        .tokens
        .update(deps.storage, &msg.token_id, |old| match old {
            Some(_) => Err(ContractError::Base(Claimed {})),
            None => Ok(token),
        })?;

    Sg721NameContract::default().increment_tokens(deps.storage)?;

    Ok(Response::new()
        .add_attribute("action", "mint")
        .add_attribute("minter", info.sender)
        .add_attribute("owner", msg.owner)
        .add_attribute("token_id", msg.token_id))
}

pub fn execute_transfer_nft(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    recipient: String,
    token_id: String,
) -> Result<Response, ContractError> {
    nonpayable(&info)?;

    // Update the ask on the marketplace
    let msg = SgNameMarketplaceExecuteMsg::UpdateAsk {
        token_id: token_id.to_string(),
        seller: recipient.to_string(),
    };
    let update_ask_msg = WasmMsg::Execute {
        contract_addr: NAME_MARKETPLACE.load(deps.storage)?.to_string(),
        funds: vec![],
        msg: to_binary(&msg)?,
    };

    let mut token = Sg721NameContract::default()
        .tokens
        .load(deps.storage, &token_id)?;

    // Reset bio, profile, records
    token.extension.bio = None;
    token.extension.profile = None;
    token.extension.records = vec![];
    Sg721NameContract::default()
        .tokens
        .save(deps.storage, &token_id, &token)?;

    let msg = Sg721ExecuteMsg::TransferNft {
        recipient,
        token_id: token_id.to_string(),
    };
    Sg721NameContract::default().execute(deps, env, info, msg)?;

    Ok(Response::new().add_message(update_ask_msg))
}

fn only_owner(deps: Deps, sender: Addr, token_id: &str) -> Result<Addr, ContractError> {
    let owner = Sg721NameContract::default()
        .tokens
        .load(deps.storage, token_id)?
        .owner;

    if owner != sender {
        return Err(ContractError::Base(Unauthorized {}));
    }

    Ok(owner)
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

pub fn query_name_marketplace(deps: Deps) -> StdResult<NameMarketplaceResponse> {
    let address = NAME_MARKETPLACE.load(deps.storage)?;

    Ok(NameMarketplaceResponse {
        address: address.to_string(),
    })
}

pub fn query_name(deps: Deps, address: String) -> StdResult<NameResponse> {
    // TODO: De-code and re-encode address if needed (for remote chains)
    let name = ADDRESS_MAP.load(deps.storage, &deps.api.addr_validate(&address)?)?;

    Ok(NameResponse { name })
}
