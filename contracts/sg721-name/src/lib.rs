use cw721_base::Extension;
pub use sg721_base::ContractError;
use sg_name::Metadata;

pub mod contract;
mod error;
pub mod msg;
pub mod state;
pub mod tests;

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:sg721-name";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub type Sg721MetadataContract<'a> = sg721_base::Sg721Contract<'a, Metadata<Extension>>;
pub type InstantiateMsg = sg721::InstantiateMsg;
pub type ExecuteMsg = crate::msg::ExecuteMsg<Metadata<Extension>>;
pub type QueryMsg = sg721_base::msg::QueryMsg;

#[cfg(not(feature = "library"))]
pub mod entry {
    use super::*;

    use cosmwasm_std::{entry_point, Binary, Deps, DepsMut, Env, MessageInfo, StdResult};
    use sg721_base::{msg::QueryMsg, ContractError};
    use sg_std::Response;

    #[entry_point]
    pub fn instantiate(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        msg: InstantiateMsg,
    ) -> Result<Response, ContractError> {
        cw2::set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

        let res = Sg721MetadataContract::default().instantiate(deps, env, info, msg)?;

        Ok(res
            .add_attribute("contract_name", CONTRACT_NAME)
            .add_attribute("contract_version", CONTRACT_VERSION))
    }

    #[entry_point]
    pub fn execute(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        msg: ExecuteMsg,
    ) -> Result<Response, ContractError> {
        match msg {
            ExecuteMsg::UpdateBio { name, bio } => unimplemented!(),
            ExecuteMsg::UpdateProfile { name, profile } => unimplemented!(),
            ExecuteMsg::AddTextRecord { name, record } => unimplemented!(),
            ExecuteMsg::RemoveTextRecord { name, record_name } => unimplemented!(),
            ExecuteMsg::UpdateTextRecord { name, record } => unimplemented!(),
            _ => Sg721MetadataContract::default()
                .execute(deps, env, info, msg.into())
                .map_err(|e| e.into()),
        }
    }

    #[entry_point]
    pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
        Sg721MetadataContract::default().query(deps, env, msg)
    }
}
