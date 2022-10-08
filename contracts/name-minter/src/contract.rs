#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    coin, to_binary, Addr, Binary, Coin, Deps, DepsMut, Env, MessageInfo, Reply, StdResult, SubMsg,
    WasmMsg,
};
use cw2::set_contract_version;
use cw721_base::Extension;
use cw_utils::{must_pay, parse_reply_instantiate_data};
use name_marketplace::msg::ExecuteMsg as MarketplaceExecuteMsg;
use sg721::{CollectionInfo, MintMsg};
use sg721_name::{ExecuteMsg as Sg721ExecuteMsg, InstantiateMsg as Sg721InstantiateMsg};
use sg_name::Metadata;
use sg_std::{create_fund_community_pool_msg, Response, NATIVE_DENOM};

use crate::error::ContractError;
use crate::msg::{ConfigResponse, ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{NAME_COLLECTION, NAME_MARKETPLACE};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:name-minter";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

const INIT_COLLECTION_REPLY_ID: u64 = 1;

// TODO: make these sudo params
const MIN_NAME_LENGTH: u64 = 3;
const MAX_NAME_LENGTH: u64 = 63;
const BASE_PRICE: u128 = 100000000;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let marketplace = deps.api.addr_validate(&msg.marketplace_addr)?;
    NAME_MARKETPLACE.save(deps.storage, &marketplace)?;

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
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::MintAndList { name } => execute_mint_and_list(deps, env, info, name.trim()),
    }
}

pub fn execute_mint_and_list(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    name: &str,
) -> Result<Response, ContractError> {
    validate_name(name)?;

    let price = validate_payment(name.len(), &info)?;
    let community_pool_msg = create_fund_community_pool_msg(vec![price]);
    let collection = NAME_COLLECTION.load(deps.storage)?;
    let marketplace = NAME_MARKETPLACE.load(deps.storage)?;

    let msg = Sg721ExecuteMsg::Mint(MintMsg::<Metadata<Extension>> {
        token_id: name.trim().to_string(),
        owner: info.sender.to_string(),
        token_uri: None,
        extension: Metadata {
            bio: None,
            profile: None,
            records: vec![],
            extension: None,
        },
    });
    let mint_msg_exec = WasmMsg::Execute {
        contract_addr: collection.to_string(),
        msg: to_binary(&msg)?,
        funds: vec![],
    };

    let msg = MarketplaceExecuteMsg::SetAsk {
        token_id: name.to_string(),
    };
    let list_msg_exec = WasmMsg::Execute {
        contract_addr: marketplace.to_string(),
        msg: to_binary(&msg)?,
        funds: vec![],
    };

    Ok(Response::new()
        .add_attribute("action", "mint_and_list")
        .add_message(community_pool_msg)
        .add_message(mint_msg_exec)
        .add_message(list_msg_exec))
}

// This follows the same rules as Internet domain names
fn validate_name(name: &str) -> Result<(), ContractError> {
    let len = name.len() as u64;
    if len < MIN_NAME_LENGTH {
        return Err(ContractError::NameTooShort {});
    } else if len >= MAX_NAME_LENGTH {
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

fn validate_payment(name_len: usize, info: &MessageInfo) -> Result<Coin, ContractError> {
    // Because we know we are left with ASCII chars, a simple byte count is enough
    let amount = match name_len {
        0..=2 => return Err(ContractError::NameTooShort {}),
        3 => BASE_PRICE * 100,
        4 => BASE_PRICE * 10,
        _ => BASE_PRICE,
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

#[cfg(test)]
mod tests {
    use cosmwasm_std::{coin, Addr, MessageInfo};

    use crate::contract::BASE_PRICE;

    use super::{validate_name, validate_payment};

    #[test]
    fn check_validate_name() {
        assert!(validate_name("bobo").is_ok());
        assert!(validate_name("-bobo").is_err());
        assert!(validate_name("bobo-").is_err());
        assert!(validate_name("bo-bo").is_ok());
        assert!(validate_name("bo--bo").is_err());
        assert!(validate_name("bob--o").is_ok());
        assert!(validate_name("bo").is_err());
        assert!(validate_name("b").is_err());
        assert!(validate_name("bob").is_ok());
        assert!(
            validate_name("bobobobobobobobobobobobobobobobobobobobobobobobobobobobobobobo").is_ok()
        );
        assert!(
            validate_name("bobobobobobobobobobobobobobobobobobobobobobobobobobobobobobobob")
                .is_err()
        );
        assert!(validate_name("0123456789").is_ok());
        assert!(validate_name("😬").is_err());
        assert!(validate_name("BOBO").is_err());
        assert!(validate_name("b-o----b").is_ok());
        assert!(validate_name("bobo.stars").is_err());
    }

    #[test]
    fn check_validate_payment() {
        let info = MessageInfo {
            sender: Addr::unchecked("sender"),
            funds: vec![coin(BASE_PRICE, "ustars")],
        };
        assert_eq!(
            validate_payment(5, &info).unwrap().amount.u128(),
            BASE_PRICE
        );

        let info = MessageInfo {
            sender: Addr::unchecked("sender"),
            funds: vec![coin(BASE_PRICE * 10, "ustars")],
        };
        assert_eq!(
            validate_payment(4, &info).unwrap().amount.u128(),
            BASE_PRICE * 10
        );

        let info = MessageInfo {
            sender: Addr::unchecked("sender"),
            funds: vec![coin(BASE_PRICE * 100, "ustars")],
        };
        assert_eq!(
            validate_payment(3, &info).unwrap().amount.u128(),
            BASE_PRICE * 100
        );
    }
}
