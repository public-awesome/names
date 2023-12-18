use std::cmp::max;

use crate::{
    execute::{finalize_sale, store_ask},
    hooks::prepare_ask_hook,
    msg::{ExecuteMsg, HookAction, QueryMsg},
    state::{ask_key, asks, bid_key, bids, Ask, Bid, SudoParams, RENEWAL_QUEUE},
    ContractError,
};
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{
    coins, ensure, to_binary, Addr, BankMsg, Decimal, Deps, DepsMut, Env, Event, Order,
    QuerierWrapper, QueryRequest, StdError, StdResult, Timestamp, Uint128, WasmMsg, WasmQuery,
};
use cw721::Cw721ExecuteMsg;
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
) -> Result<Option<Bid>, ContractError> {
    let min_time = block_time.seconds() - sudo_params.valid_bid_seconds_delta;

    let bid = bids()
        .idx
        .price
        .range(deps.storage, None, None, Order::Descending)
        .take(sudo_params.valid_bid_query_limit as usize)
        .filter_map(|item| {
            item.map_or(None, |(_, bid)| {
                if bid.created_time.seconds() <= min_time {
                    Some(bid)
                } else {
                    None
                }
            })
        })
        .next();

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
pub fn get_renewal_price(
    base_price: u128,
    name_len: usize,
    valid_bid: Option<&Bid>,
    renewal_bid_percentage: Decimal,
) -> Uint128 {
    let renewal_char_price = get_char_price(base_price, name_len);
    let renewal_bid_price = valid_bid
        .as_ref()
        .map_or(Uint128::zero(), |bid| bid.amount * renewal_bid_percentage);
    max(renewal_char_price, renewal_bid_price)
}

fn renew_name(
    deps: DepsMut,
    env: &Env,
    sudo_params: &SudoParams,
    ask: Ask,
    renewal_price: Uint128,
    mut response: Response,
) -> Result<Response, ContractError> {
    // Take renewal payment
    charge_fees(
        &mut response,
        sudo_params.trading_fee_percent,
        renewal_price,
    );

    let next_renewal_time = env.block.time.plus_seconds(SECONDS_PER_YEAR);

    // Update renewal time
    RENEWAL_QUEUE.save(
        deps.storage,
        (next_renewal_time.seconds(), ask.id),
        &ask.token_id.to_string(),
    )?;

    // Update Ask with new renewal time
    let next_renewal_fund = ask.renewal_fund - renewal_price;
    let ask = Ask {
        token_id: ask.token_id.to_string(),
        id: ask.id,
        seller: ask.seller,
        renewal_time: next_renewal_time,
        renewal_fund: next_renewal_fund,
    };
    store_ask(deps.storage, &ask)?;

    Ok(response)
}

fn sell_name(
    deps: DepsMut,
    env: &Env,
    ask: Ask,
    bid: Bid,
    mut response: Response,
) -> Result<Response, ContractError> {
    // Remove accepted bid
    bids().remove(deps.storage, bid_key(&ask.token_id, &bid.bidder))?;

    let next_renewal_time = env.block.time.plus_seconds(SECONDS_PER_YEAR);

    // Update renewal queue
    RENEWAL_QUEUE.save(
        deps.storage,
        (next_renewal_time.seconds(), ask.id),
        &ask.token_id.to_string(),
    )?;

    // Transfer funds and NFT
    finalize_sale(
        deps.as_ref(),
        ask.clone(),
        bid.amount,
        bid.bidder.clone(),
        &mut response,
    )?;

    // Update Ask with new seller and renewal time
    let ask = Ask {
        token_id: ask.token_id.to_string(),
        id: ask.id,
        seller: bid.bidder.clone(),
        renewal_time: next_renewal_time,
        renewal_fund: Uint128::zero(),
    };
    store_ask(deps.storage, &ask)?;

    Ok(response)
}

fn burn_name(
    deps: DepsMut,
    collection: &Addr,
    ask: Ask,
    mut response: Response,
) -> Result<Response, ContractError> {
    response = response.add_message(WasmMsg::Execute {
        contract_addr: collection.to_string(),
        msg: to_binary(&Cw721ExecuteMsg::Burn {
            token_id: ask.token_id.to_string(),
        })?,
        funds: vec![],
    });

    // Delete ask
    asks().remove(deps.storage, ask_key(&ask.token_id))?;

    let hook = prepare_ask_hook(deps.as_ref(), &ask, HookAction::Delete)?;
    let event = Event::new("remove-ask").add_attribute("token_id", ask.token_id);

    response = response.add_submessages(hook).add_event(event);

    Ok(response)
}

pub fn process_renewal(
    deps: DepsMut,
    env: &Env,
    sudo_params: &SudoParams,
    name_minter_params: &NameMinterParams,
    collection: &Addr,
    ask: Ask,
    mut response: Response,
) -> Result<Response, ContractError> {
    ensure!(
        ask.renewal_time.seconds() <= env.block.time.seconds(),
        ContractError::CannotProcessFutureRenewal {}
    );

    let mut process_renewal_event = Event::new("process-renewal")
        .add_attribute("token_id", ask.token_id.to_string())
        .add_attribute("renewal_time", ask.renewal_time.seconds().to_string());

    let valid_bid = find_valid_bid(deps.as_ref(), &env.block.time, sudo_params)?;

    // Renewal price is the max of the char based price and a percentage of highest valid bid
    let renewal_price = get_renewal_price(
        name_minter_params.base_price.u128(),
        ask.token_id.len(),
        valid_bid.as_ref(),
        sudo_params.renewal_bid_percentage,
    );

    // If the renewal fund is sufficient, renew it
    if ask.renewal_fund >= renewal_price {
        process_renewal_event = process_renewal_event.add_attribute("action", "renew");
        response = response.add_event(process_renewal_event);

        return renew_name(deps, env, sudo_params, ask, renewal_price, response);
    }

    // Renewal fund is insufficient, send it back to the owner
    if !ask.renewal_fund.is_zero() {
        response = response.add_message(BankMsg::Send {
            to_address: ask.seller.to_string(),
            amount: coins(ask.renewal_fund.u128(), NATIVE_DENOM),
        });
    }

    if let Some(bid) = valid_bid {
        // The renewal fund is insufficient, sell to the highest bidder
        process_renewal_event = process_renewal_event.add_attribute("action", "sell");
        response = response.add_event(process_renewal_event);

        sell_name(deps, env, ask, bid, response)
    } else {
        // The renewal fund is insufficient, and there is no valid bid, burn it
        process_renewal_event = process_renewal_event.add_attribute("action", "burn");
        response = response.add_event(process_renewal_event);

        burn_name(deps, collection, ask, response)
    }
}
