#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Addr, Binary, Deps, DepsMut, Env, MessageInfo, Reply, Response, StdResult, SubMsg,
    WasmMsg,
};
use cw2::set_contract_version;
use cw721_base::{
    ExecuteMsg as Cw721ExecuteMsg, Extension, InstantiateMsg as Cw721InstantiateMsg, MintMsg,
};
use cw_utils::parse_reply_instantiate_data;

use crate::error::ContractError;
use crate::msg::{ConfigResponse, ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::COLLECTION_ADDRESS;

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

            COLLECTION_ADDRESS.save(deps.storage, &Addr::unchecked(collection_address))?;

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
        ExecuteMsg::Mint { name } => execute_mint(deps, info, name),
    }
}

pub fn execute_mint(
    deps: DepsMut,
    info: MessageInfo,
    name: String,
) -> Result<Response, ContractError> {
    let mint_msg = Cw721ExecuteMsg::Mint::<Extension, Extension>(MintMsg::<Extension> {
        token_id: name.trim().to_string(),
        owner: info.sender.to_string(),
        token_uri: None,
        extension: None,
    });
    let msg = WasmMsg::Execute {
        contract_addr: COLLECTION_ADDRESS.load(deps.storage)?.to_string(),
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
    let config = COLLECTION_ADDRESS.load(deps.storage)?;
    Ok(ConfigResponse {
        collection_addr: config.to_string(),
    })
}
