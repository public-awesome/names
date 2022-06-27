use cosmwasm_std::{Coin, Empty, Timestamp};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug, Default)]
pub struct RenewalExtension<T> {
    pub expiration: Timestamp,
    pub cost: Coin,
    pub collected: Coin,
    pub extension: T,
}
pub type Extension = RenewalExtension<()>;

pub type SubscriptionContract<'a> = cw721_base::Cw721Contract<'a, Extension, Empty>;
pub type ExecuteMsg = cw721_base::ExecuteMsg<Extension>;

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub enum SubscriptionExecuteMsg {
    /// Renew the subscription by paying all or a partial amount.
    /// Only the final payer becomes the new owner.
    Renew { token_id: String },
    /// Someone may call this to take ownership and of an expired subscription
    Claim { token_id: String },
    /// Someone may call this to burn and collect fees from an expired subscription
    Burn { token_id: String },
    /// cw721 base execute message
    ExecuteMsg(ExecuteMsg),
}

#[cfg(not(feature = "library"))]
pub mod entry {
    use super::*;

    use cosmwasm_std::entry_point;
    use cosmwasm_std::{Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};
    use cw721_base::{ContractError, InstantiateMsg, QueryMsg};

    // This is a simple type to let us handle empty extensions

    // This makes a conscious choice on the various generics used by the contract
    #[entry_point]
    pub fn instantiate(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        msg: InstantiateMsg,
    ) -> StdResult<Response> {
        SubscriptionContract::default().instantiate(deps, env, info, msg)
    }

    #[entry_point]
    pub fn execute(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        msg: SubscriptionExecuteMsg,
    ) -> Result<Response, ContractError> {
        match msg {
            SubscriptionExecuteMsg::Renew { token_id } => todo!(),
            SubscriptionExecuteMsg::Claim { token_id } => todo!(),
            SubscriptionExecuteMsg::Burn { token_id } => todo!(),
            SubscriptionExecuteMsg::ExecuteMsg(m) => {
                SubscriptionContract::default().execute(deps, env, info, m)
            }
        }
    }

    #[entry_point]
    pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
        SubscriptionContract::default().query(deps, env, msg)
    }
}
