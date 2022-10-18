use std::vec;

#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    coin, to_binary, Addr, Coin, DepsMut, Env, MessageInfo, Reply, SubMsg, WasmMsg,
};
use cw2::set_contract_version;
use cw721_base::MintMsg;
use cw_utils::{maybe_addr, must_pay, parse_reply_instantiate_data};
use name_marketplace::msg::ExecuteMsg as MarketplaceExecuteMsg;
use sg721::CollectionInfo;
use sg721_name::{ExecuteMsg as Sg721ExecuteMsg, InstantiateMsg as Sg721InstantiateMsg};
use sg_name::{Metadata, SgNameExecuteMsg};
use sg_std::{create_fund_community_pool_msg, Response, NATIVE_DENOM};
use sg_whitelist_basic::SgWhitelistExecuteMsg;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg};
use crate::state::{SudoParams, ADMIN, NAME_COLLECTION, NAME_MARKETPLACE, SUDO_PARAMS, WHITELISTS};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:name-minter";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

const INIT_COLLECTION_REPLY_ID: u64 = 1;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let admin_addr = maybe_addr(deps.api, msg.admin)?;
    ADMIN.set(deps.branch(), admin_addr)?;

    let api = deps.api;

    let lists = msg
        .whitelists
        .iter()
        .filter_map(|addr| api.addr_validate(addr).ok())
        .collect::<Vec<_>>();

    WHITELISTS.save(deps.storage, &lists)?;

    let marketplace = deps.api.addr_validate(&msg.marketplace_addr)?;
    NAME_MARKETPLACE.save(deps.storage, &marketplace)?;

    let params = SudoParams {
        min_name_length: msg.min_name_length,
        max_name_length: msg.max_name_length,
        base_price: msg.base_price.u128(),
    };
    SUDO_PARAMS.save(deps.storage, &params)?;

    let collection_init_msg = Sg721InstantiateMsg {
        name: "Name Tokens".to_string(),
        symbol: "NAME".to_string(),
        minter: env.contract.address.to_string(),
        collection_info: CollectionInfo {
            creator: info.sender.to_string(),
            description: "Stargaze Names".to_string(),
            image: "ipfs://example.com".to_string(),
            external_link: None,
            explicit_content: false,
            trading_start_time: None,
            royalty_info: None,
        },
    };
    let wasm_msg = WasmMsg::Instantiate {
        code_id: msg.collection_code_id,
        msg: to_binary(&collection_init_msg)?,
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
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    let api = deps.api;

    match msg {
        ExecuteMsg::MintAndList { name } => execute_mint_and_list(deps, info, name.trim()),
        ExecuteMsg::UpdateAdmin { admin } => {
            Ok(ADMIN.execute_update_admin(deps, info, maybe_addr(api, admin)?)?)
        }
        ExecuteMsg::AddWhitelist { address } => execute_add_whitelist(deps, info, address),
        ExecuteMsg::RemoveWhitelist { address } => execute_remove_whitelist(deps, info, address),
    }
}

/// Mint a name for the sender, or `contract` if specified
pub fn execute_mint_and_list(
    deps: DepsMut,
    info: MessageInfo,
    name: &str,
) -> Result<Response, ContractError> {
    let sender = &info.sender.to_string();
    let mut res = Response::new();

    let params = SUDO_PARAMS.load(deps.storage)?;
    let whitelists = WHITELISTS.load(deps.storage)?;

    whitelists.iter().for_each(|whitelist| {
        let msg = WasmMsg::Execute {
            contract_addr: whitelist.to_string(),
            funds: vec![],
            msg: to_binary(&SgWhitelistExecuteMsg::ProcessAddress {
                address: sender.to_string(),
            })
            .unwrap(),
            // TODO: DO NOT unwrap(), throw error
            // TODO: needs to handle case where address is in one list
            // but not in another
        };
        res = res.clone().add_message(msg);
    });

    validate_name(name, params.min_name_length, params.max_name_length)?;

    let price = validate_payment(name.len(), &info, params.base_price)?;
    let community_pool_msg = create_fund_community_pool_msg(vec![price]);

    let collection = NAME_COLLECTION.load(deps.storage)?;
    let marketplace = NAME_MARKETPLACE.load(deps.storage)?;

    let msg = Sg721ExecuteMsg::Mint(MintMsg::<Metadata> {
        token_id: name.to_string(),
        owner: sender.to_string(),
        token_uri: None,
        extension: Metadata::default(),
    });
    let mint_msg_exec = WasmMsg::Execute {
        contract_addr: collection.to_string(),
        msg: to_binary(&msg)?,
        funds: vec![],
    };

    let msg = MarketplaceExecuteMsg::SetAsk {
        token_id: name.to_string(),
        seller: sender.to_string(),
    };
    let list_msg_exec = WasmMsg::Execute {
        contract_addr: marketplace.to_string(),
        msg: to_binary(&msg)?,
        funds: vec![],
    };

    Ok(res
        .add_attribute("action", "mint_and_list")
        .add_message(community_pool_msg)
        .add_message(mint_msg_exec)
        .add_message(list_msg_exec))
}

pub fn execute_add_whitelist(
    deps: DepsMut,
    info: MessageInfo,
    address: String,
) -> Result<Response, ContractError> {
    ADMIN.assert_admin(deps.as_ref(), &info.sender)?;

    let whitelist = deps.api.addr_validate(&address)?;
    let mut lists = WHITELISTS.load(deps.storage)?;
    lists.push(whitelist);

    WHITELISTS.save(deps.storage, &lists)?;

    Ok(Response::new())
}

pub fn execute_remove_whitelist(
    deps: DepsMut,
    info: MessageInfo,
    address: String,
) -> Result<Response, ContractError> {
    ADMIN.assert_admin(deps.as_ref(), &info.sender)?;

    let whitelist = deps.api.addr_validate(&address)?;
    let mut lists = WHITELISTS.load(deps.storage)?;
    lists.retain(|addr| addr != &whitelist);

    WHITELISTS.save(deps.storage, &lists)?;

    Ok(Response::new())
}

// This follows the same rules as Internet domain names
fn validate_name(name: &str, min: u32, max: u32) -> Result<(), ContractError> {
    let len = name.len() as u32;
    if len < min {
        return Err(ContractError::NameTooShort {});
    } else if len >= max {
        return Err(ContractError::NameTooLong {});
    }

    name.find(invalid_char)
        .map_or(Ok(()), |_| Err(ContractError::InvalidName {}))?;

    name.starts_with('-')
        .then(|| Err(ContractError::InvalidName {}))
        .unwrap_or(Ok(()))?;

    name.ends_with('-')
        .then(|| Err(ContractError::InvalidName {}))
        .unwrap_or(Ok(()))?;

    if len > 4 && name[2..4].contains("--") {
        return Err(ContractError::InvalidName {});
    }

    Ok(())
}

fn validate_payment(
    name_len: usize,
    info: &MessageInfo,
    base_price: u128,
) -> Result<Coin, ContractError> {
    // Because we know we are left with ASCII chars, a simple byte count is enough
    let amount = match name_len {
        0..=2 => return Err(ContractError::NameTooShort {}),
        3 => base_price * 100,
        4 => base_price * 10,
        _ => base_price,
    };

    let payment = must_pay(info, NATIVE_DENOM)?;
    if payment.u128() != amount {
        return Err(ContractError::IncorrectPayment {});
    }

    Ok(coin(amount, NATIVE_DENOM))
}

fn invalid_char(c: char) -> bool {
    let is_valid = c.is_digit(10) || c.is_ascii_lowercase() || (c == '-');
    !is_valid
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, _env: Env, msg: Reply) -> Result<Response, ContractError> {
    if msg.id != INIT_COLLECTION_REPLY_ID {
        return Err(ContractError::InvalidReplyID {});
    }

    let reply = parse_reply_instantiate_data(msg);
    match reply {
        Ok(res) => {
            let collection_address = &res.contract_address;

            NAME_COLLECTION.save(deps.storage, &Addr::unchecked(collection_address))?;

            let msg = WasmMsg::Execute {
                contract_addr: collection_address.to_string(),
                funds: vec![],
                msg: to_binary(&SgNameExecuteMsg::SetNameMarketplace {
                    address: NAME_MARKETPLACE.load(deps.storage)?.to_string(),
                })?,
            };

            Ok(Response::default()
                .add_attribute("action", "init_collection_reply")
                .add_message(msg))
        }
        Err(_) => Err(ContractError::ReplyOnSuccess {}),
    }
}

#[cfg(test)]
mod tests {
    use cosmwasm_std::{coin, Addr, MessageInfo};

    use crate::contract::validate_name;

    use super::validate_payment;

    #[test]
    fn check_validate_name() {
        let min = 3;
        let max = 63;
        assert!(validate_name("bobo", min, max).is_ok());
        assert!(validate_name("-bobo", min, max).is_err());
        assert!(validate_name("bobo-", min, max).is_err());
        assert!(validate_name("bo-bo", min, max).is_ok());
        assert!(validate_name("bo--bo", min, max).is_err());
        assert!(validate_name("bob--o", min, max).is_ok());
        assert!(validate_name("bo", min, max).is_err());
        assert!(validate_name("b", min, max).is_err());
        assert!(validate_name("bob", min, max).is_ok());
        assert!(validate_name(
            "bobobobobobobobobobobobobobobobobobobobobobobobobobobobobobobo",
            min,
            max
        )
        .is_ok());
        assert!(validate_name(
            "bobobobobobobobobobobobobobobobobobobobobobobobobobobobobobobob",
            min,
            max
        )
        .is_err());
        assert!(validate_name("0123456789", min, max).is_ok());
        assert!(validate_name("😬", min, max).is_err());
        assert!(validate_name("BOBO", min, max).is_err());
        assert!(validate_name("b-o----b", min, max).is_ok());
        assert!(validate_name("bobo.stars", min, max).is_err());
    }

    #[test]
    fn check_validate_payment() {
        let base_price = 100_000_000;

        let info = MessageInfo {
            sender: Addr::unchecked("sender"),
            funds: vec![coin(base_price, "ustars")],
        };
        assert_eq!(
            validate_payment(5, &info, base_price)
                .unwrap()
                .amount
                .u128(),
            base_price
        );

        let info = MessageInfo {
            sender: Addr::unchecked("sender"),
            funds: vec![coin(base_price * 10, "ustars")],
        };
        assert_eq!(
            validate_payment(4, &info, base_price)
                .unwrap()
                .amount
                .u128(),
            base_price * 10
        );

        let info = MessageInfo {
            sender: Addr::unchecked("sender"),
            funds: vec![coin(base_price * 100, "ustars")],
        };
        assert_eq!(
            validate_payment(3, &info, base_price)
                .unwrap()
                .amount
                .u128(),
            base_price * 100
        );
    }
}
