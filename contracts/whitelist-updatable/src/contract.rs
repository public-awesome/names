use crate::state::{Config, CONFIG, TOTAL_ADDRESS_COUNT, WHITELIST};
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Addr, Binary, Decimal, Deps, DepsMut, Env, Event, MessageInfo, Order, StdResult,
};
use cw2::set_contract_version;
use sg_name_minter::SgNameMinterQueryMsg;

use crate::error::ContractError;
use crate::msg::{ConfigResponse, ExecuteMsg, InstantiateMsg, QueryMsg};
use cw_utils::nonpayable;
use sg_std::Response;

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:whitelist-updatable";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    mut msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    nonpayable(&info)?;
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    let config = Config {
        admin: info.sender,
        per_address_limit: msg.per_address_limit,
        /// 1% = 100, 50% = 5000
        mint_discount_bps: msg.mint_discount_bps,
    };

    // remove duplicate addresses
    msg.addresses.sort_unstable();
    msg.addresses.dedup();

    let mut count = 0u64;
    for address in msg.addresses.into_iter() {
        let addr = deps.api.addr_validate(&address.clone())?;
        WHITELIST.save(deps.storage, addr, &0u32)?;
        count += 1;
    }

    TOTAL_ADDRESS_COUNT.save(deps.storage, &count)?;
    CONFIG.save(deps.storage, &config)?;

    Ok(Response::default()
        .add_attribute("action", "instantiate")
        .add_attribute("whitelist_addr", env.contract.address.to_string()))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::UpdateAdmin { new_admin } => execute_update_admin(deps, info, new_admin),
        ExecuteMsg::AddAddresses { addresses } => execute_add_addresses(deps, info, addresses),
        ExecuteMsg::RemoveAddresses { addresses } => {
            execute_remove_addresses(deps, info, addresses)
        }
        ExecuteMsg::ProcessAddress { address } => execute_process_address(deps, env, info, address),
        ExecuteMsg::UpdatePerAddressLimit { limit } => {
            execute_update_per_address_limit(deps, info, limit)
        }
        ExecuteMsg::Purge {} => execute_purge(deps, info),
    }
}

pub fn execute_update_admin(
    deps: DepsMut,
    info: MessageInfo,
    new_admin: String,
) -> Result<Response, ContractError> {
    nonpayable(&info)?;
    let mut config = CONFIG.load(deps.storage)?;
    if config.admin != info.sender {
        return Err(ContractError::Unauthorized {});
    }

    config.admin = deps.api.addr_validate(&new_admin)?;
    CONFIG.save(deps.storage, &config)?;
    let event = Event::new("update-admin")
        .add_attribute("new_admin", config.admin)
        .add_attribute("sender", info.sender);
    Ok(Response::new().add_event(event))
}

pub fn execute_add_addresses(
    deps: DepsMut,
    info: MessageInfo,
    mut addresses: Vec<String>,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let mut count = TOTAL_ADDRESS_COUNT.load(deps.storage)?;
    if config.admin != info.sender {
        return Err(ContractError::Unauthorized {});
    }

    // dedupe
    addresses.sort_unstable();
    addresses.dedup();

    for address in addresses.into_iter() {
        let addr = deps.api.addr_validate(&address.clone())?;
        if WHITELIST.has(deps.storage, addr.clone()) {
            return Err(ContractError::AddressAlreadyExists {
                addr: addr.to_string(),
            });
        } else {
            WHITELIST.save(deps.storage, addr, &0u32)?;
            count += 1;
        }
    }

    TOTAL_ADDRESS_COUNT.save(deps.storage, &count)?;

    let event = Event::new("add-addresses")
        .add_attribute("new-count", count.to_string())
        .add_attribute("sender", info.sender);
    Ok(Response::new().add_event(event))
}

pub fn execute_remove_addresses(
    deps: DepsMut,
    info: MessageInfo,
    mut addresses: Vec<String>,
) -> Result<Response, ContractError> {
    nonpayable(&info)?;
    let config = CONFIG.load(deps.storage)?;
    let mut count = TOTAL_ADDRESS_COUNT.load(deps.storage)?;
    if config.admin != info.sender {
        return Err(ContractError::Unauthorized {});
    }

    // dedupe
    addresses.sort_unstable();
    addresses.dedup();

    for address in addresses.into_iter() {
        let addr = deps.api.addr_validate(&address.clone())?;
        if WHITELIST.has(deps.storage, addr.clone()) {
            WHITELIST.remove(deps.storage, addr);
            count -= 1;
        } else {
            return Err(ContractError::AddressNotFound {
                addr: addr.to_string(),
            });
        }
    }

    TOTAL_ADDRESS_COUNT.save(deps.storage, &count)?;
    let event = Event::new("remove-addresses")
        .add_attribute("new-count", count.to_string())
        .add_attribute("sender", info.sender);
    Ok(Response::new().add_event(event))
}

pub fn execute_process_address(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    address: String,
) -> Result<Response, ContractError> {
    nonpayable(&info)?;
    let config = CONFIG.load(deps.storage)?;
    let minter = info.sender;

    // query whitelists from minter to see if this one exists...
    let whitelists: Vec<Addr> = deps
        .querier
        .query_wasm_smart(&minter, &SgNameMinterQueryMsg::Whitelists {})?;
    if !whitelists.contains(&env.contract.address) {
        return Err(ContractError::Unauthorized {});
    }

    let addr = deps.api.addr_validate(&address)?;
    if !WHITELIST.has(deps.storage, addr.clone()) {
        return Err(ContractError::AddressNotFound {
            addr: addr.to_string(),
        });
    }

    if WHITELIST.load(deps.storage, addr.clone())? >= config.per_address_limit {
        return Err(ContractError::OverPerAddressLimit {});
    }

    let count = WHITELIST.load(deps.storage, addr.clone())?;
    WHITELIST.save(deps.storage, addr, &(count + 1))?;

    let event = Event::new("process-address")
        .add_attribute("address", address)
        .add_attribute("mint-count", (count + 1).to_string())
        .add_attribute("sender", minter);
    Ok(Response::new().add_event(event))
}

pub fn execute_update_per_address_limit(
    deps: DepsMut,
    info: MessageInfo,
    limit: u32,
) -> Result<Response, ContractError> {
    nonpayable(&info)?;
    let mut config = CONFIG.load(deps.storage)?;
    if config.admin != info.sender {
        return Err(ContractError::Unauthorized {});
    }

    config.per_address_limit = limit;
    CONFIG.save(deps.storage, &config)?;

    let event = Event::new("update-per-address-limit")
        .add_attribute("new-limit", limit.to_string())
        .add_attribute("sender", info.sender);
    Ok(Response::new().add_event(event))
}

pub fn execute_purge(deps: DepsMut, info: MessageInfo) -> Result<Response, ContractError> {
    nonpayable(&info)?;
    let config = CONFIG.load(deps.storage)?;
    if config.admin != info.sender {
        return Err(ContractError::Unauthorized {});
    }

    let keys = WHITELIST
        .keys(deps.as_ref().storage, None, None, Order::Ascending)
        .map(|x| x.unwrap())
        .collect::<Vec<_>>();

    for key in keys {
        WHITELIST.remove(deps.storage, key);
    }

    TOTAL_ADDRESS_COUNT.save(deps.storage, &0u64)?;

    let event = Event::new("purge").add_attribute("sender", info.sender);
    Ok(Response::new().add_event(event))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query_config(deps)?),
        QueryMsg::IncludesAddress { address } => to_binary(&query_includes_address(deps, address)?),
        QueryMsg::MintCount { address } => to_binary(&query_mint_count(deps, address)?),
        QueryMsg::Admin {} => to_binary(&query_admin(deps)?),
        QueryMsg::AddressCount {} => to_binary(&query_address_count(deps)?),
        QueryMsg::PerAddressLimit {} => to_binary(&query_per_address_limit(deps)?),
        QueryMsg::IsProcessable { address } => to_binary(&query_is_processable(deps, address)?),
        QueryMsg::MintDiscountPercent {} => to_binary(&query_mint_discount_percent(deps)?),
    }
}

pub fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config = CONFIG.load(deps.storage)?;
    Ok(ConfigResponse { config })
}

pub fn query_includes_address(deps: Deps, address: String) -> StdResult<bool> {
    Ok(WHITELIST.has(deps.storage, Addr::unchecked(address)))
}

pub fn query_mint_count(deps: Deps, address: String) -> StdResult<u32> {
    let addr = deps.api.addr_validate(&address)?;
    WHITELIST.load(deps.storage, addr)
}

pub fn query_admin(deps: Deps) -> StdResult<String> {
    let config = CONFIG.load(deps.storage)?;
    Ok(config.admin.to_string())
}

pub fn query_address_count(deps: Deps) -> StdResult<u64> {
    TOTAL_ADDRESS_COUNT.load(deps.storage)
}

pub fn query_per_address_limit(deps: Deps) -> StdResult<u32> {
    let config = CONFIG.load(deps.storage)?;
    Ok(config.per_address_limit)
}

pub fn query_is_processable(deps: Deps, address: String) -> StdResult<bool> {
    let addr = deps.api.addr_validate(&address)?;
    // address not in whitelist, it's not processable
    if !WHITELIST.has(deps.storage, addr.clone()) {
        return Ok(false);
    }
    // compare addr mint count to per address limit
    let count = WHITELIST.load(deps.storage, addr)?;
    let config = CONFIG.load(deps.storage)?;
    Ok(count < config.per_address_limit)
}

pub fn query_mint_discount_percent(deps: Deps) -> StdResult<Option<Decimal>> {
    let config = CONFIG.load(deps.storage)?;
    Ok(config
        .mint_discount_bps
        .map(|x| Decimal::from_ratio(x, 10_000u128)))
}
