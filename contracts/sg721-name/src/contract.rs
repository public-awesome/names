use crate::{
    error::ContractError,
    msg::ParamsResponse,
    state::{NAME_MARKETPLACE, REVERSE_MAP, SUDO_PARAMS, VERIFIER},
    CONTRACT_NAME, CONTRACT_VERSION,
};

use cosmwasm_std::{
    to_binary, Addr, Binary, ContractInfoResponse, Deps, DepsMut, Empty, Env, Event, MessageInfo,
    StdError, StdResult, WasmMsg,
};

use cw2::set_contract_version;
use cw721_base::{state::TokenInfo, MintMsg};
use cw_utils::nonpayable;
use semver::Version;

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

    let mut event = Event::new("update-metadata")
        .add_attribute("token_id", token_id.clone())
        .add_attribute("owner", info.sender);

    let mut token_info = Sg721NameContract::default()
        .tokens
        .load(deps.storage, &token_id)?;

    if let Some(metadata) = metadata {
        let max_records = SUDO_PARAMS.load(deps.storage)?.max_record_count;
        if metadata.records.len() > max_records as usize {
            return Err(ContractError::TooManyRecords { max: max_records });
        }

        event = event.add_attribute("metadata", metadata.into_json_string());

        if let Some(image_nft) = metadata.image_nft {
            token_info.extension.image_nft = Some(image_nft);
        }

        if !metadata.records.is_empty() {
            for record in metadata.records.iter() {
                let mut updated_record = record.clone();
                updated_record.verified = None;

                validate_record(&updated_record)?;

                token_info
                    .extension
                    .records
                    .retain(|r| r.name != updated_record.name);
                token_info.extension.records.push(updated_record.clone());
            }
        };
    } else {
        token_info.extension = Metadata::default();
    };

    Sg721NameContract::default()
        .tokens
        .save(deps.storage, &token_id, &token_info)?;

    Ok(Response::new().add_event(event))
}

pub fn execute_associate_address(
    deps: DepsMut,
    info: MessageInfo,
    name: String,
    address: Option<String>,
) -> Result<Response, ContractError> {
    only_owner(deps.as_ref(), &info.sender, &name)?;

    // 1. remove old token_uri from reverse map if it exists
    Sg721NameContract::default()
        .tokens
        .load(deps.storage, &name)
        .map(|prev_token_info| {
            if let Some(address) = prev_token_info.token_uri {
                REVERSE_MAP.remove(deps.storage, &Addr::unchecked(address));
            }
        })?;

    // 2. validate the new address
    let token_uri = address
        .clone()
        .map(|address| {
            deps.api
                .addr_validate(&address)
                .map(|addr| validate_address(deps.as_ref(), &info.sender, addr))?
        })
        .transpose()?;

    // 3. look up prev name if it exists for the new address
    let old_name = token_uri
        .clone()
        .and_then(|addr| REVERSE_MAP.may_load(deps.storage, &addr).unwrap_or(None));

    // 4. remove old token_uri / address from previous name
    old_name.map(|token_id| {
        Sg721NameContract::default()
            .tokens
            .update(deps.storage, &token_id, |token| match token {
                Some(mut token_info) => {
                    token_info.token_uri = None;
                    Ok(token_info)
                }
                None => Err(ContractError::NameNotFound {}),
            })
    });

    // 5. associate new token_uri / address with new name / token_id
    Sg721NameContract::default()
        .tokens
        .update(deps.storage, &name, |token| match token {
            Some(mut token_info) => {
                token_info.token_uri = token_uri.clone().map(|addr| addr.to_string());
                Ok(token_info)
            }
            None => Err(ContractError::NameNotFound {}),
        })?;

    // 6. save new reverse map entry
    token_uri.map(|addr| REVERSE_MAP.save(deps.storage, &addr, &name));

    let event = Event::new("associate-address")
        .add_attribute("name", name)
        .add_attribute("owner", info.sender);
    address.map(|addr| event.clone().add_attribute("address", addr));

    Ok(Response::new().add_event(event))
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

    let event = Event::new("mint")
        .add_attribute("minter", info.sender)
        .add_attribute("token_id", msg.token_id)
        .add_attribute("owner", msg.owner);
    Ok(Response::new().add_event(event))
}

/// WIP Throw not implemented error
pub fn execute_burn(
    _deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    _token_id: String,
) -> Result<Response, ContractError> {
    nonpayable(&info)?;
    Err(ContractError::NotImplemented {})
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

    let update_ask_msg = _transfer_nft(deps, env, &info, &recipient, &token_id)?;

    let event = Event::new("transfer")
        .add_attribute("sender", info.sender)
        .add_attribute("recipient", recipient)
        .add_attribute("token_id", token_id);

    Ok(Response::new().add_message(update_ask_msg).add_event(event))
}

// Update the ask on the marketplace
fn update_ask_on_marketplace(
    deps: Deps,
    token_id: &str,
    recipient: Addr,
) -> Result<WasmMsg, ContractError> {
    let msg = SgNameMarketplaceExecuteMsg::UpdateAsk {
        token_id: token_id.to_string(),
        seller: recipient.to_string(),
    };
    let update_ask_msg = WasmMsg::Execute {
        contract_addr: NAME_MARKETPLACE.load(deps.storage)?.to_string(),
        funds: vec![],
        msg: to_binary(&msg)?,
    };
    Ok(update_ask_msg)
}

fn reset_token_metadata_and_reverse_map(deps: &mut DepsMut, token_id: &str) -> StdResult<()> {
    let mut token = Sg721NameContract::default()
        .tokens
        .load(deps.storage, token_id)?;

    // Reset image, records
    token.extension = Metadata::default();
    Sg721NameContract::default()
        .tokens
        .save(deps.storage, token_id, &token)?;

    remove_reverse_mapping(deps, token_id)?;

    Ok(())
}

fn remove_reverse_mapping(deps: &mut DepsMut, token_id: &str) -> StdResult<()> {
    let mut token = Sg721NameContract::default()
        .tokens
        .load(deps.storage, token_id)?;

    // remove reverse mapping if exists
    if let Some(token_uri) = token.token_uri {
        REVERSE_MAP.remove(deps.storage, &Addr::unchecked(token_uri));
        token.token_uri = None;
    }

    Sg721NameContract::default()
        .tokens
        .save(deps.storage, token_id, &token)?;

    Ok(())
}

fn _transfer_nft(
    mut deps: DepsMut,
    env: Env,
    info: &MessageInfo,
    recipient: &Addr,
    token_id: &str,
) -> Result<WasmMsg, ContractError> {
    let update_ask_msg = update_ask_on_marketplace(deps.as_ref(), token_id, recipient.clone())?;

    reset_token_metadata_and_reverse_map(&mut deps, token_id)?;

    let msg = Sg721ExecuteMsg::TransferNft {
        recipient: recipient.to_string(),
        token_id: token_id.to_string(),
    };

    Sg721NameContract::default().execute(deps, env, info.clone(), msg)?;

    Ok(update_ask_msg)
}

pub fn execute_send_nft(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
    contract: String,
    token_id: String,
    msg: Binary,
) -> Result<Response, ContractError> {
    let contract_addr = deps.api.addr_validate(&contract)?;
    let update_ask_msg =
        update_ask_on_marketplace(deps.as_ref(), &token_id, contract_addr.clone())?;

    reset_token_metadata_and_reverse_map(&mut deps, &token_id)?;

    let msg = Sg721ExecuteMsg::SendNft {
        contract: contract_addr.to_string(),
        token_id: token_id.to_string(),
        msg,
    };

    Sg721NameContract::default().execute(deps, env, info.clone(), msg)?;

    let event = Event::new("send")
        .add_attribute("sender", info.sender)
        .add_attribute("contract", contract_addr.to_string())
        .add_attribute("token_id", token_id);

    Ok(Response::new().add_message(update_ask_msg).add_event(event))
}

pub fn execute_update_image_nft(
    deps: DepsMut,
    info: MessageInfo,
    name: String,
    nft: Option<NFT>,
) -> Result<Response, ContractError> {
    let token_id = name.clone();

    nonpayable(&info)?;
    only_owner(deps.as_ref(), &info.sender, &token_id)?;

    let mut event = Event::new("update_image_nft")
        .add_attribute("owner", info.sender.to_string())
        .add_attribute("token_id", name);

    Sg721NameContract::default()
        .tokens
        .update(deps.storage, &token_id, |token| match token {
            Some(mut token_info) => {
                token_info.extension.image_nft = nft.clone();
                Ok(token_info)
            }
            None => Err(ContractError::NameNotFound {}),
        })?;

    if let Some(nft) = nft {
        event = event.add_attribute("image_nft", nft.into_json_string());
    }

    Ok(Response::new().add_event(event))
}

pub fn execute_add_text_record(
    deps: DepsMut,
    info: MessageInfo,
    name: String,
    mut record: TextRecord,
) -> Result<Response, ContractError> {
    let token_id = name;
    let params = SUDO_PARAMS.load(deps.storage)?;
    let max_record_count = params.max_record_count;

    // new records should reset verified to None
    record.verified = None;

    nonpayable(&info)?;
    only_owner(deps.as_ref(), &info.sender, &token_id)?;
    validate_record(&record)?;

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
                token_info.extension.records.push(record.clone());
                // check record length
                if token_info.extension.records.len() > max_record_count as usize {
                    return Err(ContractError::TooManyRecords {
                        max: max_record_count,
                    });
                }
                Ok(token_info)
            }
            None => Err(ContractError::NameNotFound {}),
        })?;

    let event = Event::new("add-text-record")
        .add_attribute("sender", info.sender)
        .add_attribute("name", token_id)
        .add_attribute("record", record.into_json_string());
    Ok(Response::new().add_event(event))
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

    let event = Event::new("remove-text-record")
        .add_attribute("sender", info.sender)
        .add_attribute("name", token_id)
        .add_attribute("record_name", record_name);
    Ok(Response::new().add_event(event))
}

pub fn execute_update_text_record(
    deps: DepsMut,
    info: MessageInfo,
    name: String,
    mut record: TextRecord,
) -> Result<Response, ContractError> {
    let token_id = name;

    // updated records should reset verified to None
    record.verified = None;

    nonpayable(&info)?;
    only_owner(deps.as_ref(), &info.sender, &token_id)?;
    validate_record(&record)?;

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

    let event = Event::new("update-text-record")
        .add_attribute("sender", info.sender)
        .add_attribute("name", token_id)
        .add_attribute("record", record.into_json_string());
    Ok(Response::new().add_event(event))
}

pub fn execute_verify_text_record(
    deps: DepsMut,
    info: MessageInfo,
    name: String,
    record_name: String,
    result: bool,
) -> Result<Response, ContractError> {
    nonpayable(&info)?;
    VERIFIER.assert_admin(deps.as_ref(), &info.sender)?;

    let token_id = name;

    Sg721NameContract::default()
        .tokens
        .update(deps.storage, &token_id, |token| match token {
            Some(mut token_info) => {
                if let Some(r) = token_info
                    .extension
                    .records
                    .iter_mut()
                    .find(|r| r.name == record_name)
                {
                    r.verified = Some(result);
                }
                Ok(token_info)
            }
            None => Err(ContractError::NameNotFound {}),
        })?;

    let event = Event::new("verify-text-record")
        .add_attribute("sender", info.sender)
        .add_attribute("name", token_id)
        .add_attribute("record", record_name)
        .add_attribute("result", result.to_string());
    Ok(Response::new().add_event(event))
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

    let event = Event::new("set-name-marketplace")
        .add_attribute("sender", info.sender)
        .add_attribute("address", address);
    Ok(Response::new().add_event(event))
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

fn validate_record(record: &TextRecord) -> Result<(), ContractError> {
    if record.verified.is_some() {
        return Err(ContractError::UnauthorizedVerification {});
    }
    let name_len = record.name.len();
    if name_len > MAX_TEXT_LENGTH as usize {
        return Err(ContractError::RecordNameTooLong {});
    } else if name_len == 0 {
        return Err(ContractError::RecordNameEmpty {});
    }

    if record.value.len() > MAX_TEXT_LENGTH as usize {
        return Err(ContractError::RecordValueTooLong {});
    } else if record.value.is_empty() {
        return Err(ContractError::RecordValueEmpty {});
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

pub fn query_params(deps: Deps) -> StdResult<ParamsResponse> {
    let params = SUDO_PARAMS.load(deps.storage)?;
    Ok(ParamsResponse {
        max_record_count: params.max_record_count,
    })
}

pub fn transcode(address: &str) -> StdResult<String> {
    let (_, data) =
        bech32::decode(address).map_err(|_| StdError::generic_err("Invalid bech32 address"))?;

    Ok(bech32::encode("stars", data))
}

fn validate_address(deps: Deps, sender: &Addr, addr: Addr) -> Result<Addr, ContractError> {
    // we have an EOA registration
    if sender == &addr {
        return Ok(addr);
    }

    let ContractInfoResponse { admin, creator, .. } =
        deps.querier.query_wasm_contract_info(&addr)?;

    // If the sender is not the admin or creator, return an error
    if admin.map_or(true, |a| &a != sender) && &creator != sender {
        return Err(ContractError::UnauthorizedCreatorOrAdmin {});
    }

    // we have a contract registration
    Ok(addr)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, _msg: Empty) -> Result<Response, ContractError> {
    let current_version = cw2::get_contract_version(deps.storage)?;
    if current_version.contract != CONTRACT_NAME {
        return Err(StdError::generic_err("Cannot upgrade to a different contract").into());
    }
    let version: Version = current_version
        .version
        .parse()
        .map_err(|_| StdError::generic_err("Invalid contract version"))?;
    let new_version: Version = CONTRACT_VERSION
        .parse()
        .map_err(|_| StdError::generic_err("Invalid contract version"))?;

    if version > new_version {
        return Err(StdError::generic_err("Cannot upgrade to a previous contract version").into());
    }
    // if same version return
    if version == new_version {
        return Ok(Response::new());
    }

    // set new contract version
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    Ok(Response::new())
}
