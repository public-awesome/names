#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_binary, Binary, Deps, Env, StdResult};
use sg_name_minter::{CollectionResponse, ParamsResponse, WhitelistsResponse};

use crate::{
    msg::QueryMsg,
    state::{ADMIN, NAME_COLLECTION, SUDO_PARAMS, WHITELISTS},
};

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Admin {} => to_binary(&ADMIN.query_admin(deps)?),
        QueryMsg::Collection {} => to_binary(&query_collection(deps)?),
        QueryMsg::Params {} => to_binary(&query_params(deps)?),
        QueryMsg::Whitelists {} => to_binary(&query_whitelists(deps)?),
    }
}

fn query_whitelists(deps: Deps) -> StdResult<WhitelistsResponse> {
    let whitelists = WHITELISTS.load(deps.storage)?;
    Ok(WhitelistsResponse {
        whitelists: whitelists.iter().map(|w| w.addr()).collect(),
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

    Ok(ParamsResponse { params })
}
