use crate::msg::{
    AskCountResponse, AskOffset, AskResponse, AsksResponse, BidOffset, BidResponse, Bidder,
    BidsResponse, Collection, CollectionBidOffset, CollectionOffset, ParamsResponse, QueryMsg,
};
use crate::state::{
    ask_key, asks, bid_key, bids, BidKey, TokenId, ASK_HOOKS, BID_HOOKS, SALE_HOOKS, SUDO_PARAMS,
};
use cosmwasm_std::{entry_point, to_binary, Addr, Binary, Deps, Env, Order, StdResult};
use cw_storage_plus::{Bound, PrefixBound};
use cw_utils::maybe_addr;

// Query limits
const DEFAULT_QUERY_LIMIT: u32 = 10;
const MAX_QUERY_LIMIT: u32 = 100;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    let api = deps.api;

    match msg {
        QueryMsg::Ask { token_id } => to_binary(&query_ask(deps, token_id)?),
        QueryMsg::Asks {
            include_inactive,
            start_after,
            limit,
        } => to_binary(&query_asks(deps, include_inactive, start_after, limit)?),
        QueryMsg::ReverseAsks {
            include_inactive,
            start_before,
            limit,
        } => to_binary(&reverse_query_asks(
            deps,
            include_inactive,
            start_before,
            limit,
        )?),
        QueryMsg::AsksBySeller {
            seller,
            include_inactive,
            start_after,
            limit,
        } => to_binary(&query_asks_by_seller(
            deps,
            api.addr_validate(&seller)?,
            include_inactive,
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
        QueryMsg::Params {} => to_binary(&query_params(deps)?),
        QueryMsg::BidsByBidderSortedByExpiration {
            bidder,
            start_after,
            limit,
        } => todo!(),
        QueryMsg::AskHooks {} => todo!(),
        QueryMsg::BidHooks {} => todo!(),
        QueryMsg::SaleHooks {} => todo!(),
    }
}

pub fn query_asks(
    deps: Deps,
    include_inactive: Option<bool>,
    start_after: Option<TokenId>,
    limit: Option<u32>,
) -> StdResult<AsksResponse> {
    let limit = limit.unwrap_or(DEFAULT_QUERY_LIMIT).min(MAX_QUERY_LIMIT) as usize;

    // let asks = asks()
    //     // .idx
    //     // .collection
    //     // .prefix(collection.clone())
    //     .range(
    //         deps.storage,
    //         Some(Bound::exclusive((
    //             collection,
    //             start_after.unwrap_or_default(),
    //         ))),
    //         None,
    //         Order::Ascending,
    //     )
    //     .filter(|item| match item {
    //         Ok((_, ask)) => match include_inactive {
    //             Some(true) => true,
    //             _ => ask.is_active,
    //         },
    //         Err(_) => true,
    //     })
    //     .take(limit)
    //     .map(|res| res.map(|item| item.1))
    //     .collect::<StdResult<Vec<_>>>()?;

    let asks = asks()
        .range(deps.storage, None, None, Order::Ascending)
        .collect::<StdResult<Vec<_>>>()?;

    Ok(AsksResponse { asks: vec![] })
}

pub fn reverse_query_asks(
    deps: Deps,
    include_inactive: Option<bool>,
    start_before: Option<TokenId>,
    limit: Option<u32>,
) -> StdResult<AsksResponse> {
    let limit = limit.unwrap_or(DEFAULT_QUERY_LIMIT).min(MAX_QUERY_LIMIT) as usize;

    // let asks = asks()
    //     .idx
    //     .collection
    //     .prefix(collection.clone())
    //     .range(
    //         deps.storage,
    //         None,
    //         Some(Bound::exclusive((
    //             collection,
    //             start_before.unwrap_or_default(),
    //         ))),
    //         Order::Descending,
    //     )
    //     .filter(|item| match item {
    //         Ok((_, ask)) => match include_inactive {
    //             Some(true) => true,
    //             _ => ask.is_active,
    //         },
    //         Err(_) => true,
    //     })
    //     .take(limit)
    //     .map(|res| res.map(|item| item.1))
    //     .collect::<StdResult<Vec<_>>>()?;

    Ok(AsksResponse { asks: vec![] })
}

pub fn query_asks_sorted_by_price(
    deps: Deps,
    include_inactive: Option<bool>,
    start_after: Option<AskOffset>,
    limit: Option<u32>,
) -> StdResult<AsksResponse> {
    let limit = limit.unwrap_or(DEFAULT_QUERY_LIMIT).min(MAX_QUERY_LIMIT) as usize;

    // let start =
    //     start_after.map(|offset| Bound::exclusive((offset.price.u128(), ask_key(offset.token_id))));

    // let asks = asks()
    //     .idx
    //     .collection_price
    //     .sub_prefix(collection)
    //     .range(deps.storage, start, None, Order::Ascending)
    //     .filter(|item| match item {
    //         Ok((_, ask)) => match include_inactive {
    //             Some(true) => true,
    //             _ => ask.is_active,
    //         },
    //         Err(_) => true,
    //     })
    //     .take(limit)
    //     .map(|res| res.map(|item| item.1))
    //     .collect::<StdResult<Vec<_>>>()?;

    Ok(AsksResponse { asks: vec![] })
}

pub fn reverse_query_asks_sorted_by_price(
    deps: Deps,
    include_inactive: Option<bool>,
    start_before: Option<AskOffset>,
    limit: Option<u32>,
) -> StdResult<AsksResponse> {
    let limit = limit.unwrap_or(DEFAULT_QUERY_LIMIT).min(MAX_QUERY_LIMIT) as usize;

    // let end = start_before
    //     .map(|offset| Bound::exclusive((offset.price.u128(), ask_key(offset.token_id))));

    // let asks = asks()
    //     .idx
    //     .collection_price
    //     .sub_prefix(collection)
    //     .range(deps.storage, None, end, Order::Descending)
    //     .filter(|item| match item {
    //         Ok((_, ask)) => match include_inactive {
    //             Some(true) => true,
    //             _ => ask.is_active,
    //         },
    //         Err(_) => true,
    //     })
    //     .take(limit)
    //     .map(|res| res.map(|item| item.1))
    //     .collect::<StdResult<Vec<_>>>()?;

    Ok(AsksResponse { asks: vec![] })
}

pub fn query_ask_count(deps: Deps) -> StdResult<AskCountResponse> {
    // let count = asks()
    //     .idx
    //     .collection
    //     .prefix(collection)
    //     .keys_raw(deps.storage, None, None, Order::Ascending)
    //     .count() as u32;

    Ok(AskCountResponse { count: 7 })
}

pub fn query_asks_by_seller(
    deps: Deps,
    seller: Addr,
    include_inactive: Option<bool>,
    start_after: Option<CollectionOffset>,
    limit: Option<u32>,
) -> StdResult<AsksResponse> {
    let limit = limit.unwrap_or(DEFAULT_QUERY_LIMIT).min(MAX_QUERY_LIMIT) as usize;

    let start = if let Some(start) = start_after {
        let collection = deps.api.addr_validate(&start.collection)?;
        Some(Bound::exclusive(ask_key(start.token_id)))
    } else {
        None
    };

    let asks = asks()
        .idx
        .seller
        .prefix(seller)
        .range(deps.storage, start, None, Order::Ascending)
        .filter(|item| match item {
            Ok((_, ask)) => match include_inactive {
                Some(true) => true,
                _ => false,
            },
            Err(_) => true,
        })
        .take(limit)
        .map(|res| res.map(|item| item.1))
        .collect::<StdResult<Vec<_>>>()?;

    Ok(AsksResponse { asks })
}

pub fn query_ask(deps: Deps, token_id: TokenId) -> StdResult<AskResponse> {
    let ask = asks().may_load(deps.storage, ask_key(token_id))?;

    Ok(AskResponse { ask })
}

pub fn query_bid(deps: Deps, token_id: TokenId, bidder: Addr) -> StdResult<BidResponse> {
    let bid = bids().may_load(deps.storage, (token_id, bidder))?;

    Ok(BidResponse { bid })
}

pub fn query_bids_by_bidder(
    deps: Deps,
    bidder: Addr,
    start_after: Option<CollectionOffset>,
    limit: Option<u32>,
) -> StdResult<BidsResponse> {
    let limit = limit.unwrap_or(DEFAULT_QUERY_LIMIT).min(MAX_QUERY_LIMIT) as usize;

    let start = if let Some(start) = start_after {
        let collection = deps.api.addr_validate(&start.collection)?;
        Some(Bound::exclusive(bid_key(start.token_id, &bidder)))
    } else {
        None
    };

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

pub fn query_bids(
    deps: Deps,
    token_id: TokenId,
    start_after: Option<Bidder>,
    limit: Option<u32>,
) -> StdResult<BidsResponse> {
    let limit = limit.unwrap_or(DEFAULT_QUERY_LIMIT).min(MAX_QUERY_LIMIT) as usize;
    // let start = start_after.map(|s| Bound::ExclusiveRaw(s.into()));

    // let bids = bids()
    //     .idx
    //     .collection_token_id
    //     .prefix((collection, token_id))
    //     .range(deps.storage, start, None, Order::Ascending)
    //     .take(limit)
    //     .map(|item| item.map(|(_, b)| b))
    //     .collect::<StdResult<Vec<_>>>()?;

    Ok(BidsResponse { bids: vec![] })
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
            bid_key(offset.token_id, &offset.bidder),
        ))
    });

    // let bids = bids()
    //     .idx
    //     .collection_price
    //     .sub_prefix(collection)
    //     .range(deps.storage, start, None, Order::Ascending)
    //     .take(limit)
    //     .map(|item| item.map(|(_, b)| b))
    //     .collect::<StdResult<Vec<_>>>()?;

    Ok(BidsResponse { bids: vec![] })
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
            bid_key(offset.token_id, &offset.bidder),
        ))
    });

    // let bids = bids()
    //     .idx
    //     .collection_price
    //     .sub_prefix(collection)
    //     .range(deps.storage, None, end, Order::Descending)
    //     .take(limit)
    //     .map(|item| item.map(|(_, b)| b))
    //     .collect::<StdResult<Vec<_>>>()?;

    Ok(BidsResponse { bids: vec![] })
}

// pub fn query_bids_by_bidder_sorted_by_expiry(
//     deps: Deps,
//     bidder: Addr,
//     start_after: Option<CollectionOffset>,
//     limit: Option<u32>,
// ) -> StdResult<BidsResponse> {
//     let limit = limit.unwrap_or(DEFAULT_QUERY_LIMIT).min(MAX_QUERY_LIMIT) as usize;

//     let start = match start_after {
//         Some(offset) => {
//             let collection = deps.api.addr_validate(&offset.collection)?;
//             let bid = query_bid(deps, offset.token_id, bidder.clone())?;
//             match bid.bid {
//                 Some(bid) => Some(Bound::exclusive((
//                     bid.expires_at.seconds(),
//                     bid_key(offset.token_id, &bidder),
//                 ))),
//                 None => None,
//             }
//         }
//         None => None,
//     };

//     let bids = bids()
//         .idx
//         .bidder_expires_at
//         .sub_prefix(bidder)
//         .range(deps.storage, start, None, Order::Ascending)
//         .take(limit)
//         .map(|item| item.map(|(_, b)| b))
//         .collect::<StdResult<Vec<_>>>()?;

//     Ok(BidsResponse { bids })
// }

pub fn query_params(deps: Deps) -> StdResult<ParamsResponse> {
    let config = SUDO_PARAMS.load(deps.storage)?;

    Ok(ParamsResponse { params: config })
}
