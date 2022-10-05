#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    coin, to_binary, Addr, Binary, Coin, CosmosMsg, Deps, DepsMut, Env, MessageInfo, Reply,
    Response, StdResult, SubMsg, WasmMsg,
};
use cw2::set_contract_version;
use cw721_base::{
    ExecuteMsg as Cw721ExecuteMsg, Extension, InstantiateMsg as Cw721InstantiateMsg, MintMsg,
};
use cw_utils::{must_pay, parse_reply_instantiate_data};
use sg_name::Metadata;
use unicode_segmentation::UnicodeSegmentation;

use crate::error::ContractError;
use crate::msg::{ConfigResponse, ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::NAME_COLLECTION;

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:name-minter";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

const INIT_COLLECTION_REPLY_ID: u64 = 1;

const MIN_NAME_LENGTH: u64 = 3;
const MAX_NAME_LENGTH: u64 = 63;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let wasm_msg = WasmMsg::Instantiate {
        code_id: msg.collection_code_id,
        msg: to_binary(&Cw721InstantiateMsg {
            name: "Name Tokens".to_string(),
            symbol: "NAME".to_string(),
            minter: info.sender.to_string(),
        })?,
        funds: info.funds,
        admin: None,
        label: "Name Collection".to_string(),
    };
    let submsg = SubMsg::reply_on_success(wasm_msg, INIT_COLLECTION_REPLY_ID);

    Ok(Response::new()
        .add_attribute("action", "instantiate")
        .add_submessage(submsg)
        .add_attribute("owner", info.sender))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, _env: Env, msg: Reply) -> Result<Response, ContractError> {
    if msg.id != INIT_COLLECTION_REPLY_ID {
        return Err(ContractError::InvalidReplyID {});
    }

    let reply = parse_reply_instantiate_data(msg);
    match reply {
        Ok(res) => {
            let collection_address = res.contract_address;

            NAME_COLLECTION.save(deps.storage, &Addr::unchecked(collection_address))?;

            Ok(Response::default().add_attribute("action", "init_collection_reply"))
        }
        Err(_) => Err(ContractError::ReplyOnSuccess {}),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::MintAndList { name } => execute_mint_and_list(deps, info, name.trim()),
        ExecuteMsg::UpdateBio { name, bio } => todo!(),
        ExecuteMsg::UpdateProfile { name, profile } => todo!(),
        ExecuteMsg::AddTextRecord { name, record } => todo!(),
        ExecuteMsg::RemoveTextRecord { name, record_name } => todo!(),
        ExecuteMsg::UpdateTextRecord { name, record } => todo!(),
    }
}

pub fn execute_mint_and_list(
    deps: DepsMut,
    info: MessageInfo,
    name: &str,
) -> Result<Response, ContractError> {
    // TODO: add to governance
    let BASE_PRICE = 100000000u128;

    if !name.is_ascii() {
        return Err(ContractError::InvalidName {});
    }

    // Because we know we are left with ASCII chars, a simple byte count is enough
    let amount = match name.len() {
        0..2 => return Err(ContractError::NameTooShort {}),
        3 => BASE_PRICE * 100,
        4 => BASE_PRICE * 10,
        5..63 => BASE_PRICE,
        _ => return Err(ContractError::NameTooLong {}),
    };
    let price = coin(amount, "ustars");

    let payment = must_pay(&info, "ustars")?;
    if payment != amount {
        return Err(ContractError::InvalidPayment {});
    }
    let msg = CosmosMsg::Distribution(DistributionMsg::FundCommunityPool {
        amount: payment,
        denom: "ustars",
    });

    let mint_msg = MintMsg::<Metadata<Extension>> {
        token_id: name.trim().to_string(),
        owner: info.sender.to_string(),
        token_uri: None,
        extension: None,
    };

    let msg = WasmMsg::Execute {
        contract_addr: NAME_COLLECTION.load(deps.storage)?.to_string(),
        msg: to_binary(&mint_msg)?,
        funds: vec![],
    };

    // TODO: list on name marketplace

    Ok(Response::new()
        .add_attribute("action", "mint")
        .add_message(msg))
}

// We store the name without the TLD so it can be mapped to a raw address
// that is not bech32 encoded. This way, all Cosmos / Interchain names can
// be resolved to an address that is derived via the same (118) derivation
// path.
//
// For example:
//
// bobo -> D93385094E906D7DA4EBFDEC2C4B167D5CAA431A (in hex)
//
// Now this can be resolved per chain:
//
// bobo.stars -> stars1myec2z2wjpkhmf8tlhkzcjck04w25sc6ymhplz
// bobo.cosmos -> cosmos1myec2z2wjpkhmf8tlhkzcjck04w25sc6y2xq2r
fn validate_name(name: &str) -> Result<(), ContractError> {
    let len = name.len() as u64;
    if len < MIN_NAME_LENGTH {
        return Err(ContractError::NameTooShort {});
    } else if len > MAX_NAME_LENGTH {
        return Err(ContractError::NameTooLong {});
    }

    name.find(invalid_char)
        .map_or(Ok(()), |_| Err(ContractError::InvalidName {}));

    name.starts_with("-")
        .then(|| Err(ContractError::InvalidName {}));

    name.ends_with("-")
        .then(|| Err(ContractError::InvalidName {}));

    if len > 4 {
        name[2..]
            .find("--")
            .map_or(Ok(()), |_| Err(ContractError::InvalidName {}))
    }

    Ok(())
}

fn invalid_char(c: char) -> bool {
    let is_valid = c.is_digit(10) || c.is_ascii_lowercase() || (c == '-');
    !is_valid
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query_collection_addr(deps)?),
    }
}

fn query_collection_addr(deps: Deps) -> StdResult<ConfigResponse> {
    let config = NAME_COLLECTION.load(deps.storage)?;
    Ok(ConfigResponse {
        collection_addr: config.to_string(),
    })
}
