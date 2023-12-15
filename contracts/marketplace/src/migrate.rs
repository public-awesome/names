use crate::{
    error::ContractError,
    execute::{CONTRACT_NAME, CONTRACT_VERSION},
    state::{SudoParams, SUDO_PARAMS},
};

use cosmwasm_schema::cw_serde;
use cosmwasm_std::{ensure, Decimal, DepsMut, Env, Event, StdError, Uint128};
use cw_storage_plus::Item;
use sg_std::Response;

#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;

#[cw_serde]
pub struct SudoParamsV1_2 {
    /// Fair Burn + Community Pool fee for winning bids
    pub trading_fee_percent: Decimal,
    /// Min value for a bid
    pub min_price: Uint128,
    /// Interval to rate limit setting asks (in seconds)
    pub ask_interval: u64,
}

pub const SUDO_PARAMS_V1_2: Item<SudoParamsV1_2> = Item::new("sudo-params");

#[cw_serde]
pub struct MigrateMsg {
    max_renewals_per_block: u32,
    valid_bid_query_limit: u32,
    valid_bid_seconds_delta: u64,
    renewal_bid_percentage: Decimal,
}

#[cfg_attr(not(feature = "library"), entry_point)]
#[allow(clippy::cmp_owned)]
pub fn migrate(deps: DepsMut, _env: Env, msg: MigrateMsg) -> Result<Response, ContractError> {
    let prev_contract_version = cw2::get_contract_version(deps.storage)?;

    let valid_contract_names = vec![CONTRACT_NAME.to_string()];
    ensure!(
        valid_contract_names.contains(&prev_contract_version.contract),
        StdError::generic_err("Invalid contract name for migration")
    );

    ensure!(
        prev_contract_version.version < CONTRACT_VERSION.to_string(),
        StdError::generic_err("Must upgrade contract version")
    );

    let sudo_params_v1_2 = SUDO_PARAMS_V1_2.load(deps.storage)?;
    let sudo_params = SudoParams {
        trading_fee_percent: sudo_params_v1_2.trading_fee_percent,
        min_price: sudo_params_v1_2.min_price,
        ask_interval: sudo_params_v1_2.ask_interval,
        max_renewals_per_block: msg.max_renewals_per_block,
        valid_bid_query_limit: msg.valid_bid_query_limit,
        valid_bid_seconds_delta: msg.valid_bid_seconds_delta,
        renewal_bid_percentage: msg.renewal_bid_percentage,
    };
    SUDO_PARAMS.save(deps.storage, &sudo_params)?;

    cw2::set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let response = Response::new().add_event(
        Event::new("migrate")
            .add_attribute("from_name", prev_contract_version.contract)
            .add_attribute("from_version", prev_contract_version.version)
            .add_attribute("to_name", CONTRACT_NAME)
            .add_attribute("to_version", CONTRACT_VERSION),
    );

    Ok(response)
}
