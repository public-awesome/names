#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_binary, Binary, Deps, Env, StdResult, Uint128};

use crate::{
    msg::{ConfigResponse, ParamsResponse, QueryMsg},
    state::{ADMIN, NAME_COLLECTION, SUDO_PARAMS},
};

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Admin {} => to_binary(&ADMIN.query_admin(deps)?),
        QueryMsg::Config {} => to_binary(&query_collection_addr(deps)?),
        QueryMsg::Params {} => to_binary(&query_params(deps)?),
    }
}

fn query_collection_addr(deps: Deps) -> StdResult<ConfigResponse> {
    let config = NAME_COLLECTION.load(deps.storage)?;
    Ok(ConfigResponse {
        collection_addr: config.to_string(),
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
