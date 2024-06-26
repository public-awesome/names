use std::str::FromStr;

use crate::execute::store_ask;
#[cfg(test)]
use crate::execute::{execute, instantiate};
use crate::msg::{ExecuteMsg, InstantiateMsg};
use crate::query::{query_asks_by_renew_time, query_asks_by_seller, query_bids_by_bidder};
use crate::state::{ask_key, asks, bid_key, bids, Ask, Bid};

use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
use cosmwasm_std::{coins, Addr, Decimal, DepsMut, Timestamp, Uint128};
use sg_std::NATIVE_DENOM;

const CREATOR: &str = "creator";
const OPERATOR: &str = "operator";
const TOKEN_ID: &str = "name";
const TOKEN_ID_NEXT: &str = "name2";

// Governance parameters
const TRADING_FEE_BASIS_POINTS: u64 = 200; // 2%

#[test]
fn ask_indexed_map() {
    let mut deps = mock_dependencies();
    let seller = Addr::unchecked("seller");

    let env = mock_env();

    let ask = Ask {
        token_id: TOKEN_ID.to_string(),
        id: 1,
        seller: seller.clone(),
        renewal_time: env.block.time,
        renewal_fund: Uint128::zero(),
    };
    let key = ask_key(TOKEN_ID);
    let res = asks().save(deps.as_mut().storage, key.clone(), &ask);
    assert!(res.is_ok());

    let ask2 = Ask {
        token_id: TOKEN_ID_NEXT.to_string(),
        id: 2,
        seller: seller.clone(),
        renewal_time: env.block.time,
        renewal_fund: Uint128::zero(),
    };
    let key2 = ask_key(TOKEN_ID_NEXT);
    let res = asks().save(deps.as_mut().storage, key2, &ask2);
    assert!(res.is_ok());

    let res = asks().load(deps.as_ref().storage, key);
    assert_eq!(res.unwrap(), ask);

    let res = query_asks_by_seller(deps.as_ref(), seller, None, None).unwrap();
    assert_eq!(res.len(), 2);
    assert_eq!(res[0], ask);
}

#[test]
fn bid_indexed_map() {
    let mut deps = mock_dependencies();
    let bidder = Addr::unchecked("bidder");

    let bid = Bid {
        token_id: TOKEN_ID.to_string(),
        bidder: bidder.clone(),
        amount: Uint128::from(500u128),
        created_time: Timestamp::from_seconds(6),
    };
    let key = bid_key(TOKEN_ID, &bidder);
    let res = bids().save(deps.as_mut().storage, key.clone(), &bid);
    assert!(res.is_ok());

    let bid2 = Bid {
        token_id: TOKEN_ID_NEXT.to_string(),
        bidder: bidder.clone(),
        amount: Uint128::from(500u128),
        created_time: Timestamp::from_seconds(6),
    };
    let key2 = bid_key(TOKEN_ID_NEXT, &bidder);
    let res = bids().save(deps.as_mut().storage, key2, &bid2);
    assert!(res.is_ok());

    let res = bids().load(deps.as_ref().storage, key);
    assert_eq!(res.unwrap(), bid);

    let res = query_bids_by_bidder(deps.as_ref(), bidder.clone(), None, None).unwrap();
    assert_eq!(res.len(), 2);
    assert_eq!(res[0], bid);

    let remove_bid_msg = ExecuteMsg::RemoveBid {
        token_id: TOKEN_ID_NEXT.to_string(),
    };
    let res = execute(
        deps.as_mut(),
        mock_env(),
        mock_info(bidder.as_ref(), &[]),
        remove_bid_msg,
    );
    assert!(res.is_ok());

    let res = query_bids_by_bidder(deps.as_ref(), bidder, None, None).unwrap();
    assert_eq!(res.len(), 1);
    assert_eq!(res[0], bid);
}

fn setup_contract(deps: DepsMut) {
    let msg = InstantiateMsg {
        trading_fee_bps: TRADING_FEE_BASIS_POINTS,
        min_price: Uint128::from(5u128),
        ask_interval: 60,
        max_renewals_per_block: 20,
        valid_bid_query_limit: 10,
        renew_window: 60 * 60 * 24 * 30,
        renewal_bid_percentage: Decimal::from_str("0.005").unwrap(),
        operator: OPERATOR.to_string(),
    };
    let info = mock_info(CREATOR, &[]);
    let res = instantiate(deps, mock_env(), info, msg).unwrap();
    assert_eq!(0, res.messages.len());
}

#[test]
fn proper_initialization() {
    let mut deps = mock_dependencies();

    let msg = InstantiateMsg {
        trading_fee_bps: TRADING_FEE_BASIS_POINTS,
        min_price: Uint128::from(5u128),
        ask_interval: 60,
        max_renewals_per_block: 20,
        valid_bid_query_limit: 10,
        renew_window: 60 * 60 * 24 * 30,
        renewal_bid_percentage: Decimal::from_str("0.005").unwrap(),
        operator: OPERATOR.to_string(),
    };
    let info = mock_info("creator", &coins(1000, NATIVE_DENOM));

    // we can just call .unwrap() to assert this was a success
    let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
    assert_eq!(0, res.messages.len());
}

#[test]
fn bad_fees_initialization() {
    let mut deps = mock_dependencies();

    // throw error if trading fee bps > 100%
    let msg = InstantiateMsg {
        trading_fee_bps: 10001,
        min_price: Uint128::from(5u128),
        ask_interval: 60,
        max_renewals_per_block: 20,
        valid_bid_query_limit: 10,
        renew_window: 60 * 60 * 24 * 30,
        renewal_bid_percentage: Decimal::from_str("0.005").unwrap(),
        operator: OPERATOR.to_string(),
    };
    let info = mock_info("creator", &coins(1000, NATIVE_DENOM));
    let res = instantiate(deps.as_mut(), mock_env(), info, msg);
    assert!(res.is_err());
}

#[test]
fn try_set_bid() {
    let mut deps = mock_dependencies();
    setup_contract(deps.as_mut());

    let bidder = mock_info("bidder", &coins(1000, NATIVE_DENOM));

    // Bidder calls SetBid before an Ask is set, fails
    let set_bid_msg = ExecuteMsg::SetBid {
        token_id: TOKEN_ID.to_string(),
    };
    let res = execute(deps.as_mut(), mock_env(), bidder, set_bid_msg);
    assert!(res.is_err());
}

#[test]
fn try_set_ask() {
    let mut deps = mock_dependencies();
    setup_contract(deps.as_mut());

    let set_ask = ExecuteMsg::SetAsk {
        token_id: TOKEN_ID.to_string(),
        seller: CREATOR.to_string(),
    };

    // Reject if not called by the media owner
    let not_allowed = mock_info("random", &[]);
    let err = execute(deps.as_mut(), mock_env(), not_allowed, set_ask);
    assert!(err.is_err());
}

#[test]
fn try_query_asks_by_renew_time() {
    let env = mock_env();
    let mut deps = mock_dependencies();
    setup_contract(deps.as_mut());

    let seller = Addr::unchecked("seller");

    let five_days_in_seconds = 60 * 60 * 24 * 5;

    for n in 1..=10 {
        let token_id = format!("renew-name-{}", n);
        let ask = Ask {
            token_id: token_id.to_string(),
            id: n,
            seller: seller.clone(),
            renewal_time: env.block.time.plus_seconds(five_days_in_seconds * n),
            renewal_fund: Uint128::zero(),
        };
        let result = store_ask(&mut deps.storage, &ask);
        assert!(result.is_ok());
    }

    let start_after = env.block.time.plus_seconds(five_days_in_seconds * 2);
    let max_time = env.block.time.plus_seconds(five_days_in_seconds * 9);

    let result = query_asks_by_renew_time(deps.as_ref(), max_time, Some(start_after), None);
    assert!(result.is_ok());

    let asks = result.unwrap();
    assert_eq!(asks.len(), 7);
}

#[test]
fn try_migrate_bids_fails_with_no_bids() {
    let mut deps = mock_dependencies();
    setup_contract(deps.as_mut());

    let migrate_bids_msg = ExecuteMsg::MigrateBids { limit: 100 };
    let result = execute(
        deps.as_mut(),
        mock_env(),
        mock_info(OPERATOR, &[]),
        migrate_bids_msg,
    );
    assert!(result.is_err());
}
