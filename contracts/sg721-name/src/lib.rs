pub use crate::error::ContractError;
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use semver::Version;
use sg_name::Metadata;

pub mod contract;
mod error;
pub mod msg;
pub mod state;
pub mod sudo;

#[cfg(test)]
pub mod unit_tests;

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:sg721-name";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub type Sg721NameContract<'a> = sg721_base::Sg721Contract<'a, Metadata>;
pub type ExecuteMsg = crate::msg::ExecuteMsg<Metadata>;
pub type QueryMsg = crate::msg::QueryMsg;

pub mod entry {
    use crate::{
        contract::{execute_verify_text_record, query_image_nft, query_text_records},
        msg::InstantiateMsg,
        state::{SudoParams, SUDO_PARAMS, VERIFIER},
    };

    use super::*;

    use contract::{
        execute_add_text_record, execute_associate_address, execute_burn, execute_mint,
        execute_remove_text_record, execute_send_nft, execute_set_name_marketplace,
        execute_transfer_nft, execute_update_image_nft, execute_update_text_record,
        query_associated_address, query_name, query_name_marketplace, query_params,
    };
    use cosmwasm_std::{
        to_binary, Binary, Deps, DepsMut, Empty, Env, MessageInfo, StdError, StdResult,
    };
    use cw2::set_contract_version;
    use cw_utils::maybe_addr;
    use sg721_base::ContractError as Sg721ContractError;
    use sg_std::Response;

    #[cfg_attr(not(feature = "library"), entry_point)]
    pub fn instantiate(
        mut deps: DepsMut,
        env: Env,
        info: MessageInfo,
        msg: InstantiateMsg,
    ) -> Result<Response, Sg721ContractError> {
        cw2::set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

        // Initialize max record count to 10, can be changed by sudo params
        SUDO_PARAMS.save(
            deps.storage,
            &SudoParams {
                max_record_count: 10,
            },
        )?;

        let api = deps.api;
        VERIFIER.set(deps.branch(), maybe_addr(api, msg.verifier)?)?;

        let res =
            Sg721NameContract::default().instantiate(deps, env.clone(), info, msg.base_init_msg)?;

        Ok(res
            .add_attribute("action", "instantiate")
            .add_attribute("sg721_names_addr", env.contract.address.to_string()))
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
            ExecuteMsg::AssociateAddress { name, address } => {
                execute_associate_address(deps, info, name, address)
            }
            ExecuteMsg::UpdateImageNft { name, nft } => {
                execute_update_image_nft(deps, info, name, nft)
            }
            ExecuteMsg::AddTextRecord { name, record } => {
                execute_add_text_record(deps, info, name, record)
            }
            ExecuteMsg::RemoveTextRecord { name, record_name } => {
                execute_remove_text_record(deps, info, name, record_name)
            }
            ExecuteMsg::UpdateTextRecord { name, record } => {
                execute_update_text_record(deps, info, name, record)
            }
            ExecuteMsg::VerifyTextRecord {
                name,
                record_name,
                result,
            } => execute_verify_text_record(deps, info, name, record_name, result),
            ExecuteMsg::UpdateVerifier { verifier } => {
                Ok(VERIFIER.execute_update_admin(deps, info, maybe_addr(api, verifier)?)?)
            }
            ExecuteMsg::SetNameMarketplace { address } => {
                execute_set_name_marketplace(deps, info, address)
            }
            ExecuteMsg::TransferNft {
                recipient,
                token_id,
            } => execute_transfer_nft(deps, env, info, recipient, token_id),
            ExecuteMsg::SendNft {
                contract,
                token_id,
                msg,
            } => execute_send_nft(deps, env, info, contract, token_id, msg),
            ExecuteMsg::Mint(msg) => execute_mint(deps, info, msg),
            ExecuteMsg::Burn { token_id } => execute_burn(deps, env, info, token_id),
            _ => Sg721NameContract::default()
                .execute(deps, env, info, msg.into())
                .map_err(|e| e.into()),
        }
    }

    #[cfg_attr(not(feature = "library"), entry_point)]
    pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
        match msg {
            QueryMsg::Params {} => to_binary(&query_params(deps)?),
            QueryMsg::NameMarketplace {} => to_binary(&query_name_marketplace(deps)?),
            QueryMsg::Name { address } => to_binary(&query_name(deps, address)?),
            QueryMsg::Verifier {} => to_binary(&VERIFIER.query_admin(deps)?),
            QueryMsg::AssociatedAddress { name } => {
                to_binary(&query_associated_address(deps, &name)?)
            }
            QueryMsg::ImageNFT { name } => to_binary(&query_image_nft(deps, &name)?),
            QueryMsg::TextRecords { name } => to_binary(&query_text_records(deps, &name)?),
            _ => Sg721NameContract::default().query(deps, env, msg.into()),
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
            return Err(
                StdError::generic_err("Cannot upgrade to a previous contract version").into(),
            );
        }
        // if same version return
        if version == new_version {
            return Ok(Response::new());
        }

        // set new contract version
        set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
        Ok(Response::new())
    }
}
