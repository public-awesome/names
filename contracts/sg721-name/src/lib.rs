pub use crate::error::ContractError;
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
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
        contract::execute_verify_text_record,
        msg::InstantiateMsg,
        state::{SudoParams, SUDO_PARAMS, VERIFIER},
    };

    use super::*;

    use contract::{
        execute_add_text_record, execute_associate_address, execute_burn, execute_mint,
        execute_remove_text_record, execute_send_nft, execute_set_name_marketplace,
        execute_transfer_nft, execute_update_image_nft, execute_update_metadata,
        execute_update_text_record, query_name, query_name_marketplace, query_params,
    };
    use cosmwasm_std::{to_binary, Binary, Deps, DepsMut, Env, MessageInfo, StdResult};
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
            ExecuteMsg::UpdateMetadata { name, metadata } => {
                execute_update_metadata(deps, env, info, name, metadata)
            }
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
            _ => Sg721NameContract::default().query(deps, env, msg.into()),
        }
    }
}
