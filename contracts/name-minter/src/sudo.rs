#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{Addr, Decimal, DepsMut, Env, Event, Uint128};
use sg_name_minter::SudoParams;
use sg_std::Response;

use crate::{
    msg::SudoMsg,
    state::{NAME_COLLECTION, NAME_MARKETPLACE, SUDO_PARAMS},
    ContractError,
};

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn sudo(deps: DepsMut, _env: Env, msg: SudoMsg) -> Result<Response, ContractError> {
    let api = deps.api;

    match msg {
        SudoMsg::UpdateParams {
            min_name_length,
            max_name_length,
            base_price,
            fair_burn_bps,
        } => sudo_update_params(
            deps,
            min_name_length,
            max_name_length,
            base_price,
            fair_burn_bps,
        ),
        SudoMsg::UpdateNameCollection { collection } => {
            sudo_update_name_collection(deps, api.addr_validate(&collection)?)
        }
        SudoMsg::UpdateNameMarketplace { marketplace } => {
            sudo_update_name_marketplace(deps, api.addr_validate(&marketplace)?)
        }
    }
}

pub fn sudo_update_params(
    deps: DepsMut,
    min_name_length: u32,
    max_name_length: u32,
    base_price: Uint128,
    fair_burn_bps: u64,
) -> Result<Response, ContractError> {
    SUDO_PARAMS.save(
        deps.storage,
        &SudoParams {
            min_name_length,
            max_name_length,
            base_price,
            fair_burn_percent: Decimal::percent(fair_burn_bps) / Uint128::from(100u128),
        },
    )?;

    Ok(Response::new().add_attribute("action", "sudo_update_params"))
}

pub fn sudo_update_name_collection(
    deps: DepsMut,
    collection: Addr,
) -> Result<Response, ContractError> {
    NAME_COLLECTION.save(deps.storage, &collection)?;

    let event = Event::new("update_name_collection").add_attribute("collection", collection);
    Ok(Response::new().add_event(event))
}

pub fn sudo_update_name_marketplace(
    deps: DepsMut,
    marketplace: Addr,
) -> Result<Response, ContractError> {
    NAME_MARKETPLACE.save(deps.storage, &marketplace)?;

    let event = Event::new("update_name_marketplace").add_attribute("marketplace", marketplace);
    Ok(Response::new().add_event(event))
}
