use std::vec;

#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    coin, to_json_binary, Addr, Coin, Decimal, DepsMut, Empty, Env, Event, MessageInfo, Reply,
    StdError, Uint128, WasmMsg,
};
use cw2::set_contract_version;
use cw721_base::MintMsg;
use cw_utils::{maybe_addr, must_pay, parse_reply_instantiate_data};
use name_marketplace::msg::ExecuteMsg as MarketplaceExecuteMsg;

use semver::Version;

use sg721::{CollectionInfo, InstantiateMsg as Sg721InstantiateMsg};
use sg721_name::msg::{
    ExecuteMsg as NameCollectionExecuteMsg, InstantiateMsg as NameCollectionInstantiateMsg,
};
use sg_name::{Metadata, SgNameExecuteMsg};
use sg_name_common::{charge_fees, SECONDS_PER_YEAR};
use sg_name_minter::{Config, SudoParams, PUBLIC_MINT_START_TIME_IN_SECONDS};
use sg_std::{Response, SubMsg, NATIVE_DENOM};
use whitelist_updatable::helpers::WhitelistUpdatableContract;
use whitelist_updatable_flatrate::helpers::WhitelistUpdatableFlatrateContract;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg};
use crate::state::{
    WhitelistContract, WhitelistContractType, ADMIN, CONFIG, NAME_COLLECTION, NAME_MARKETPLACE,
    PAUSED, SUDO_PARAMS, WHITELISTS,
};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:name-minter";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

const INIT_COLLECTION_REPLY_ID: u64 = 1;
const TRADING_START_TIME_OFFSET_IN_SECONDS: u64 = 2 * SECONDS_PER_YEAR;

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
        .map(|addr| WhitelistContract {
            contract_type: WhitelistContractType::UpdatableFlatrateDiscount,
            addr,
        })
        .collect::<Vec<_>>();

    WHITELISTS.save(deps.storage, &lists)?;

    PAUSED.save(deps.storage, &false)?;

    let marketplace = deps.api.addr_validate(&msg.marketplace_addr)?;
    NAME_MARKETPLACE.save(deps.storage, &marketplace)?;

    let params = SudoParams {
        min_name_length: msg.min_name_length,
        max_name_length: msg.max_name_length,
        base_price: msg.base_price,
        fair_burn_percent: Decimal::percent(msg.fair_burn_bps) / Uint128::from(100u128),
    };
    SUDO_PARAMS.save(deps.storage, &params)?;

    let config = Config {
        public_mint_start_time: PUBLIC_MINT_START_TIME_IN_SECONDS,
    };
    CONFIG.save(deps.storage, &config)?;

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
            start_trading_time: Some(
                env.block
                    .time
                    .plus_seconds(TRADING_START_TIME_OFFSET_IN_SECONDS),
            ),
            royalty_info: None,
        },
    };
    let name_collection_init_msg = NameCollectionInstantiateMsg {
        verifier: msg.verifier,
        base_init_msg: collection_init_msg,
    };

    let wasm_msg = WasmMsg::Instantiate {
        code_id: msg.collection_code_id,
        msg: to_json_binary(&name_collection_init_msg)?,
        funds: info.funds,
        admin: Some(info.sender.to_string()),
        label: "Name Collection".to_string(),
    };
    let submsg = SubMsg::reply_on_success(wasm_msg, INIT_COLLECTION_REPLY_ID);

    Ok(Response::new()
        .add_attribute("action", "instantiate")
        .add_attribute("names_minter_addr", env.contract.address.to_string())
        .add_submessage(submsg))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    let api = deps.api;

    match msg {
        ExecuteMsg::MintAndList { name } => execute_mint_and_list(deps, info, env, name.trim()),
        ExecuteMsg::UpdateAdmin { admin } => {
            Ok(ADMIN.execute_update_admin(deps, info, maybe_addr(api, admin)?)?)
        }
        ExecuteMsg::Pause { pause } => execute_pause(deps, info, pause),
        ExecuteMsg::AddWhitelist {
            address,
            whitelist_type,
        } => execute_add_whitelist(deps, info, address, whitelist_type),
        ExecuteMsg::RemoveWhitelist { address } => execute_remove_whitelist(deps, info, address),
        ExecuteMsg::UpdateConfig { config } => execute_update_config(deps, info, env, config),
    }
}

/// Mint a name for the sender, or `contract` if specified
pub fn execute_mint_and_list(
    deps: DepsMut,
    info: MessageInfo,
    env: Env,
    name: &str,
) -> Result<Response, ContractError> {
    if PAUSED.load(deps.storage)? {
        return Err(ContractError::MintingPaused {});
    }

    let whitelists = WHITELISTS.load(deps.storage)?;
    let sender = &info.sender.to_string();
    let mut res = Response::new();
    let config = CONFIG.load(deps.storage)?;

    let params = SUDO_PARAMS.load(deps.storage)?;
    validate_name(name, params.min_name_length, params.max_name_length)?;

    // Assumes no duplicate addresses between whitelists
    // Otherwise there will be edge cases with per addr limit between the whitelists
    // currently this is going to match the _first_ WL they appear in...
    let list = whitelists
        .iter()
        .find(|whitelist| match whitelist.contract_type {
            WhitelistContractType::UpdatableFlatrateDiscount => {
                let contract = WhitelistUpdatableFlatrateContract(whitelist.addr.clone());
                contract
                    .includes(&deps.querier, sender.to_string())
                    .unwrap_or(false)
            }
            WhitelistContractType::UpdatablePercentDiscount => {
                let contract = WhitelistUpdatableContract(whitelist.addr.clone());
                contract
                    .includes(&deps.querier, sender.to_string())
                    .unwrap_or(false)
            }
        });

    // if not on any whitelist, check public mint start time
    if list.is_none() && env.block.time < config.public_mint_start_time {
        return Err(ContractError::MintingNotStarted {});
    }

    if let Some(list) = list {
        match list.contract_type {
            WhitelistContractType::UpdatableFlatrateDiscount => {
                let contract = WhitelistUpdatableFlatrateContract(list.addr.clone());
                res.messages
                    .push(SubMsg::new(contract.process_address(sender)?));
            }
            WhitelistContractType::UpdatablePercentDiscount => {
                let contract = WhitelistUpdatableContract(list.addr.clone());
                res.messages
                    .push(SubMsg::new(contract.process_address(sender)?));
            }
        }
    }

    let discount = list.map(|list| match list.contract_type {
        WhitelistContractType::UpdatableFlatrateDiscount => {
            let contract = WhitelistUpdatableFlatrateContract(list.addr.clone());
            Discount::Flatrate(
                contract
                    .mint_discount_amount(&deps.querier)
                    .unwrap()
                    .unwrap_or(0u64),
            )
        }
        WhitelistContractType::UpdatablePercentDiscount => {
            let contract = WhitelistUpdatableContract(list.addr.clone());
            Discount::Percent(
                contract
                    .mint_discount_percent(&deps.querier)
                    .unwrap()
                    .unwrap_or(Decimal::zero()),
            )
        }
    });

    let price = validate_payment(name.len(), &info, params.base_price.u128(), discount)?;
    if price.is_some() {
        charge_fees(
            &mut res,
            params.fair_burn_percent,
            price.clone().unwrap().amount,
        );
    }

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
        msg: to_json_binary(&mint_msg)?,
        funds: vec![],
    };

    let ask_msg = MarketplaceExecuteMsg::SetAsk {
        token_id: name.to_string(),
        seller: sender.to_string(),
    };
    let list_msg_exec = WasmMsg::Execute {
        contract_addr: marketplace.to_string(),
        msg: to_json_binary(&ask_msg)?,
        funds: vec![],
    };

    let event = Event::new("mint-and-list")
        .add_attribute("name", name)
        .add_attribute("owner", sender)
        .add_attribute(
            "price",
            price
                .unwrap_or_else(|| coin(0, NATIVE_DENOM))
                .amount
                .to_string(),
        );
    Ok(res
        .add_event(event)
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
    whitelist_type: String,
) -> Result<Response, ContractError> {
    ADMIN.assert_admin(deps.as_ref(), &info.sender)?;

    let whitelist = deps
        .api
        .addr_validate(&address)
        .map(WhitelistUpdatableFlatrateContract)?;

    let mut lists = WHITELISTS.load(deps.storage)?;
    match whitelist_type.as_str() {
        "FlatrateDiscount" => {
            lists.push(WhitelistContract {
                contract_type: WhitelistContractType::UpdatableFlatrateDiscount,
                addr: whitelist.addr(),
            });
        }
        "PercentDiscount" => {
            lists.push(WhitelistContract {
                contract_type: WhitelistContractType::UpdatablePercentDiscount,
                addr: whitelist.addr(),
            });
        }
        _ => return Err(ContractError::InvalidWhitelistType {}),
    }

    WHITELISTS.save(deps.storage, &lists)?;

    let event = Event::new("add-whitelist").add_attribute("address", address);
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
    lists.retain(|whitelist_contract| whitelist_contract.addr != whitelist);

    WHITELISTS.save(deps.storage, &lists)?;

    let event = Event::new("remove-whitelist").add_attribute("address", address);
    Ok(Response::new().add_event(event))
}

pub fn execute_update_config(
    deps: DepsMut,
    info: MessageInfo,
    env: Env,
    config: Config,
) -> Result<Response, ContractError> {
    ADMIN.assert_admin(deps.as_ref(), &info.sender)?;
    let start_time = config.public_mint_start_time;

    // Can not set public mint time in the past
    if env.block.time > start_time {
        return Err(ContractError::InvalidTradingStartTime(
            env.block.time,
            start_time,
        ));
    }

    CONFIG.save(deps.storage, &config)?;

    let event = Event::new("update-config").add_attribute("address", info.sender.to_string());
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

    (if name.starts_with('-') || name.ends_with('-') {
        Err(ContractError::InvalidName {})
    } else {
        Ok(())
    })?;

    if len > 4 && name[2..4].contains("--") {
        return Err(ContractError::InvalidName {});
    }

    Ok(())
}

pub enum Discount {
    Flatrate(u64),
    Percent(Decimal),
}

fn validate_payment(
    name_len: usize,
    info: &MessageInfo,
    base_price: u128,
    discount: Option<Discount>,
) -> Result<Option<Coin>, ContractError> {
    // Because we know we are left with ASCII chars, a simple byte count is enough
    let mut amount: Uint128 = (match name_len {
        0..=2 => {
            return Err(ContractError::NameTooShort {});
        }
        3 => base_price * 100,
        4 => base_price * 10,
        _ => base_price,
    })
    .into();

    match discount {
        Some(Discount::Flatrate(discount)) => {
            let discount = Uint128::from(discount);
            if amount.ge(&discount) {
                amount = amount
                    .checked_sub(discount)
                    .map_err(|_| StdError::generic_err("invalid discount amount"))?;
            }
        }
        Some(Discount::Percent(discount)) => {
            amount = amount * (Decimal::one() - discount);
        }
        None => {}
    }

    if amount.is_zero() {
        return Ok(None);
    }

    let payment = must_pay(info, NATIVE_DENOM)?;
    if payment != amount {
        return Err(ContractError::IncorrectPayment {
            got: payment.u128(),
            expected: amount.u128(),
        });
    }

    Ok(Some(coin(amount.u128(), NATIVE_DENOM)))
}

fn invalid_char(c: char) -> bool {
    let is_valid = c.is_ascii_digit() || c.is_ascii_lowercase() || c == '-';
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
                msg: to_json_binary(
                    &(SgNameExecuteMsg::SetNameMarketplace {
                        address: NAME_MARKETPLACE.load(deps.storage)?.to_string(),
                    }),
                )?,
            };

            Ok(Response::default()
                .add_attribute("action", "init_collection_reply")
                .add_message(msg))
        }
        Err(_) => Err(ContractError::ReplyOnSuccess {}),
    }
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

#[cfg(test)]
mod tests {
    use cosmwasm_std::{coin, Addr, MessageInfo};

    use crate::contract::{self, validate_name};

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
                .unwrap()
                .amount
                .u128(),
            base_price * 100
        );
    }

    #[test]
    fn check_validate_payment_with_flatrate_discount() {
        let base_price = 100_000_000;

        let info = MessageInfo {
            sender: Addr::unchecked("sender"),
            funds: vec![coin(base_price - 100, "ustars")],
        };
        assert_eq!(
            // we treat the discount as a flat amount given as 100.0
            validate_payment(
                5,
                &info,
                base_price,
                Some(contract::Discount::Flatrate(100)),
            )
            .unwrap()
            .unwrap()
            .amount
            .u128(),
            base_price - 100
        );
    }
}
