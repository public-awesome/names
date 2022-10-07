#[cfg(not(feature = "library"))]
use cosmwasm_std::{Addr, Deps, DepsMut, MessageInfo};

use crate::error::ContractError;
use cw721_base::Extension;
use cw_utils::nonpayable;
use sg721_base::ContractError::Unauthorized;
use sg_name::Metadata;
use sg_std::Response;

pub type Sg721NameContract<'a> = sg721_base::Sg721Contract<'a, Metadata<Extension>>;

pub fn execute_update_bio(
    deps: DepsMut,
    info: MessageInfo,
    name: String,
    bio: Option<String>,
) -> Result<Response, ContractError> {
    nonpayable(&info)?;
    let token_id = name;
    if is_sender_token_id_owner(deps.as_ref(), info.sender, &token_id) == false {
        return Err(ContractError::Base(Unauthorized {}));
    }
    // load token info
    Sg721NameContract::default()
        .tokens
        .update(deps.storage, &token_id, |token| match token {
            Some(mut token_info) => {
                token_info.extension.bio = bio.clone();
                Ok(token_info)
            }
            None => Err(ContractError::NameNotFound {}),
        })?;
    Ok(Response::new())
}

fn is_sender_token_id_owner(deps: Deps, sender: Addr, token_id: &str) -> bool {
    let owner = match Sg721NameContract::default()
        .tokens
        .load(deps.storage, &token_id)
    {
        Ok(token_info) => token_info.owner,
        Err(_) => return false,
    };
    owner == sender
}
