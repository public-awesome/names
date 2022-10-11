pub use crate::error::ContractError;
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cw721_base::Extension;
use sg_name::Metadata;

pub mod contract;
mod error;
pub mod msg;
pub mod state;

// TODO: add lookup map

#[cfg(test)]
pub mod unit_tests;

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:sg721-name";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub type Sg721NameContract<'a> = sg721_base::Sg721Contract<'a, Metadata<Extension>>;
pub type InstantiateMsg = sg721::InstantiateMsg;
pub type ExecuteMsg = crate::msg::ExecuteMsg<Metadata<Extension>>;
pub type QueryMsg = crate::msg::QueryMsg;

pub mod entry {
    use crate::contract::{execute_set_name_marketplace, query_name_marketplace};

    use super::*;

    use contract::{
        execute_add_text_record, execute_remove_text_record, execute_transfer_nft,
        execute_update_bio, execute_update_profile, execute_update_text_record,
    };
    use cosmwasm_std::{to_binary, Binary, Deps, DepsMut, Env, MessageInfo, StdResult};
    use sg721_base::ContractError as Sg721ContractError;
    use sg_std::Response;

    #[cfg_attr(not(feature = "library"), entry_point)]
    pub fn instantiate(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        msg: InstantiateMsg,
    ) -> Result<Response, Sg721ContractError> {
        cw2::set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

        let res = Sg721NameContract::default().instantiate(deps, env, info, msg)?;

        Ok(res
            .add_attribute("contract_name", CONTRACT_NAME)
            .add_attribute("contract_version", CONTRACT_VERSION))
    }

    #[cfg_attr(not(feature = "library"), entry_point)]
    pub fn execute(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        msg: ExecuteMsg,
    ) -> Result<Response, ContractError> {
        match msg {
            ExecuteMsg::UpdateBio { name, bio } => execute_update_bio(deps, info, name, bio),
            ExecuteMsg::UpdateProfile { name, profile } => {
                execute_update_profile(deps, info, name, profile)
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
            ExecuteMsg::SetNameMarketplace { address } => {
                execute_set_name_marketplace(deps, info, address)
            }
            ExecuteMsg::TransferNft {
                recipient,
                token_id,
            } => execute_transfer_nft(deps, env, info, recipient, token_id),
            _ => Sg721NameContract::default()
                .execute(deps, env, info, msg.into())
                .map_err(|e| e.into()),
        }
    }

    #[cfg_attr(not(feature = "library"), entry_point)]
    pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
        match msg {
            QueryMsg::NameMarketplace {} => to_binary(&query_name_marketplace(deps)?),
            _ => Sg721NameContract::default().query(deps, env, msg.into()),
        }
    }
}
