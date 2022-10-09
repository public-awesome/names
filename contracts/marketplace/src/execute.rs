use std::marker::PhantomData;

use crate::error::ContractError;
use crate::hooks::{prepare_ask_hook, prepare_bid_hook, prepare_sale_hook};
use crate::msg::{ExecuteMsg, HookAction, InstantiateMsg};
use crate::state::{
    ask_key, asks, bid_key, bids, increment_asks, Ask, Bid, SudoParams, NAME_COLLECTION,
    NAME_MINTER, RENEWAL_QUEUE, SUDO_PARAMS,
};
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    coin, coins, to_binary, Addr, BankMsg, Decimal, Deps, DepsMut, Empty, Env, Event, MessageInfo,
    StdError, StdResult, Storage, Uint128, WasmMsg,
};
use cw2::set_contract_version;
use cw721::{Cw721ExecuteMsg, OwnerOfResponse};
use cw721_base::helpers::Cw721Contract;
use cw_utils::{may_pay, must_pay, nonpayable};
use sg_std::{create_fund_community_pool_msg, Response, SubMsg, NATIVE_DENOM};

// Version info for migration info
const CONTRACT_NAME: &str = "crates.io:sg-name-marketplace";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

// bps fee can not exceed 100%
const MAX_FEE_BPS: u64 = 10000;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    if msg.trading_fee_bps > MAX_FEE_BPS {
        return Err(ContractError::InvalidTradingFeeBps(msg.trading_fee_bps));
    }

    let params = SudoParams {
        trading_fee_percent: Decimal::percent(msg.trading_fee_bps),
        min_price: msg.min_price,
        blocks_per_year: msg.blocks_per_year,
    };
    SUDO_PARAMS.save(deps.storage, &params)?;

    Ok(Response::new())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    let api = deps.api;

    match msg {
        ExecuteMsg::SetAsk { token_id, seller } => {
            execute_set_ask(deps, env, info, &token_id, api.addr_validate(&seller)?)
        }
        ExecuteMsg::SetBid { token_id } => execute_set_bid(deps, env, info, &token_id),
        ExecuteMsg::RemoveBid { token_id } => execute_remove_bid(deps, env, info, &token_id),
        ExecuteMsg::AcceptBid { token_id, bidder } => {
            execute_accept_bid(deps, env, info, &token_id, api.addr_validate(&bidder)?)
        }
        ExecuteMsg::FundRenewal { token_id } => execute_fund_renewal(deps, info, &token_id),
        ExecuteMsg::RefundRenewal { token_id } => execute_refund_renewal(deps, info, &token_id),
        ExecuteMsg::ProcessRenewals { height } => execute_process_renewal(deps, env, height),
    }
}

/// A seller may set an Ask on their NFT to list it on Marketplace
pub fn execute_set_ask(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    token_id: &str,
    seller: Addr,
) -> Result<Response, ContractError> {
    let minter = NAME_MINTER.load(deps.storage)?;
    if info.sender != minter {
        return Err(ContractError::UnauthorizedMinter {});
    }

    let funds = may_pay(&info, NATIVE_DENOM)?;

    let collection = NAME_COLLECTION.load(deps.storage)?;

    // // Check if this contract is approved to transfer the token
    // Cw721Contract(collection.clone()).approval(
    //     &deps.querier,
    //     token_id.clone(),
    //     env.contract.address.to_string(),
    //     None,
    // )?;

    let ask = Ask {
        token_id: token_id.to_string(),
        id: increment_asks(deps.storage)?,
        seller: seller.clone(),
        height: env.block.height,
        renewal_fund: funds,
    };
    store_ask(deps.storage, &ask)?;

    let params = SUDO_PARAMS.load(deps.storage)?;

    // store reference to ask in expiration queue for future renewal processing
    let mut queue = RENEWAL_QUEUE
        .may_load(deps.storage, env.block.height)?
        .unwrap_or_default();
    queue.push(token_id.to_string());
    RENEWAL_QUEUE.save(
        deps.storage,
        env.block.height + params.blocks_per_year,
        &queue,
    )?;

    let hook = prepare_ask_hook(deps.as_ref(), &ask, HookAction::Create)?;

    let event = Event::new("set-ask")
        .add_attribute("collection", collection.to_string())
        .add_attribute("token_id", token_id)
        .add_attribute("seller", seller);

    Ok(Response::new().add_event(event).add_submessages(hook))
}

/// Places a bid on a name. The bid is escrowed in the contract.
pub fn execute_set_bid(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    token_id: &str,
) -> Result<Response, ContractError> {
    let params = SUDO_PARAMS.load(deps.storage)?;

    let ask_key = ask_key(token_id);
    asks().load(deps.storage, ask_key)?;

    let bid_price = must_pay(&info, NATIVE_DENOM)?;
    if bid_price < params.min_price {
        return Err(ContractError::PriceTooSmall(bid_price));
    }

    let bidder = info.sender;
    let mut res = Response::new();
    let bid_key = bid_key(token_id, &bidder);

    if let Some(existing_bid) = bids().may_load(deps.storage, bid_key.clone())? {
        bids().remove(deps.storage, bid_key)?;
        let refund_bidder = BankMsg::Send {
            to_address: bidder.to_string(),
            amount: vec![coin(existing_bid.amount.u128(), NATIVE_DENOM)],
        };
        res = res.add_message(refund_bidder)
    }

    let bid = Bid::new(token_id, bidder.clone(), bid_price, env.block.height);
    store_bid(deps.storage, &bid)?;

    let hook = prepare_bid_hook(deps.as_ref(), &bid, HookAction::Create)?;

    let event = Event::new("set-bid")
        .add_attribute("token_id", token_id)
        .add_attribute("bidder", bidder)
        .add_attribute("bid_price", bid_price.to_string());

    Ok(res.add_event(event).add_submessages(hook))
}

/// Removes a bid made by the bidder. Bidders can only remove their own bids
pub fn execute_remove_bid(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    token_id: &str,
) -> Result<Response, ContractError> {
    nonpayable(&info)?;
    let bidder = info.sender;

    let key = bid_key(token_id, &bidder);
    let bid = bids().load(deps.storage, key.clone())?;
    bids().remove(deps.storage, key)?;

    let refund_bidder_msg = BankMsg::Send {
        to_address: bid.bidder.to_string(),
        amount: vec![coin(bid.amount.u128(), NATIVE_DENOM)],
    };

    let hook = prepare_bid_hook(deps.as_ref(), &bid, HookAction::Delete)?;

    let event = Event::new("remove-bid")
        .add_attribute("token_id", token_id)
        .add_attribute("bidder", bidder);

    let res = Response::new()
        .add_message(refund_bidder_msg)
        .add_submessages(hook)
        .add_event(event);

    Ok(res)
}

/// Seller can accept a bid which transfers funds as well as the token.
/// The bid is removed, then a new ask is created for the same token.
pub fn execute_accept_bid(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    token_id: &str,
    bidder: Addr,
) -> Result<Response, ContractError> {
    nonpayable(&info)?;
    let collection = NAME_COLLECTION.load(deps.storage)?;
    only_owner(deps.as_ref(), &info, &collection, token_id)?;

    let ask_key = ask_key(token_id);
    let bid_key = bid_key(token_id, &bidder);

    let ask = asks().load(deps.storage, ask_key)?;
    let bid = bids().load(deps.storage, bid_key.clone())?;

    // Remove accepted bid
    bids().remove(deps.storage, bid_key)?;

    let mut res = Response::new();

    // Return renewal funds if there's any
    if !ask.renewal_fund.is_zero() {
        let msg = BankMsg::Send {
            to_address: ask.seller.to_string(),
            amount: coins(ask.renewal_fund.u128(), NATIVE_DENOM),
        };
        res = res.add_message(msg);
    }

    // Transfer funds and NFT
    finalize_sale(
        deps.as_ref(),
        ask.clone(),
        bid.amount,
        bidder.clone(),
        &mut res,
    )?;

    // Update Ask with new seller and height
    let ask = Ask {
        token_id: token_id.to_string(),
        id: ask.id,
        seller: bidder.clone(),
        height: env.block.height,
        renewal_fund: Uint128::zero(),
    };
    store_ask(deps.storage, &ask)?;

    let event = Event::new("accept-bid")
        .add_attribute("token_id", token_id)
        .add_attribute("bidder", bidder)
        .add_attribute("price", bid.amount.to_string());

    Ok(res.add_event(event))
}

pub fn execute_fund_renewal(
    deps: DepsMut,
    info: MessageInfo,
    token_id: &str,
) -> Result<Response, ContractError> {
    let payment = must_pay(&info, NATIVE_DENOM)?;

    let mut ask = asks().load(deps.storage, ask_key(token_id))?;
    // TODO: should anyone be able to fund a renewal?
    // if ask.seller != info.sender {
    //     return Err(ContractError::Unauthorized {});
    // }
    ask.renewal_fund += payment;
    asks().save(deps.storage, ask_key(token_id), &ask)?;

    Ok(Response::new().add_event(Event::new("fund-renewal").add_attribute("token_id", token_id)))
}

pub fn execute_refund_renewal(
    deps: DepsMut,
    info: MessageInfo,
    token_id: &str,
) -> Result<Response, ContractError> {
    nonpayable(&info)?;

    let mut ask = asks().load(deps.storage, ask_key(token_id))?;

    if ask.seller != info.sender {
        return Err(ContractError::Unauthorized {});
    }
    if ask.renewal_fund.is_zero() {
        return Err(ContractError::NoRenewalFund {});
    }

    let msg = BankMsg::Send {
        to_address: ask.seller.to_string(),
        amount: vec![coin(ask.renewal_fund.u128(), NATIVE_DENOM)],
    };

    ask.renewal_fund = Uint128::zero();
    asks().save(deps.storage, ask_key(token_id), &ask)?;

    Ok(Response::new()
        .add_event(Event::new("refund-renewal").add_attribute("token_id", token_id))
        .add_message(msg))
}

/// Anyone can call this to process renewals for a block and earn a reward
pub fn execute_process_renewal(
    _deps: DepsMut,
    env: Env,
    height: u64,
) -> Result<Response, ContractError> {
    println!("Processing renewals at height {}", height);

    if height > env.block.height {
        return Err(ContractError::CannotProcessFutureHeight {});
    }

    // // TODO: add renewal processing logic
    // let renewal_queue = RENEWAL_QUEUE.load(deps.storage, height)?;
    // for name in renewal_queue.iter() {
    //     let ask = asks().load(deps.storage, ask_key(name))?;
    //     if ask.renewal_fund.is_zero() {
    //         continue;
    //         // transfer ownership to name service
    //         // list in marketplace for 0.5% of bid price
    //         // if no bids, list for original price
    //     }

    //     // charge renewal fee
    //     // pay out reward to operator
    //     // reset ask

    //     // Update Ask with new height
    //     let ask = Ask {
    //         token_id: name.to_string(),
    //         id: ask.id,
    //         seller: ask.seller,
    //         height: env.block.height,
    //         renewal_fund: Uint128::zero(),
    //     };
    //     store_ask(deps.storage, &ask)?;
    // }

    Ok(Response::new()
        .add_event(Event::new("process-renewal").add_attribute("height", height.to_string())))
}

/// Transfers funds and NFT, updates bid
fn finalize_sale(
    deps: Deps,
    ask: Ask,
    price: Uint128,
    buyer: Addr,
    res: &mut Response,
) -> StdResult<()> {
    payout(deps, price, ask.seller.clone(), res)?;

    let cw721_transfer_msg = Cw721ExecuteMsg::TransferNft {
        token_id: ask.token_id.to_string(),
        recipient: buyer.to_string(),
    };

    let collection = NAME_COLLECTION.load(deps.storage)?;

    let exec_cw721_transfer = WasmMsg::Execute {
        contract_addr: collection.to_string(),
        msg: to_binary(&cw721_transfer_msg)?,
        funds: vec![],
    };
    res.messages.push(SubMsg::new(exec_cw721_transfer));

    res.messages
        .append(&mut prepare_sale_hook(deps, &ask, buyer.clone())?);

    let event = Event::new("finalize-sale")
        .add_attribute("token_id", ask.token_id.to_string())
        .add_attribute("seller", ask.seller.to_string())
        .add_attribute("buyer", buyer.to_string())
        .add_attribute("price", price.to_string());
    res.events.push(event);

    Ok(())
}

/// Payout a bid
fn payout(
    deps: Deps,
    payment: Uint128,
    payment_recipient: Addr,
    res: &mut Response,
) -> StdResult<()> {
    let params = SUDO_PARAMS.load(deps.storage)?;

    // send fees to community pool
    let network_fee = payment * params.trading_fee_percent / Uint128::from(100u128);
    if payment < network_fee {
        return Err(StdError::generic_err("Fees exceed payment"));
    }
    let community_pool_msg =
        create_fund_community_pool_msg(vec![coin(network_fee.u128(), NATIVE_DENOM)]);
    res.messages.push(SubMsg::new(community_pool_msg));

    // pay seller
    let seller_share_msg = BankMsg::Send {
        to_address: payment_recipient.to_string(),
        amount: vec![coin(
            (payment - network_fee).u128(),
            NATIVE_DENOM.to_string(),
        )],
    };
    res.messages.push(SubMsg::new(seller_share_msg));

    Ok(())
}

fn store_bid(store: &mut dyn Storage, bid: &Bid) -> StdResult<()> {
    bids().save(store, bid_key(&bid.token_id, &bid.bidder), bid)
}

fn store_ask(store: &mut dyn Storage, ask: &Ask) -> StdResult<()> {
    asks().save(store, ask_key(&ask.token_id), ask)
}

/// Checks to enfore only NFT owner can call
fn only_owner(
    deps: Deps,
    info: &MessageInfo,
    collection: &Addr,
    token_id: &str,
) -> Result<OwnerOfResponse, ContractError> {
    let res = Cw721Contract::<Empty, Empty>(collection.clone(), PhantomData, PhantomData)
        .owner_of(&deps.querier, token_id, false)?;
    if res.owner != info.sender {
        return Err(ContractError::UnauthorizedOwner {});
    }

    Ok(res)
}
