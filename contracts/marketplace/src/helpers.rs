use std::cmp::max;

use crate::{
    execute::{finalize_sale, store_ask},
    msg::{ExecuteMsg, QueryMsg},
    state::{bid_key, bids, Ask, Bid, SudoParams, RENEWAL_QUEUE},
    ContractError,
};
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{
    coins, ensure, to_binary, Addr, BankMsg, Deps, DepsMut, Env, Event, Order, QuerierWrapper,
    QueryRequest, StdError, StdResult, Timestamp, Uint128, WasmMsg, WasmQuery,
};
use cw_storage_plus::Bound;
use sg_name_common::{charge_fees, SECONDS_PER_YEAR};
use sg_name_minter::SudoParams as NameMinterParams;
use sg_std::{CosmosMsg, Response, NATIVE_DENOM};

/// MarketplaceContract is a wrapper around Addr that provides a lot of helpers
#[cw_serde]
pub struct NameMarketplaceContract(pub Addr);

impl NameMarketplaceContract {
    pub fn addr(&self) -> Addr {
        self.0.clone()
    }

    pub fn call<T: Into<ExecuteMsg>>(&self, msg: T) -> StdResult<CosmosMsg> {
        let msg = to_binary(&msg.into())?;
        Ok(WasmMsg::Execute {
            contract_addr: self.addr().into(),
            msg,
            funds: vec![],
        }
        .into())
    }

    pub fn remove_ask(&self, token_id: &str) -> StdResult<CosmosMsg> {
        self.call(ExecuteMsg::RemoveAsk {
            token_id: token_id.to_string(),
        })
    }

    pub fn highest_bid(&self, querier: &QuerierWrapper, token_id: &str) -> StdResult<Option<Bid>> {
        let res: Option<Bid> = querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: self.addr().into(),
            msg: to_binary(&QueryMsg::HighestBid {
                token_id: token_id.to_string(),
            })?,
        }))?;

        Ok(res)
    }

    // contract needs approval from nft owner before accepting bid
    pub fn accept_bid(
        &self,
        querier: &QuerierWrapper,
        token_id: &str,
        bidder: &str,
    ) -> StdResult<CosmosMsg> {
        let highest_bid: Option<Bid> = self.highest_bid(querier, token_id)?;
        let bid = highest_bid.ok_or_else(|| {
            StdError::generic_err(format!("No bid found for token_id {}", token_id))
        })?;

        if bid.bidder != bidder {
            return Err(StdError::generic_err(format!(
                "Bidder {} is not the highest bidder",
                bidder
            )));
        }

        self.call(ExecuteMsg::AcceptBid {
            token_id: token_id.to_string(),
            bidder: bidder.to_string(),
        })
    }
}

/// Iterate over top n priced bids, if one is within the time window then it is valid
pub fn find_valid_bid(
    deps: Deps,
    block_time: &Timestamp,
    sudo_params: &SudoParams,
    token_id: &str,
    min_price: Uint128,
) -> Result<Option<Bid>, ContractError> {
    let max_time = block_time.seconds() - sudo_params.renew_window;

    let bid = bids()
        .idx
        .price
        .sub_prefix(token_id.to_string())
        .range(
            deps.storage,
            Some(Bound::inclusive((
                min_price.u128(),
                (token_id.to_string(), Addr::unchecked("")),
            ))),
            None,
            Order::Descending,
        )
        .take(sudo_params.valid_bid_query_limit as usize)
        .find_map(|item| {
            item.map_or(None, |(_, bid)| {
                if bid.created_time.seconds() <= max_time {
                    Some(bid)
                } else {
                    None
                }
            })
        });

    Ok(bid)
}

// Calculate the renewal price based on the name length
pub fn get_char_price(base_price: u128, name_len: usize) -> Uint128 {
    match name_len {
        0..=2 => unreachable!("name_len should be at least 3"),
        3 => base_price * 100,
        4 => base_price * 10,
        _ => base_price,
    }
    .into()
}

// Renewal price is the max of the char based price and a percentage of highest valid bid
pub fn get_renewal_price_and_bid(
    deps: Deps,
    block_time: &Timestamp,
    sudo_params: &SudoParams,
    token_id: &str,
    base_price: u128,
) -> Result<(Uint128, Option<Bid>), ContractError> {
    let renewal_char_price = get_char_price(base_price, token_id.len());
    let valid_bid = find_valid_bid(deps, block_time, sudo_params, token_id, renewal_char_price)?;

    let renewal_bid_price = valid_bid.as_ref().map_or(Uint128::zero(), |bid| {
        bid.amount * sudo_params.renewal_bid_percentage
    });

    let renewal_price = max(renewal_char_price, renewal_bid_price);

    Ok((renewal_price, valid_bid))
}

pub fn renew_name(
    deps: DepsMut,
    _env: &Env,
    sudo_params: &SudoParams,
    mut ask: Ask,
    renewal_price: Uint128,
    mut response: Response,
) -> Result<Response, ContractError> {
    if !renewal_price.is_zero() {
        // Take renewal payment
        ask.renewal_fund -= renewal_price;
        charge_fees(
            &mut response,
            sudo_params.trading_fee_percent,
            renewal_price,
        );
    }

    // Update renewal time
    RENEWAL_QUEUE.remove(deps.storage, (ask.renewal_time.seconds(), ask.id));
    ask.renewal_time = ask.renewal_time.plus_seconds(SECONDS_PER_YEAR);
    RENEWAL_QUEUE.save(
        deps.storage,
        (ask.renewal_time.seconds(), ask.id),
        &ask.token_id,
    )?;

    store_ask(deps.storage, &ask)?;

    response = response.add_event(
        Event::new("renew-name")
            .add_attribute("token_id", ask.token_id.to_string())
            .add_attribute("renewal_price", renewal_price)
            .add_attribute("next_renewal_time", ask.renewal_time.to_string()),
    );

    Ok(response)
}

fn sell_name(
    deps: DepsMut,
    env: &Env,
    mut ask: Ask,
    bid: Bid,
    mut response: Response,
) -> Result<Response, ContractError> {
    // Remove accepted bid
    bids().remove(deps.storage, bid_key(&ask.token_id, &bid.bidder))?;

    // Update renewal time
    RENEWAL_QUEUE.remove(deps.storage, (ask.renewal_time.seconds(), ask.id));
    ask.renewal_time = env.block.time.plus_seconds(SECONDS_PER_YEAR);
    RENEWAL_QUEUE.save(
        deps.storage,
        (ask.renewal_time.seconds(), ask.id),
        &ask.token_id,
    )?;

    // Transfer funds and NFT
    finalize_sale(
        deps.as_ref(),
        ask.clone(),
        bid.amount,
        bid.bidder.clone(),
        &mut response,
    )?;

    store_ask(deps.storage, &ask)?;

    Ok(response)
}

pub fn process_renewal(
    deps: DepsMut,
    env: &Env,
    sudo_params: &SudoParams,
    name_minter_params: &NameMinterParams,
    mut ask: Ask,
    mut response: Response,
) -> Result<Response, ContractError> {
    ensure!(
        ask.renewal_time.seconds() <= env.block.time.seconds(),
        ContractError::CannotProcessFutureRenewal {}
    );

    let mut process_renewal_event = Event::new("process-renewal")
        .add_attribute("token_id", ask.token_id.to_string())
        .add_attribute("renewal_time", ask.renewal_time.seconds().to_string());

    let (renewal_price, valid_bid) = get_renewal_price_and_bid(
        deps.as_ref(),
        &env.block.time,
        sudo_params,
        &ask.token_id,
        name_minter_params.base_price.u128(),
    )?;

    if let Some(bid) = valid_bid {
        // If the renewal fund is sufficient, renew it
        if ask.renewal_fund >= renewal_price {
            process_renewal_event = process_renewal_event.add_attribute("action", "renew");
            response = response.add_event(process_renewal_event);

            renew_name(deps, env, sudo_params, ask, renewal_price, response)
        } else {
            // Renewal fund is insufficient, send it back to the owner
            if !ask.renewal_fund.is_zero() {
                response = response.add_message(BankMsg::Send {
                    to_address: ask.seller.to_string(),
                    amount: coins(ask.renewal_fund.u128(), NATIVE_DENOM),
                });
                ask.renewal_fund = Uint128::zero();
            }

            // The renewal fund is insufficient, sell to the highest bidder
            process_renewal_event = process_renewal_event.add_attribute("action", "sell");
            response = response.add_event(process_renewal_event);

            sell_name(deps, env, ask, bid, response)
        }
    } else {
        process_renewal_event = process_renewal_event.add_attribute("action", "renew");
        response = response.add_event(process_renewal_event);

        renew_name(deps, env, sudo_params, ask, Uint128::zero(), response)
    }
}
