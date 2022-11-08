use crate::msg::{
    AskCountResponse, AskResponse, AsksResponse, BidOffset, BidResponse, Bidder, BidsResponse,
    ConfigResponse, ParamsResponse, QueryMsg,
};
use crate::state::{
    ask_key, asks, bid_key, bids, BidKey, Id, TokenId, ASK_HOOKS, BID_HOOKS, NAME_COLLECTION,
    NAME_MINTER, RENEWAL_QUEUE, SALE_HOOKS, SUDO_PARAMS,
};
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_binary, Addr, Binary, Deps, Env, Order, StdResult, Timestamp};
use cw_storage_plus::Bound;

// Query limits
const DEFAULT_QUERY_LIMIT: u32 = 10;
const MAX_QUERY_LIMIT: u32 = 100;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    let api = deps.api;

    match msg {
        QueryMsg::Ask { token_id } => to_binary(&query_ask(deps, token_id)?),
        QueryMsg::Asks { start_after, limit } => to_binary(&query_asks(deps, start_after, limit)?),
        QueryMsg::ReverseAsks {
            start_before,
            limit,
        } => to_binary(&reverse_query_asks(deps, start_before, limit)?),
        QueryMsg::AsksBySeller {
            seller,
            start_after,
            limit,
        } => to_binary(&query_asks_by_seller(
            deps,
            api.addr_validate(&seller)?,
            start_after,
            limit,
        )?),
        QueryMsg::AskCount {} => to_binary(&query_ask_count(deps)?),
        QueryMsg::Bid { token_id, bidder } => {
            to_binary(&query_bid(deps, token_id, api.addr_validate(&bidder)?)?)
        }
        QueryMsg::Bids {
            token_id,
            start_after,
            limit,
        } => to_binary(&query_bids(deps, token_id, start_after, limit)?),
        QueryMsg::BidsByBidder {
            bidder,
            start_after,
            limit,
        } => to_binary(&query_bids_by_bidder(
            deps,
            api.addr_validate(&bidder)?,
            start_after,
            limit,
        )?),
        QueryMsg::BidsSortedByPrice { start_after, limit } => {
            to_binary(&query_bids_sorted_by_price(deps, start_after, limit)?)
        }
        QueryMsg::ReverseBidsSortedByPrice {
            start_before,
            limit,
        } => to_binary(&reverse_query_bids_sorted_by_price(
            deps,
            start_before,
            limit,
        )?),
        QueryMsg::BidsBySeller { seller } => {
            to_binary(&query_bids_by_seller(deps, api.addr_validate(&seller)?)?)
        }
        QueryMsg::HighestBid { token_id } => to_binary(&query_highest_bid(deps, token_id)?),
        QueryMsg::Params {} => to_binary(&query_params(deps)?),
        QueryMsg::AskHooks {} => to_binary(&ASK_HOOKS.query_hooks(deps)?),
        QueryMsg::BidHooks {} => to_binary(&BID_HOOKS.query_hooks(deps)?),
        QueryMsg::SaleHooks {} => to_binary(&SALE_HOOKS.query_hooks(deps)?),
        QueryMsg::RenewalQueue { time } => to_binary(&query_renewal_queue(deps, time)?),
        QueryMsg::Config {} => to_binary(&query_config(deps)?),
    }
}

pub fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let minter = NAME_MINTER.load(deps.storage)?;
    let collection = NAME_COLLECTION.load(deps.storage)?;

    Ok(ConfigResponse { minter, collection })
}

pub fn query_renewal_queue(deps: Deps, time: Timestamp) -> StdResult<AsksResponse> {
    let names = RENEWAL_QUEUE
        .prefix(time.seconds())
        .range(deps.storage, None, None, Order::Ascending)
        .map(|item| item.map(|item| item.1))
        .collect::<StdResult<Vec<_>>>()?;

    let asks = names
        .iter()
        .map(|name| asks().load(deps.storage, ask_key(name)))
        .collect::<StdResult<Vec<_>>>()?;

    Ok(AsksResponse { asks })
}

pub fn query_asks(
    deps: Deps,
    start_after: Option<Id>,
    limit: Option<u32>,
) -> StdResult<AsksResponse> {
    let limit = limit.unwrap_or(DEFAULT_QUERY_LIMIT).min(MAX_QUERY_LIMIT) as usize;

    let asks = asks()
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
        .collect::<StdResult<Vec<_>>>()?;

    Ok(AsksResponse { asks })
}

pub fn reverse_query_asks(
    deps: Deps,
    start_before: Option<Id>,
    limit: Option<u32>,
) -> StdResult<AsksResponse> {
    let limit = limit.unwrap_or(DEFAULT_QUERY_LIMIT).min(MAX_QUERY_LIMIT) as usize;

    let start = start_before.unwrap_or(
        (asks()
            .keys_raw(deps.storage, None, None, Order::Ascending)
            .count()
            + 1) as u64,
    );

    let asks = asks()
        .idx
        .id
        .range(
            deps.storage,
            None,
            Some(Bound::exclusive(start)),
            Order::Descending,
        )
        .take(limit)
        .map(|res| res.map(|item| item.1))
        .collect::<StdResult<Vec<_>>>()?;

    Ok(AsksResponse { asks })
}

pub fn query_ask_count(deps: Deps) -> StdResult<AskCountResponse> {
    let count = asks()
        .keys_raw(deps.storage, None, None, Order::Ascending)
        .count() as u32;

    Ok(AskCountResponse { count })
}

// TODO: figure out how to paginate by `Id` instead of `TokenId`
pub fn query_asks_by_seller(
    deps: Deps,
    seller: Addr,
    start_after: Option<TokenId>,
    limit: Option<u32>,
) -> StdResult<AsksResponse> {
    let limit = limit.unwrap_or(DEFAULT_QUERY_LIMIT).min(MAX_QUERY_LIMIT) as usize;

    let start = start_after.map(|start| Bound::exclusive(ask_key(&start)));

    let asks = asks()
        .idx
        .seller
        .prefix(seller)
        .range(deps.storage, start, None, Order::Ascending)
        .take(limit)
        .map(|res| res.map(|item| item.1))
        .collect::<StdResult<Vec<_>>>()?;

    Ok(AsksResponse { asks })
}

pub fn query_ask(deps: Deps, token_id: TokenId) -> StdResult<AskResponse> {
    let ask = asks().may_load(deps.storage, ask_key(&token_id))?;

    Ok(AskResponse { ask })
}

pub fn query_bid(deps: Deps, token_id: TokenId, bidder: Addr) -> StdResult<BidResponse> {
    let bid = bids().may_load(deps.storage, (token_id, bidder))?;

    Ok(BidResponse { bid })
}

pub fn query_bids_by_bidder(
    deps: Deps,
    bidder: Addr,
    start_after: Option<TokenId>,
    limit: Option<u32>,
) -> StdResult<BidsResponse> {
    let limit = limit.unwrap_or(DEFAULT_QUERY_LIMIT).min(MAX_QUERY_LIMIT) as usize;

    let start = start_after.map(|start| Bound::exclusive(bid_key(&start, &bidder)));

    let bids = bids()
        .idx
        .bidder
        .prefix(bidder)
        .range(deps.storage, start, None, Order::Ascending)
        .take(limit)
        .map(|item| item.map(|(_, b)| b))
        .collect::<StdResult<Vec<_>>>()?;

    Ok(BidsResponse { bids })
}

pub fn query_bids_by_seller(deps: Deps, seller: Addr) -> StdResult<BidsResponse> {
    let bids = asks()
        .idx
        .seller
        .prefix(seller)
        .range(deps.storage, None, None, Order::Ascending)
        .map(|res| res.map(|item| item.0).unwrap())
        .flat_map(|token_id| {
            bids()
                .prefix(token_id)
                .range(deps.storage, None, None, Order::Ascending)
                .flat_map(|item| item.map(|(_, b)| b))
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();

    Ok(BidsResponse { bids })
}

pub fn query_bids(
    deps: Deps,
    token_id: TokenId,
    start_after: Option<Bidder>,
    limit: Option<u32>,
) -> StdResult<BidsResponse> {
    let limit = limit.unwrap_or(DEFAULT_QUERY_LIMIT).min(MAX_QUERY_LIMIT) as usize;
    let start = start_after.map(|s| Bound::ExclusiveRaw(s.into()));

    let bids = bids()
        .prefix(token_id)
        .range(deps.storage, start, None, Order::Ascending)
        .take(limit)
        .map(|item| item.map(|(_, b)| b))
        .collect::<StdResult<Vec<_>>>()?;

    Ok(BidsResponse { bids })
}

pub fn query_highest_bid(deps: Deps, token_id: TokenId) -> StdResult<BidResponse> {
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

    Ok(BidResponse { bid })
}

pub fn query_bids_sorted_by_price(
    deps: Deps,
    start_after: Option<BidOffset>,
    limit: Option<u32>,
) -> StdResult<BidsResponse> {
    let limit = limit.unwrap_or(DEFAULT_QUERY_LIMIT).min(MAX_QUERY_LIMIT) as usize;

    let start: Option<Bound<(u128, BidKey)>> = start_after.map(|offset| {
        Bound::exclusive((
            offset.price.u128(),
            bid_key(&offset.token_id, &offset.bidder),
        ))
    });

    let bids = bids()
        .idx
        .price
        .range(deps.storage, start, None, Order::Ascending)
        .take(limit)
        .map(|item| item.map(|(_, b)| b))
        .collect::<StdResult<Vec<_>>>()?;

    Ok(BidsResponse { bids })
}

pub fn reverse_query_bids_sorted_by_price(
    deps: Deps,
    start_before: Option<BidOffset>,
    limit: Option<u32>,
) -> StdResult<BidsResponse> {
    let limit = limit.unwrap_or(DEFAULT_QUERY_LIMIT).min(MAX_QUERY_LIMIT) as usize;

    let end: Option<Bound<(u128, BidKey)>> = start_before.map(|offset| {
        Bound::exclusive((
            offset.price.u128(),
            bid_key(&offset.token_id, &offset.bidder),
        ))
    });

    let bids = bids()
        .idx
        .price
        .range(deps.storage, None, end, Order::Descending)
        .take(limit)
        .map(|item| item.map(|(_, b)| b))
        .collect::<StdResult<Vec<_>>>()?;

    Ok(BidsResponse { bids })
}

pub fn query_params(deps: Deps) -> StdResult<ParamsResponse> {
    let config = SUDO_PARAMS.load(deps.storage)?;

    Ok(ParamsResponse { params: config })
}
