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

    let count = name.graphemes(true).count();
    let amount = match count {
        1 => return Err(ContractError::NameTooShort {}),
        2 => return Err(ContractError::NameTooShort {}),
        3 => BASE_PRICE * 100,
        4 => BASE_PRICE * 10,
        5..255 => BASE_PRICE,
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
