use crate::error::ContractError;
use crate::msg::SudoMsg;
use crate::state::{ASK_HOOKS, BID_HOOKS, NAME_COLLECTION, NAME_MINTER, SALE_HOOKS, SUDO_PARAMS};
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{Addr, Decimal, DepsMut, Env, Uint128};
use sg_std::Response;

// bps fee can not exceed 100%
const MAX_FEE_BPS: u64 = 10000;

pub struct ParamInfo {
    trading_fee_bps: Option<u64>,
    min_price: Option<Uint128>,
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn sudo(deps: DepsMut, env: Env, msg: SudoMsg) -> Result<Response, ContractError> {
    let api = deps.api;

    match msg {
        SudoMsg::UpdateParams {
            trading_fee_bps,
            min_price,
        } => sudo_update_params(
            deps,
            env,
            ParamInfo {
                trading_fee_bps,
                min_price,
            },
        ),
        SudoMsg::AddSaleHook { hook } => sudo_add_sale_hook(deps, api.addr_validate(&hook)?),
        SudoMsg::AddAskHook { hook } => sudo_add_ask_hook(deps, env, api.addr_validate(&hook)?),
        SudoMsg::AddBidHook { hook } => sudo_add_bid_hook(deps, env, api.addr_validate(&hook)?),
        SudoMsg::RemoveSaleHook { hook } => sudo_remove_sale_hook(deps, api.addr_validate(&hook)?),
        SudoMsg::RemoveAskHook { hook } => sudo_remove_ask_hook(deps, api.addr_validate(&hook)?),
        SudoMsg::RemoveBidHook { hook } => sudo_remove_bid_hook(deps, api.addr_validate(&hook)?),
        SudoMsg::UpdateNameCollection { collection } => {
            sudo_update_name_collection(deps, api.addr_validate(&collection)?)
        }
        SudoMsg::UpdateNameMinter { minter } => {
            sudo_update_name_minter(deps, api.addr_validate(&minter)?)
        }
    }
}

pub fn sudo_update_name_minter(deps: DepsMut, collection: Addr) -> Result<Response, ContractError> {
    NAME_MINTER.save(deps.storage, &collection)?;

    Ok(Response::new().add_attribute("action", "sudo_update_name_minter"))
}

pub fn sudo_update_name_collection(
    deps: DepsMut,
    collection: Addr,
) -> Result<Response, ContractError> {
    NAME_COLLECTION.save(deps.storage, &collection)?;

    Ok(Response::new().add_attribute("action", "sudo_update_name_collection"))
}

/// Only governance can update contract params
pub fn sudo_update_params(
    deps: DepsMut,
    _env: Env,
    param_info: ParamInfo,
) -> Result<Response, ContractError> {
    let ParamInfo {
        trading_fee_bps,
        min_price,
    } = param_info;
    if let Some(trading_fee_bps) = trading_fee_bps {
        if trading_fee_bps > MAX_FEE_BPS {
            return Err(ContractError::InvalidTradingFeeBps(trading_fee_bps));
        }
    }

    let mut params = SUDO_PARAMS.load(deps.storage)?;

    params.trading_fee_percent = trading_fee_bps
        .map(Decimal::percent)
        .unwrap_or(params.trading_fee_percent);

    params.min_price = min_price.unwrap_or(params.min_price);

    SUDO_PARAMS.save(deps.storage, &params)?;

    Ok(Response::new().add_attribute("action", "update_params"))
}

pub fn sudo_add_sale_hook(deps: DepsMut, hook: Addr) -> Result<Response, ContractError> {
    SALE_HOOKS.add_hook(deps.storage, hook.clone())?;

    let res = Response::new()
        .add_attribute("action", "add_sale_hook")
        .add_attribute("hook", hook);
    Ok(res)
}

pub fn sudo_add_ask_hook(deps: DepsMut, _env: Env, hook: Addr) -> Result<Response, ContractError> {
    ASK_HOOKS.add_hook(deps.storage, hook.clone())?;

    let res = Response::new()
        .add_attribute("action", "add_ask_hook")
        .add_attribute("hook", hook);
    Ok(res)
}

pub fn sudo_add_bid_hook(deps: DepsMut, _env: Env, hook: Addr) -> Result<Response, ContractError> {
    BID_HOOKS.add_hook(deps.storage, hook.clone())?;

    let res = Response::new()
        .add_attribute("action", "add_bid_hook")
        .add_attribute("hook", hook);
    Ok(res)
}

pub fn sudo_remove_sale_hook(deps: DepsMut, hook: Addr) -> Result<Response, ContractError> {
    SALE_HOOKS.remove_hook(deps.storage, hook.clone())?;

    let res = Response::new()
        .add_attribute("action", "remove_sale_hook")
        .add_attribute("hook", hook);
    Ok(res)
}

pub fn sudo_remove_ask_hook(deps: DepsMut, hook: Addr) -> Result<Response, ContractError> {
    ASK_HOOKS.remove_hook(deps.storage, hook.clone())?;

    let res = Response::new()
        .add_attribute("action", "remove_ask_hook")
        .add_attribute("hook", hook);
    Ok(res)
}

pub fn sudo_remove_bid_hook(deps: DepsMut, hook: Addr) -> Result<Response, ContractError> {
    BID_HOOKS.remove_hook(deps.storage, hook.clone())?;

    let res = Response::new()
        .add_attribute("action", "remove_bid_hook")
        .add_attribute("hook", hook);
    Ok(res)
}
