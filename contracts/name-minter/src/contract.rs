use std::vec;

#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    coin, to_binary, Addr, Coin, Decimal, DepsMut, Env, Event, MessageInfo, Reply, Uint128, WasmMsg,
};
use cw2::set_contract_version;
use cw721_base::MintMsg;
use cw_utils::{maybe_addr, must_pay, parse_reply_instantiate_data};
use name_marketplace::msg::ExecuteMsg as MarketplaceExecuteMsg;
use sg721::{CollectionInfo, InstantiateMsg as Sg721InstantiateMsg};
use sg721_name::msg::{
    ExecuteMsg as NameCollectionExecuteMsg, InstantiateMsg as NameCollectionInstantiateMsg,
};
use sg_name::{Metadata, SgNameExecuteMsg};
use sg_std::{create_fund_community_pool_msg, Response, SubMsg, NATIVE_DENOM};
use whitelist_updatable::helpers::WhitelistUpdatableContract;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg};
use crate::state::{
    SudoParams, ADMIN, NAME_COLLECTION, NAME_MARKETPLACE, PAUSED, SUDO_PARAMS, WHITELISTS,
};

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
        .map(WhitelistUpdatableContract)
        .collect::<Vec<_>>();

    WHITELISTS.save(deps.storage, &lists)?;

    PAUSED.save(deps.storage, &false)?;

    let marketplace = deps.api.addr_validate(&msg.marketplace_addr)?;
    NAME_MARKETPLACE.save(deps.storage, &marketplace)?;

    let params = SudoParams {
        min_name_length: msg.min_name_length,
        max_name_length: msg.max_name_length,
        base_price: msg.base_price.u128(),
        fair_burn_percent: Decimal::from_ratio(msg.fair_burn_bps, 100u128),
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
            explicit_content: None,
            start_trading_time: None,
            royalty_info: None,
        },
    };
    let name_collection_init_msg = NameCollectionInstantiateMsg {
        verifier: msg.verifier,
        base_init_msg: collection_init_msg,
    };
    let wasm_msg = WasmMsg::Instantiate {
        code_id: msg.collection_code_id,
        msg: to_binary(&name_collection_init_msg)?,
        funds: info.funds,
        admin: Some(info.sender.to_string()),
        label: "Name Collection".to_string(),
    };
    let submsg = SubMsg::reply_on_success(wasm_msg, INIT_COLLECTION_REPLY_ID);

    Ok(Response::new()
        .add_attribute("action", "instantiate")
        .add_submessage(submsg))
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
        ExecuteMsg::Pause { pause } => execute_pause(deps, info, pause),
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
    if PAUSED.load(deps.storage)? {
        return Err(ContractError::MintingPaused {});
    }

    let whitelists = WHITELISTS.load(deps.storage)?;
    let sender = &info.sender.to_string();
    let mut res = Response::new();

    let params = SUDO_PARAMS.load(deps.storage)?;
    validate_name(name, params.min_name_length, params.max_name_length)?;

    // Assumes no duplicate addresses between whitelists
    // Otherwise there will be edge cases with per addr limit between the whitelists
    let list = whitelists.iter().find(|whitelist| {
        whitelist
            .includes(&deps.querier, sender.to_string())
            .unwrap_or(false)
    });

    if !whitelists.is_empty() && list.is_none() {
        return Err(ContractError::NotWhitelisted {});
    }

    let discount = if let Some(list) = list {
        res = res.add_message(list.process_address(sender)?);
        list.config(&deps.querier).map(|c| c.mint_discount())?
    } else {
        None
    };

    let price = validate_payment(name.len(), &info, params.base_price, discount)?;
    let community_pool_msg = create_fund_community_pool_msg(vec![price.clone()]);

    let collection = NAME_COLLECTION.load(deps.storage)?;
    let marketplace = NAME_MARKETPLACE.load(deps.storage)?;

    let mint_msg = NameCollectionExecuteMsg::Mint(MintMsg::<Metadata> {
        token_id: name.to_string(),
        owner: sender.to_string(),
        token_uri: None,
        extension: Metadata::default(),
    });
    let mint_msg_exec = WasmMsg::Execute {
        contract_addr: collection.to_string(),
        msg: to_binary(&mint_msg)?,
        funds: vec![],
    };

    let ask_msg = MarketplaceExecuteMsg::SetAsk {
        token_id: name.to_string(),
        seller: sender.to_string(),
    };
    let list_msg_exec = WasmMsg::Execute {
        contract_addr: marketplace.to_string(),
        msg: to_binary(&ask_msg)?,
        funds: vec![],
    };

    let event = Event::new("mint_and_list")
        .add_attribute("name", name)
        .add_attribute("owner", sender)
        .add_attribute("price", price.amount.to_string());
    Ok(res
        .add_event(event)
        .add_message(community_pool_msg)
        .add_message(mint_msg_exec)
        .add_message(list_msg_exec))
}

/// Pause or unpause minting
pub fn execute_pause(
    deps: DepsMut,
    info: MessageInfo,
    pause: bool,
) -> Result<Response, ContractError> {
    ADMIN.assert_admin(deps.as_ref(), &info.sender)?;

    PAUSED.save(deps.storage, &pause)?;

    let event = Event::new("pause").add_attribute("pause", pause.to_string());
    Ok(Response::new().add_event(event))
}

pub fn execute_add_whitelist(
    deps: DepsMut,
    info: MessageInfo,
    address: String,
) -> Result<Response, ContractError> {
    ADMIN.assert_admin(deps.as_ref(), &info.sender)?;

    let whitelist = deps
        .api
        .addr_validate(&address)
        .map(WhitelistUpdatableContract)?;
    let mut lists = WHITELISTS.load(deps.storage)?;
    lists.push(whitelist);

    WHITELISTS.save(deps.storage, &lists)?;

    let event = Event::new("add_whitelist").add_attribute("address", address);
    Ok(Response::new().add_event(event))
}

pub fn execute_remove_whitelist(
    deps: DepsMut,
    info: MessageInfo,
    address: String,
) -> Result<Response, ContractError> {
    ADMIN.assert_admin(deps.as_ref(), &info.sender)?;

    let whitelist = deps.api.addr_validate(&address)?;
    let mut lists = WHITELISTS.load(deps.storage)?;
    lists.retain(|addr| addr.addr() != whitelist);

    WHITELISTS.save(deps.storage, &lists)?;

    let event = Event::new("remove_whitelist").add_attribute("address", address);
    Ok(Response::new().add_event(event))
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

    if name.starts_with('-') || name.ends_with('-') {
        Err(ContractError::InvalidName {})
    } else {
        Ok(())
    }?;

    if len > 4 && name[2..4].contains("--") {
        return Err(ContractError::InvalidName {});
    }

    Ok(())
}

fn validate_payment(
    name_len: usize,
    info: &MessageInfo,
    base_price: u128,
    discount: Option<Decimal>,
) -> Result<Coin, ContractError> {
    // Because we know we are left with ASCII chars, a simple byte count is enough
    let amount: Uint128 = match name_len {
        0..=2 => return Err(ContractError::NameTooShort {}),
        3 => base_price * 100,
        4 => base_price * 10,
        _ => base_price,
    }
    .into();

    let amount = discount
        .map(|d| amount * (Decimal::one() - d))
        .unwrap_or(amount);

    let payment = must_pay(info, NATIVE_DENOM)?;
    if payment != amount {
        return Err(ContractError::IncorrectPayment {
            got: amount.u128(),
            expected: payment.u128(),
        });
    }

    Ok(coin(amount.u128(), NATIVE_DENOM))
}

fn invalid_char(c: char) -> bool {
    let is_valid = c.is_ascii_digit() || c.is_ascii_lowercase() || (c == '-');
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
    use cosmwasm_std::{coin, Addr, Decimal, MessageInfo};

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
            validate_payment(5, &info, base_price, None)
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
            validate_payment(4, &info, base_price, None)
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
            validate_payment(3, &info, base_price, None)
                .unwrap()
                .amount
                .u128(),
            base_price * 100
        );
    }

    #[test]
    fn check_validate_payment_with_discount() {
        let base_price = 100_000_000;

        let info = MessageInfo {
            sender: Addr::unchecked("sender"),
            funds: vec![coin(base_price / 2, "ustars")],
        };
        assert_eq!(
            validate_payment(5, &info, base_price, Some(Decimal::percent(50)))
                .unwrap()
                .amount
                .u128(),
            base_price / 2
        );
    }
}
