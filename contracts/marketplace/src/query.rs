use crate::helpers::get_renewal_price_and_bid;
use crate::msg::{AskRenewPriceResponse, BidOffset, Bidder, ConfigResponse, QueryMsg};
use crate::state::{
    ask_key, asks, bid_key, bids, legacy_bids, Ask, AskKey, Bid, Id, SudoParams, TokenId,
    ASK_COUNT, ASK_HOOKS, BID_HOOKS, NAME_COLLECTION, NAME_MINTER, RENEWAL_QUEUE, SALE_HOOKS,
    SUDO_PARAMS,
};

use cosmwasm_std::{
    coin, to_json_binary, Addr, Binary, Coin, Deps, Env, Order, StdError, StdResult, Timestamp,
};
use cw_storage_plus::Bound;
use sg_name_minter::{SgNameMinterQueryMsg, SudoParams as NameMinterParams};

#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use sg_std::NATIVE_DENOM;

// Query limits
const DEFAULT_QUERY_LIMIT: u32 = 10;
const MAX_QUERY_LIMIT: u32 = 100;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    let api = deps.api;

    match msg {
        QueryMsg::Ask { token_id } => to_json_binary(&query_ask(deps, token_id)?),
        QueryMsg::Asks { start_after, limit } => {
            to_json_binary(&query_asks(deps, start_after, limit)?)
        }
        QueryMsg::AsksBySeller {
            seller,
            start_after,
            limit,
        } => to_json_binary(&query_asks_by_seller(
            deps,
            api.addr_validate(&seller)?,
            start_after,
            limit,
        )?),
        QueryMsg::AsksByRenewTime {
            max_time,
            start_after,
            limit,
        } => to_json_binary(&query_asks_by_renew_time(
            deps,
            max_time,
            start_after,
            limit,
        )?),
        QueryMsg::AskRenewPrice {
            current_time,
            token_id,
        } => to_json_binary(&query_ask_renew_price(deps, current_time, token_id)?),
        QueryMsg::AskCount {} => to_json_binary(&query_ask_count(deps)?),
        QueryMsg::AskRenewalPrices {
            current_time,
            token_ids,
        } => to_json_binary(&query_ask_renew_prices(deps, current_time, token_ids)?),
        QueryMsg::Bid { token_id, bidder } => {
            to_json_binary(&query_bid(deps, token_id, api.addr_validate(&bidder)?)?)
        }
        QueryMsg::Bids {
            token_id,
            start_after,
            limit,
        } => to_json_binary(&query_bids(deps, token_id, start_after, limit)?),
        QueryMsg::LegacyBids { start_after, limit } => {
            to_json_binary(&query_legacy_bids(deps, start_after, limit)?)
        }
        QueryMsg::BidsByBidder {
            bidder,
            start_after,
            limit,
        } => to_json_binary(&query_bids_by_bidder(
            deps,
            api.addr_validate(&bidder)?,
            start_after,
            limit,
        )?),
        QueryMsg::BidsSortedByPrice { start_after, limit } => {
            to_json_binary(&query_bids_sorted_by_price(deps, start_after, limit)?)
        }
        QueryMsg::ReverseBidsSortedByPrice {
            start_before,
            limit,
        } => to_json_binary(&reverse_query_bids_sorted_by_price(
            deps,
            start_before,
            limit,
        )?),
        QueryMsg::BidsForSeller {
            seller,
            start_after,
            limit,
        } => to_json_binary(&query_bids_for_seller(
            deps,
            api.addr_validate(&seller)?,
            start_after,
            limit,
        )?),
        QueryMsg::HighestBid { token_id } => to_json_binary(&query_highest_bid(deps, token_id)?),
        QueryMsg::Params {} => to_json_binary(&query_params(deps)?),
        QueryMsg::AskHooks {} => to_json_binary(&ASK_HOOKS.query_hooks(deps)?),
        QueryMsg::BidHooks {} => to_json_binary(&BID_HOOKS.query_hooks(deps)?),
        QueryMsg::SaleHooks {} => to_json_binary(&SALE_HOOKS.query_hooks(deps)?),
        QueryMsg::RenewalQueue { time } => to_json_binary(&query_renewal_queue(deps, time)?),
        QueryMsg::Config {} => to_json_binary(&query_config(deps)?),
    }
}

pub fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let minter = NAME_MINTER.load(deps.storage)?;
    let collection = NAME_COLLECTION.load(deps.storage)?;

    Ok(ConfigResponse { minter, collection })
}

pub fn query_renewal_queue(deps: Deps, time: Timestamp) -> StdResult<Vec<Ask>> {
    let names = RENEWAL_QUEUE
        .prefix(time.seconds())
        .range(deps.storage, None, None, Order::Ascending)
        .map(|item| item.map(|item| item.1))
        .collect::<StdResult<Vec<_>>>()?;

    names
        .iter()
        .map(|name| asks().load(deps.storage, ask_key(name)))
        .collect::<StdResult<Vec<_>>>()
}

pub fn query_asks(deps: Deps, start_after: Option<Id>, limit: Option<u32>) -> StdResult<Vec<Ask>> {
    let limit = limit.unwrap_or(DEFAULT_QUERY_LIMIT).min(MAX_QUERY_LIMIT) as usize;

    asks()
        .idx
        .id
        .range(
            deps.storage,
            Some(Bound::exclusive(start_after.unwrap_or_default())),
            None,
            Order::Ascending,
        )
        .take(limit)
        .map(|res| res.map(|item| item.1))
        .collect::<StdResult<Vec<_>>>()
}

pub fn query_ask_count(deps: Deps) -> StdResult<u64> {
    ASK_COUNT.load(deps.storage)
}

// TODO: figure out how to paginate by `Id` instead of `TokenId`
pub fn query_asks_by_seller(
    deps: Deps,
    seller: Addr,
    start_after: Option<TokenId>,
    limit: Option<u32>,
) -> StdResult<Vec<Ask>> {
    let limit = limit.unwrap_or(DEFAULT_QUERY_LIMIT).min(MAX_QUERY_LIMIT) as usize;

    let start = start_after.map(|start| Bound::exclusive(ask_key(&start)));

    asks()
        .idx
        .seller
        .prefix(seller)
        .range(deps.storage, start, None, Order::Ascending)
        .take(limit)
        .map(|res| res.map(|item| item.1))
        .collect::<StdResult<Vec<_>>>()
}

pub fn query_asks_by_renew_time(
    deps: Deps,
    max_time: Timestamp,
    start_after: Option<Timestamp>,
    limit: Option<u32>,
) -> StdResult<Vec<Ask>> {
    let limit = limit.unwrap_or(DEFAULT_QUERY_LIMIT) as usize;

    let renewable_asks = asks()
        .idx
        .renewal_time
        .range(
            deps.storage,
            start_after.map(|start| Bound::inclusive((start.seconds() + 1, "".to_string()))),
            Some(Bound::exclusive((max_time.seconds() + 1, "".to_string()))),
            Order::Ascending,
        )
        .take(limit)
        .map(|item| item.map(|(_, v)| v))
        .collect::<StdResult<Vec<_>>>()?;

    Ok(renewable_asks)
}

pub fn query_ask_renew_price(
    deps: Deps,
    current_time: Timestamp,
    token_id: String,
) -> StdResult<(Option<Coin>, Option<Bid>)> {
    let ask = asks().load(deps.storage, ask_key(&token_id))?;
    let sudo_params = SUDO_PARAMS.load(deps.storage)?;

    let ask_renew_start_time = ask.renewal_time.seconds() - sudo_params.renew_window;

    if current_time.seconds() < ask_renew_start_time {
        return Ok((None, None));
    }

    let name_minter = NAME_MINTER.load(deps.storage)?;
    let name_minter_params = deps
        .querier
        .query_wasm_smart::<NameMinterParams>(name_minter, &(SgNameMinterQueryMsg::Params {}))?;

    let (renewal_price, valid_bid) = get_renewal_price_and_bid(
        deps,
        &current_time,
        &sudo_params,
        &ask.token_id,
        name_minter_params.base_price.u128(),
    )
    .map_err(|_| StdError::generic_err("failed to fetch renewal price".to_string()))?;

    Ok((Some(coin(renewal_price.u128(), NATIVE_DENOM)), valid_bid))
}

pub fn query_ask_renew_prices(
    deps: Deps,
    current_time: Timestamp,
    token_ids: Vec<String>,
) -> StdResult<Vec<AskRenewPriceResponse>> {
    token_ids
        .iter()
        .map(|token_id| {
            let (coin_option, bid_option) =
                query_ask_renew_price(deps, current_time, token_id.to_string())?;
            Ok(AskRenewPriceResponse {
                token_id: token_id.to_string(),
                price: coin_option.unwrap_or_default(),
                bid: bid_option,
            })
        })
        .collect::<StdResult<Vec<_>>>()
}

pub fn query_ask(deps: Deps, token_id: TokenId) -> StdResult<Option<Ask>> {
    asks().may_load(deps.storage, ask_key(&token_id))
}

pub fn query_bid(deps: Deps, token_id: TokenId, bidder: Addr) -> StdResult<Option<Bid>> {
    bids().may_load(deps.storage, (token_id, bidder))
}

pub fn query_bids_by_bidder(
    deps: Deps,
    bidder: Addr,
    start_after: Option<TokenId>,
    limit: Option<u32>,
) -> StdResult<Vec<Bid>> {
    let limit = limit.unwrap_or(DEFAULT_QUERY_LIMIT).min(MAX_QUERY_LIMIT) as usize;

    let start = start_after.map(|start| Bound::exclusive((start, bidder.clone())));

    bids()
        .idx
        .bidder
        .prefix(bidder)
        .range(deps.storage, start, None, Order::Ascending)
        .take(limit)
        .map(|item| item.map(|(_, b)| b))
        .collect::<StdResult<Vec<_>>>()
}

pub fn query_bids_for_seller(
    deps: Deps,
    seller: Addr,
    start_after: Option<BidOffset>,
    limit: Option<u32>,
) -> StdResult<Vec<Bid>> {
    let limit = limit.unwrap_or(DEFAULT_QUERY_LIMIT).min(MAX_QUERY_LIMIT) as usize;
    // Query seller asks, then collect bids starting after token_id
    // Limitation: Can not collect bids in the middle using `start_after: token_id` pattern
    // This leads to imprecise pagination based on token id and not bid count
    let start_token_id =
        start_after.map(|start| Bound::<AskKey>::exclusive(ask_key(&start.token_id)));

    let bids = asks()
        .idx
        .seller
        .prefix(seller)
        .range(deps.storage, start_token_id, None, Order::Ascending)
        .take(limit)
        .map(|res| res.map(|item| item.0).unwrap())
        .flat_map(|token_id| {
            bids()
                .prefix(token_id)
                .range(deps.storage, None, None, Order::Ascending)
                .flat_map(|item| item.map(|(_, b)| b))
                .collect::<Vec<_>>()
        })
        .collect();

    Ok(bids)
}

pub fn query_bids(
    deps: Deps,
    token_id: TokenId,
    start_after: Option<Bidder>,
    limit: Option<u32>,
) -> StdResult<Vec<Bid>> {
    let limit = limit.unwrap_or(DEFAULT_QUERY_LIMIT).min(MAX_QUERY_LIMIT) as usize;
    let start = start_after.map(|s| Bound::ExclusiveRaw(s.into()));

    bids()
        .prefix(token_id)
        .range(deps.storage, start, None, Order::Ascending)
        .take(limit)
        .map(|item| item.map(|(_, b)| b))
        .collect::<StdResult<Vec<_>>>()
}

pub fn query_legacy_bids(
    deps: Deps,
    start_after: Option<BidOffset>,
    limit: Option<u32>,
) -> StdResult<Vec<Bid>> {
    let limit = limit.unwrap_or(DEFAULT_QUERY_LIMIT).min(MAX_QUERY_LIMIT) as usize;

    let start =
        start_after.map(|offset| Bound::exclusive(bid_key(&offset.token_id, &offset.bidder)));

    legacy_bids()
        .range(deps.storage, start, None, Order::Ascending)
        .take(limit)
        .map(|item| item.map(|(_, b)| b))
        .collect::<StdResult<Vec<_>>>()
}

pub fn query_highest_bid(deps: Deps, token_id: TokenId) -> StdResult<Option<Bid>> {
    let bid = bids()
        .idx
        .price
        .range(deps.storage, None, None, Order::Descending)
        .filter_map(|item| {
            let (key, bid) = item.unwrap();
            if key.0 == token_id {
                Some(bid)
            } else {
                None
            }
        })
        .take(1)
        .collect::<Vec<_>>()
        .first()
        .cloned();

    Ok(bid)
}

pub fn query_bids_sorted_by_price(
    deps: Deps,
    start_after: Option<BidOffset>,
    limit: Option<u32>,
) -> StdResult<Vec<Bid>> {
    let limit = limit.unwrap_or(DEFAULT_QUERY_LIMIT).min(MAX_QUERY_LIMIT) as usize;

    let start = start_after.map(|offset| {
        Bound::exclusive((
            (offset.token_id.clone(), offset.price.u128()),
            bid_key(&offset.token_id, &offset.bidder),
        ))
    });

    bids()
        .idx
        .price
        .range(deps.storage, start, None, Order::Ascending)
        .take(limit)
        .map(|item| item.map(|(_, b)| b))
        .collect::<StdResult<Vec<_>>>()
}

pub fn reverse_query_bids_sorted_by_price(
    deps: Deps,
    start_before: Option<BidOffset>,
    limit: Option<u32>,
) -> StdResult<Vec<Bid>> {
    let limit = limit.unwrap_or(DEFAULT_QUERY_LIMIT).min(MAX_QUERY_LIMIT) as usize;

    let end = start_before.map(|offset| {
        Bound::exclusive((
            (offset.token_id.clone(), offset.price.u128()),
            bid_key(&offset.token_id, &offset.bidder),
        ))
    });

    bids()
        .idx
        .price
        .range(deps.storage, None, end, Order::Descending)
        .take(limit)
        .map(|item| item.map(|(_, b)| b))
        .collect::<StdResult<Vec<_>>>()
}

pub fn query_params(deps: Deps) -> StdResult<SudoParams> {
    SUDO_PARAMS.load(deps.storage)
}
