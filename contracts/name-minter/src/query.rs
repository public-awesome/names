#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_binary, Binary, Deps, Env, StdResult, Uint128};

use crate::{
    msg::{CollectionResponse, ParamsResponse, QueryMsg, WhitelistResponse},
    state::{ADMIN, NAME_COLLECTION, SUDO_PARAMS, WHITELIST},
};

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Admin {} => to_binary(&ADMIN.query_admin(deps)?),
        QueryMsg::Collection {} => to_binary(&query_collection(deps)?),
        QueryMsg::Params {} => to_binary(&query_params(deps)?),
        QueryMsg::Whitelist {} => to_binary(&query_whitelist(deps)?),
    }
}

fn query_whitelist(deps: Deps) -> StdResult<WhitelistResponse> {
    let whitelist = WHITELIST.load(deps.storage)?;
    Ok(WhitelistResponse {
        whitelist: whitelist.map(|x| x.to_string()),
    })
}

fn query_collection(deps: Deps) -> StdResult<CollectionResponse> {
    let collection = NAME_COLLECTION.load(deps.storage)?;
    Ok(CollectionResponse {
        collection: collection.to_string(),
    })
}

fn query_params(deps: Deps) -> StdResult<ParamsResponse> {
    let params = SUDO_PARAMS.load(deps.storage)?;

    Ok(ParamsResponse {
        base_price: Uint128::from(params.base_price),
        min_name_length: params.min_name_length,
        max_name_length: params.max_name_length,
    })
}
