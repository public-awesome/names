use crate::{
    error::ContractError,
    state::{NAME_MARKETPLACE, REVERSE_MAP},
};

use cosmwasm_std::{
    to_binary, Addr, ContractInfoResponse, Deps, DepsMut, Env, MessageInfo, StdError, StdResult,
    WasmMsg,
};

use cw721_base::{state::TokenInfo, MintMsg};
use cw_utils::nonpayable;

use sg721::ExecuteMsg as Sg721ExecuteMsg;
use sg721_base::ContractError::{Claimed, Unauthorized};
use sg_name::{Metadata, NameMarketplaceResponse, NameResponse, TextRecord, MAX_TEXT_LENGTH, NFT};
use sg_name_market::SgNameMarketplaceExecuteMsg;
use sg_std::Response;

use subtle_encoding::bech32;

pub type Sg721NameContract<'a> = sg721_base::Sg721Contract<'a, Metadata>;

pub fn execute_update_metadata(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    name: String,
    metadata: Option<Metadata>,
) -> Result<Response, ContractError> {
    let token_id = name;

    nonpayable(&info)?;
    only_owner(deps.as_ref(), &info.sender, &token_id)?;

    let mut token_info = Sg721NameContract::default()
        .tokens
        .load(deps.storage, &token_id)?;

    // Update to new metadata or current metadata
    match metadata {
        None => {
            // reset metadata to empty
            token_info.extension = Metadata::default();
            Sg721NameContract::default()
                .tokens
                .save(deps.storage, &token_id, &token_info)?;
        }
        Some(metadata) => {
            // update metadata
            token_info.extension.bio = match metadata.bio {
                None => token_info.extension.bio,
                Some(bio) => Some(bio),
            };

            // update nft profile
            token_info.extension.profile_nft = match metadata.profile_nft {
                None => token_info.extension.profile_nft,
                Some(profile_nft) => Some(profile_nft),
            };
            // update records. If empty, do nothing.
            if !metadata.records.is_empty() {
                for record in metadata.records.iter() {
                    // update same record name
                    token_info
                        .extension
                        .records
                        .retain(|r| r.name != record.name);
                    token_info.extension.records.push(record.clone());
                }
            };
        }
    };

    validate_bio(token_info.clone().extension.bio)?;

    Sg721NameContract::default()
        .tokens
        .update(deps.storage, &token_id, |token| match token {
            Some(mut new_token_info) => {
                new_token_info = token_info;
                Ok(new_token_info)
            }
            None => Err(ContractError::NameNotFound {}),
        })?;
    Ok(Response::new())
}

pub fn execute_associate_address(
    deps: DepsMut,
    info: MessageInfo,
    name: String,
    address: String,
) -> Result<Response, ContractError> {
    let sender = info.sender;

    only_owner(deps.as_ref(), &sender, &name)?;

    let token_uri = deps.api.addr_validate(&address)?;

    validate_address(deps.as_ref(), &sender, &token_uri.clone())?;

    Sg721NameContract::default()
        .tokens
        .update(deps.storage, &name, |token| match token {
            Some(mut token_info) => {
                token_info.token_uri = Some(address);
                Ok(token_info)
            }
            None => Err(ContractError::NameNotFound {}),
        })?;

    REVERSE_MAP.save(deps.storage, &token_uri, &name)?;

    Ok(Response::new())
}

pub fn execute_mint(
    deps: DepsMut,
    info: MessageInfo,
    msg: MintMsg<Metadata>,
) -> Result<Response, ContractError> {
    let minter = Sg721NameContract::default().minter.load(deps.storage)?;
    if info.sender != minter {
        return Err(ContractError::Base(Unauthorized {}));
    }
    // create the token
    let token = TokenInfo {
        owner: deps.api.addr_validate(&msg.owner)?,
        approvals: vec![],
        token_uri: None,
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

pub fn execute_burn(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    token_id: String,
) -> Result<Response, ContractError> {
    let token = Sg721NameContract::default()
        .tokens
        .load(deps.storage, &token_id)?;

    Sg721NameContract::default()
        .check_can_send(deps.as_ref(), &env, &info, &token)
        .map_err(|_| ContractError::Base(Unauthorized {}))?;

    if let Some(token_uri) = token.token_uri {
        REVERSE_MAP.remove(deps.storage, &Addr::unchecked(token_uri));
    }

    let msg = SgNameMarketplaceExecuteMsg::RemoveAsk {
        token_id: token_id.to_string(),
    };
    let remove_ask_msg = WasmMsg::Execute {
        contract_addr: NAME_MARKETPLACE.load(deps.storage)?.to_string(),
        funds: vec![],
        msg: to_binary(&msg)?,
    };

    // TODO: what to do with bids? refund bids?

    Sg721NameContract::default()
        .tokens
        .remove(deps.storage, &token_id)?;
    Sg721NameContract::default().decrement_tokens(deps.storage)?;

    Ok(Response::new()
        .add_attribute("action", "burn")
        .add_attribute("sender", info.sender)
        .add_attribute("token_id", token_id)
        .add_message(remove_ask_msg))
}

pub fn execute_transfer_nft(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    recipient: String,
    token_id: String,
) -> Result<Response, ContractError> {
    nonpayable(&info)?;
    let recipient = deps.api.addr_validate(&recipient)?;

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
    token.extension = Metadata::default();
    Sg721NameContract::default()
        .tokens
        .save(deps.storage, &token_id, &token)?;

    // TODO: reset token_uri
    // TODO: update approval?

    // remove reverse mapping if exists
    if let Some(token_uri) = token.token_uri {
        REVERSE_MAP.remove(deps.storage, &Addr::unchecked(token_uri));
    }

    let msg = Sg721ExecuteMsg::TransferNft {
        recipient: recipient.to_string(),
        token_id: token_id.to_string(),
    };
    Sg721NameContract::default().execute(deps, env, info, msg)?;

    Ok(Response::new().add_message(update_ask_msg))
}

pub fn execute_update_bio(
    deps: DepsMut,
    info: MessageInfo,
    name: String,
    bio: Option<String>,
) -> Result<Response, ContractError> {
    let token_id = name;

    nonpayable(&info)?;
    only_owner(deps.as_ref(), &info.sender, &token_id)?;
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

pub fn execute_update_profile_nft(
    deps: DepsMut,
    info: MessageInfo,
    name: String,
    nft: Option<NFT>,
) -> Result<Response, ContractError> {
    let token_id = name;

    nonpayable(&info)?;
    only_owner(deps.as_ref(), &info.sender, &token_id)?;

    Sg721NameContract::default()
        .tokens
        .update(deps.storage, &token_id, |token| match token {
            Some(mut token_info) => {
                token_info.extension.profile_nft = nft;
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
    let token_id = name;

    nonpayable(&info)?;
    only_owner(deps.as_ref(), &info.sender, &token_id)?;
    validate_and_sanitize_record(&record)?;

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
    only_owner(deps.as_ref(), &info.sender, &token_id)?;

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
    let token_id = name;

    nonpayable(&info)?;
    only_owner(deps.as_ref(), &info.sender, &token_id)?;
    validate_and_sanitize_record(&record)?;

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

fn only_owner(deps: Deps, sender: &Addr, token_id: &str) -> Result<Addr, ContractError> {
    let owner = Sg721NameContract::default()
        .tokens
        .load(deps.storage, token_id)?
        .owner;

    if &owner != sender {
        return Err(ContractError::Base(Unauthorized {}));
    }

    Ok(owner)
}

fn validate_bio(bio: Option<String>) -> Result<(), ContractError> {
    if let Some(bio) = bio {
        if bio.len() > MAX_TEXT_LENGTH as usize {
            return Err(ContractError::BioTooLong {});
        }
    }
    Ok(())
}

fn validate_and_sanitize_record(record: &TextRecord) -> Result<(), ContractError> {
    let name_len = record.name.len();
    if name_len > MAX_TEXT_LENGTH as usize {
        return Err(ContractError::RecordNameTooLong {});
    } else if name_len == 0 {
        return Err(ContractError::RecordNameEmpty {});
    }

    if record.value.len() > MAX_TEXT_LENGTH as usize {
        return Err(ContractError::RecordValueTooLong {});
    }
    Ok(())
}

pub fn query_name_marketplace(deps: Deps) -> StdResult<NameMarketplaceResponse> {
    let address = NAME_MARKETPLACE.load(deps.storage)?;

    Ok(NameMarketplaceResponse {
        address: address.to_string(),
    })
}

pub fn query_name(deps: Deps, mut address: String) -> StdResult<NameResponse> {
    if !address.starts_with("stars") {
        address = transcode(&address)?;
    }

    let name = REVERSE_MAP.load(deps.storage, &deps.api.addr_validate(&address)?)?;

    Ok(NameResponse { name })
}

pub fn transcode(address: &str) -> StdResult<String> {
    let (_, data) =
        bech32::decode(address).map_err(|_| StdError::generic_err("Invalid bech32 address"))?;

    Ok(bech32::encode("stars", data))
}

fn validate_address(deps: Deps, sender: &Addr, addr: &Addr) -> Result<(), ContractError> {
    // we have an EOA registration
    if sender == addr {
        return Ok(());
    }

    let ContractInfoResponse { admin, creator, .. } =
        deps.querier.query_wasm_contract_info(addr)?;

    // If the sender is not the admin or creator, return an error
    if admin.map_or(true, |a| &a != sender) && &creator != sender {
        return Err(ContractError::UnauthorizedCreatorOrAdmin {});
    }

    // we have a contract registration
    Ok(())
}
