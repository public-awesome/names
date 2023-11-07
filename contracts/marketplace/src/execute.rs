use std::marker::PhantomData;

use crate::error::ContractError;
use crate::hooks::{prepare_ask_hook, prepare_bid_hook, prepare_sale_hook};
use crate::msg::{ExecuteMsg, HookAction, InstantiateMsg};
use crate::state::{
    ask_key, asks, bid_key, bids, increment_asks, Ask, Bid, SudoParams, IS_SETUP, NAME_COLLECTION,
    NAME_MINTER, RENEWAL_QUEUE, SUDO_PARAMS,
};
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    coin, coins, to_binary, Addr, BankMsg, Decimal, Deps, DepsMut, Empty, Env, Event, MessageInfo,
    Order, StdError, StdResult, Storage, Timestamp, Uint128, WasmMsg,
};
use cw2::set_contract_version;
use cw721::{Cw721ExecuteMsg, OwnerOfResponse};
use cw721_base::helpers::Cw721Contract;
use cw_storage_plus::Bound;
use cw_utils::{must_pay, nonpayable};
use semver::Version;
use sg_name_common::{charge_fees, SECONDS_PER_WEEK, SECONDS_PER_YEAR};
use sg_std::{Response, SubMsg, NATIVE_DENOM};

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
        trading_fee_percent: Decimal::percent(msg.trading_fee_bps) / Uint128::from(100u128),
        min_price: msg.min_price,
        ask_interval: msg.ask_interval,
    };
    SUDO_PARAMS.save(deps.storage, &params)?;

    IS_SETUP.save(deps.storage, &false)?;

    Ok(Response::new().add_attribute("action", "instantiate"))
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
        ExecuteMsg::RemoveAsk { token_id } => execute_remove_ask(deps, info, &token_id),
        ExecuteMsg::UpdateAsk { token_id, seller } => {
            execute_update_ask(deps, info, &token_id, api.addr_validate(&seller)?)
        }
        ExecuteMsg::SetBid { token_id } => execute_set_bid(deps, env, info, &token_id),
        ExecuteMsg::RemoveBid { token_id } => execute_remove_bid(deps, env, info, &token_id),
        ExecuteMsg::AcceptBid { token_id, bidder } => {
            execute_accept_bid(deps, env, info, &token_id, api.addr_validate(&bidder)?)
        }
        ExecuteMsg::FundRenewal { token_id } => execute_fund_renewal(deps, info, &token_id),
        ExecuteMsg::RefundRenewal { token_id } => execute_refund_renewal(deps, info, &token_id),
        ExecuteMsg::ProcessRenewals { time } => execute_process_renewal(deps, env, time),
        ExecuteMsg::Setup { minter, collection } => execute_setup(
            deps,
            api.addr_validate(&minter)?,
            api.addr_validate(&collection)?,
        ),
    }
}

/// Setup this contract (can be run once only)
pub fn execute_setup(
    deps: DepsMut,
    minter: Addr,
    collection: Addr,
) -> Result<Response, ContractError> {
    if IS_SETUP.load(deps.storage)? {
        return Err(ContractError::AlreadySetup {});
    }
    NAME_MINTER.save(deps.storage, &minter)?;
    NAME_COLLECTION.save(deps.storage, &collection)?;
    IS_SETUP.save(deps.storage, &true)?;

    let event = Event::new("setup")
        .add_attribute("minter", minter)
        .add_attribute("collection", collection);
    Ok(Response::new().add_event(event))
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

    let collection = NAME_COLLECTION.load(deps.storage)?;

    // check if collection is approved to transfer on behalf of the seller
    let ops = Cw721Contract::<Empty, Empty>(collection, PhantomData, PhantomData).all_operators(
        &deps.querier,
        seller.to_string(),
        false,
        None,
        None,
    )?;
    if ops.is_empty() {
        return Err(ContractError::NotApproved {});
    }

    let renewal_time = env.block.time.plus_seconds(SECONDS_PER_YEAR);

    let ask = Ask {
        token_id: token_id.to_string(),
        id: increment_asks(deps.storage)?,
        seller: seller.clone(),
        renewal_time,
        renewal_fund: Uint128::zero(),
    };
    store_ask(deps.storage, &ask)?;

    RENEWAL_QUEUE.save(
        deps.storage,
        (renewal_time.seconds(), ask.id),
        &token_id.to_string(),
    )?;

    let hook = prepare_ask_hook(deps.as_ref(), &ask, HookAction::Create)?;

    let event = Event::new("set-ask")
        .add_attribute("token_id", token_id)
        .add_attribute("ask_id", ask.id.to_string())
        .add_attribute("renewal_time", renewal_time.to_string())
        .add_attribute("seller", seller);

    Ok(Response::new().add_event(event).add_submessages(hook))
}

/// Removes the ask on a particular NFT
pub fn execute_remove_ask(
    deps: DepsMut,
    info: MessageInfo,
    token_id: &str,
) -> Result<Response, ContractError> {
    nonpayable(&info)?;

    // `ask` can only be removed by burning from the collection
    let collection = NAME_COLLECTION.load(deps.storage)?;
    if info.sender != collection {
        return Err(ContractError::Unauthorized {});
    }

    // don't allow burning if ask has bids on it
    let bid_count = bids()
        .prefix(token_id.to_string())
        .keys(deps.storage, None, None, Order::Ascending)
        .count();
    if bid_count > 0 {
        return Err(ContractError::ExistingBids {});
    }

    let key = ask_key(token_id);
    let ask = asks().load(deps.storage, key.clone())?;
    asks().remove(deps.storage, key)?;

    RENEWAL_QUEUE.remove(deps.storage, (ask.renewal_time.seconds(), ask.id));

    let hook = prepare_ask_hook(deps.as_ref(), &ask, HookAction::Delete)?;

    let event = Event::new("remove-ask").add_attribute("token_id", token_id);

    Ok(Response::new().add_event(event).add_submessages(hook))
}

/// When an NFT is transferred, the `ask` has to be updated with the new
/// seller. Also any renewal funds should be refunded to the previous owner.
pub fn execute_update_ask(
    deps: DepsMut,
    info: MessageInfo,
    token_id: &str,
    seller: Addr,
) -> Result<Response, ContractError> {
    let collection = NAME_COLLECTION.load(deps.storage)?;
    if info.sender != collection {
        return Err(ContractError::Unauthorized {});
    }

    let mut res = Response::new();

    // refund any renewal funds and update the seller
    let mut ask = asks().load(deps.storage, ask_key(token_id))?;
    if !ask.renewal_fund.is_zero() {
        let msg = BankMsg::Send {
            to_address: ask.seller.to_string(),
            amount: coins(ask.renewal_fund.u128(), NATIVE_DENOM),
        };
        res = res.add_message(msg);
        ask.renewal_fund = Uint128::zero();
    }
    ask.seller = seller.clone();
    asks().save(deps.storage, ask_key(token_id), &ask)?;

    let event = Event::new("update-ask")
        .add_attribute("token_id", token_id)
        .add_attribute("seller", seller);

    Ok(res.add_event(event))
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

    let bid = Bid::new(token_id, bidder.clone(), bid_price, env.block.time);
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

    // Check if token is approved for transfer
    Cw721Contract::<Empty, Empty>(collection, PhantomData, PhantomData).approval(
        &deps.querier,
        token_id,
        info.sender.as_ref(),
        None,
    )?;

    // Remove accepted bid
    bids().remove(deps.storage, bid_key)?;

    // Update renewal queue
    let renewal_time = env.block.time.plus_seconds(SECONDS_PER_YEAR);
    RENEWAL_QUEUE.save(
        deps.storage,
        (renewal_time.seconds(), ask.id),
        &token_id.to_string(),
    )?;

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

    // Update Ask with new seller and renewal time
    let ask = Ask {
        token_id: token_id.to_string(),
        id: ask.id,
        seller: bidder.clone(),
        renewal_time,
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
    ask.renewal_fund += payment;
    asks().save(deps.storage, ask_key(token_id), &ask)?;

    let event = Event::new("fund-renewal")
        .add_attribute("token_id", token_id)
        .add_attribute("payment", payment);
    Ok(Response::new().add_event(event))
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

    let event = Event::new("refund-renewal")
        .add_attribute("token_id", token_id)
        .add_attribute("refund", ask.renewal_fund);
    Ok(Response::new().add_event(event).add_message(msg))
}

/// Anyone can call this to process renewals for a block and earn a reward
pub fn execute_process_renewal(
    deps: DepsMut,
    env: Env,
    time: Timestamp,
) -> Result<Response, ContractError> {
    println!("Processing renewals at time {}", time);
    let collection = NAME_COLLECTION.load(deps.storage)?;

    // make sure we're not trying to renew in the future
    if time > env.block.time {
        return Err(ContractError::CannotProcessFutureRenewal {});
    }

    // get the queue to process
    let renewal_queue = RENEWAL_QUEUE.load(deps.storage, (time.seconds(), 0))?;
    // if renewal queue is empty we error out
    if renewal_queue.is_empty() {
        print!("Queue is empty");
        return Err(ContractError::NoRenewalQueue {});
    }
    // fetch all items in the renewal queue
    let items: Result<Vec<((u64, u64), String)>, cosmwasm_std::StdError> = RENEWAL_QUEUE
        .range(
            deps.as_ref().storage,
            None,
            None,
            cosmwasm_std::Order::Ascending,
        )
        .collect();

    // iterate through the items in the queue
    // and process the renewal for each of them
    for item in items.unwrap() {
        // TODO: remove println debug statement
        println!("Processing renewal queue item: {:?}", item.1);
        let res = Response::new();
        let name = item.1;

        // fetch the asks and if there aren't any
        // burn the name
        let asks_for_name = asks()
            .range(
                deps.as_ref().storage,
                Some(Bound::inclusive(ask_key(&name))),
                None,
                Order::Ascending,
            )
            .collect::<Vec<_>>();
        if asks_for_name.is_empty() {
            let burn_msg = Cw721ExecuteMsg::Burn {
                token_id: name.to_string(),
            };
            let exec_burn_msg = WasmMsg::Execute {
                contract_addr: collection.to_string(),
                msg: to_binary(&burn_msg)?,
                funds: vec![],
            };

            res.add_event(Event::new("burn").add_attribute("token_id", name.to_string()))
                .add_message(exec_burn_msg);
        } else {

            // otherwise we begin the renewal logic
            let mut res = Response::new();

            // pull the ask
            let mut ask = asks().load(deps.storage, ask_key(&name))?;

            // if the renewal was never funded then
            // burn the name
            if ask.renewal_fund.is_zero() || true {
                let burn_msg = Cw721ExecuteMsg::Burn {
                    token_id: name.to_string(),
                };
                let exec_burn_msg = WasmMsg::Execute {
                    contract_addr: collection.to_string(),
                    msg: to_binary(&burn_msg)?,
                    funds: vec![],
                };
                let burn_event = Event::new("burn").add_attribute("token_id", name.to_string());

                res.add_event(burn_event).add_message(exec_burn_msg);
            } else {

                // Otherwise it was funded so we will find the
                // highest bid for this name in the last 4 weeks
                let four_weeks_ago = env.block.time.minus_seconds(4 * SECONDS_PER_WEEK);
                let bids_for_name = bids()
                    .range(
                        deps.as_ref().storage,
                        Some(Bound::inclusive(bid_key(&name, &ask.seller))),
                        None,
                        Order::Descending,
                    )
                    .collect::<Vec<_>>();
                let highest_bid_in_last_4_weeks = bids_for_name
                    .iter()
                    .filter(|bid| bid.as_ref().unwrap().1.created_time >= four_weeks_ago)
                    .max_by(|a, b| {
                        a.as_ref()
                            .unwrap()
                            .1
                            .amount
                            .cmp(&b.as_ref().unwrap().1.amount)
                    });

                // set the payment from the bid to 0.5% of the that value
                let payment = match highest_bid_in_last_4_weeks {
                    Some(bid) => bid
                        .as_ref()
                        .unwrap()
                        .1
                        .amount
                        .multiply_ratio(5u128, 1000u128), // 0.5% of the highest bid
                    None => Uint128::zero(),
                };

                // actually move tokens around
                if payment <= ask.renewal_fund {
                    // Deduct the tokens for the payment from the renewal fund
                    charge_fees(&mut res, Decimal::zero(), payment);
                    ask.renewal_fund = ask.renewal_fund.checked_sub(payment).unwrap();
                } else {
                    // TODO: ??
                    // do we burn the name?
                    // do we list it for sale and set the
                    // price at 0.5% of the highest bid?
                    
                }

                // Reset the ask
                let renewal_time = env.block.time.plus_seconds(SECONDS_PER_YEAR);
                let reset_ask = Ask {
                    token_id: name.to_string(),
                    id: ask.id,
                    seller: ask.seller,
                    renewal_time,
                    renewal_fund: ask.renewal_fund,
                };
                store_ask(deps.storage, &reset_ask)?;

                let reset_event = Event::new("reset").add_attribute("token_id", name.to_string());

                let exec_reset_msg = WasmMsg::Execute {
                    contract_addr: collection.to_string(),
                    msg: to_binary(&reset_ask)?,
                    funds: vec![],
                };

                res.add_event(reset_event).add_message(exec_reset_msg);
            }
        }
    }

    let event = Event::new("process-renewal").add_attribute("time", time.to_string());
    Ok(Response::new().add_event(event))
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

    let fee = payment * params.trading_fee_percent;
    if fee > payment {
        return Err(StdError::generic_err("Fees exceed payment"));
    }
    charge_fees(res, params.trading_fee_percent, fee);

    // pay seller
    let seller_share_msg = BankMsg::Send {
        to_address: payment_recipient.to_string(),
        amount: vec![coin((payment - fee).u128(), NATIVE_DENOM.to_string())],
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

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, _msg: Empty) -> Result<Response, ContractError> {
    let current_version = cw2::get_contract_version(deps.storage)?;
    if current_version.contract != CONTRACT_NAME {
        return Err(StdError::generic_err("Cannot upgrade to a different contract").into());
    }
    let version: Version = current_version
        .version
        .parse()
        .map_err(|_| StdError::generic_err("Invalid contract version"))?;
    let new_version: Version = CONTRACT_VERSION
        .parse()
        .map_err(|_| StdError::generic_err("Invalid contract version"))?;

    if version > new_version {
        return Err(StdError::generic_err("Cannot upgrade to a previous contract version").into());
    }
    // if same version return
    if version == new_version {
        return Ok(Response::new());
    }

    // set new contract version
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    Ok(Response::new())
}
