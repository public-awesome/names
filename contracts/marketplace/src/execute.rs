use crate::error::ContractError;
use crate::helpers::map_validate;
use crate::msg::{ExecuteMsg, InstantiateMsg};
use crate::state::{
    ask_key, asks, bid_key, bids, Ask, Bid, Order, SudoParams, TokenId, SUDO_PARAMS,
};
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    coin, to_binary, Addr, BankMsg, Coin, Decimal, Deps, DepsMut, Env, Event, MessageInfo,
    StdError, StdResult, Storage, Timestamp, Uint128, WasmMsg,
};
use cw2::set_contract_version;
use cw721::{Cw721ExecuteMsg, OwnerOfResponse};
use cw721_base::helpers::Cw721Contract;
use cw_utils::{may_pay, maybe_addr, must_pay, nonpayable, Duration, Expiration};
use sg1::fair_burn;
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
    if msg.max_finders_fee_bps > MAX_FEE_BPS {
        return Err(ContractError::InvalidFindersFeeBps(msg.max_finders_fee_bps));
    }
    if msg.trading_fee_bps > MAX_FEE_BPS {
        return Err(ContractError::InvalidTradingFeeBps(msg.trading_fee_bps));
    }
    if msg.bid_removal_reward_bps > MAX_FEE_BPS {
        return Err(ContractError::InvalidBidRemovalRewardBps(
            msg.bid_removal_reward_bps,
        ));
    }

    msg.bid_expiry.validate()?;

    match msg.stale_bid_duration {
        Duration::Height(_) => return Err(ContractError::InvalidDuration {}),
        Duration::Time(_) => {}
    };

    let params = SudoParams {
        trading_fee_percent: Decimal::percent(msg.trading_fee_bps),
        bid_expiry: msg.bid_expiry,
        operators: map_validate(deps.api, &msg.operators)?,
        min_price: msg.min_price,
        stale_bid_duration: msg.stale_bid_duration,
        bid_removal_reward_percent: Decimal::percent(msg.bid_removal_reward_bps),
        listing_fee: msg.listing_fee,
        collection: deps.api.addr_validate(&msg.collection)?,
    };
    SUDO_PARAMS.save(deps.storage, &params)?;

    Ok(Response::new())
}

pub struct AskInfo {
    token_id: TokenId,
    funds_recipient: Option<Addr>,
}

pub struct BidInfo {
    token_id: TokenId,
    expires: Timestamp,
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
        ExecuteMsg::SetAsk {
            token_id,
            funds_recipient,
        } => execute_set_ask(
            deps,
            env,
            info,
            AskInfo {
                token_id,
                funds_recipient: maybe_addr(api, funds_recipient)?,
            },
        ),
        ExecuteMsg::SetBid { token_id, expires } => {
            execute_set_bid(deps, env, info, BidInfo { token_id, expires })
        }
        ExecuteMsg::RemoveBid { token_id } => execute_remove_bid(deps, env, info, token_id),
        ExecuteMsg::AcceptBid { token_id, bidder } => {
            execute_accept_bid(deps, env, info, token_id, api.addr_validate(&bidder)?)
        }
        ExecuteMsg::SyncAsk { token_id } => execute_sync_ask(deps, env, info, token_id),
        ExecuteMsg::RemoveStaleAsk { token_id } => {
            execute_remove_stale_ask(deps, env, info, token_id)
        }
        ExecuteMsg::RemoveStaleBid { token_id, bidder } => {
            execute_remove_stale_bid(deps, env, info, token_id, api.addr_validate(&bidder)?)
        }
    }
}

/// A seller may set an Ask on their NFT to list it on Marketplace
pub fn execute_set_ask(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    ask_info: AskInfo,
) -> Result<Response, ContractError> {
    let AskInfo {
        token_id,
        funds_recipient,
    } = ask_info;

    let params = SUDO_PARAMS.load(deps.storage)?;
    let collection = params.collection;

    only_owner(deps.as_ref(), &info, &collection, token_id)?;

    // Check if this contract is approved to transfer the token
    Cw721Contract(collection.clone()).approval(
        &deps.querier,
        token_id.to_string(),
        env.contract.address.to_string(),
        None,
    )?;

    // Check if msg has correct listing fee
    let listing_fee = may_pay(&info, NATIVE_DENOM)?;
    if listing_fee != params.listing_fee {
        return Err(ContractError::InvalidListingFee(listing_fee));
    }

    let seller = info.sender;
    let ask = Ask {
        token_id,
        seller: seller.clone(),
        funds_recipient,
        is_active: true,
    };
    store_ask(deps.storage, &ask)?;

    // Append fair_burn msg
    let mut res = Response::new();
    if listing_fee > Uint128::zero() {
        fair_burn(listing_fee.u128(), None, &mut res);
    }

    let event = Event::new("set-ask")
        .add_attribute("collection", collection.to_string())
        .add_attribute("token_id", token_id.to_string())
        .add_attribute("seller", seller);

    Ok(res.add_event(event))
}

// TODO: use internally after bid is accepted?
/// Removes the ask on a particular NFT
fn execute_remove_ask(
    deps: DepsMut,
    info: MessageInfo,
    token_id: TokenId,
) -> Result<Response, ContractError> {
    nonpayable(&info)?;
    let params = SUDO_PARAMS.load(deps.storage)?;
    let collection = params.collection;
    only_owner(deps.as_ref(), &info, &collection, token_id)?;

    let key = ask_key(token_id);
    let ask = asks().load(deps.storage, key.clone())?;
    asks().remove(deps.storage, key)?;

    let event = Event::new("remove-ask")
        .add_attribute("collection", collection.to_string())
        .add_attribute("token_id", token_id.to_string());

    Ok(Response::new().add_event(event))
}

/// Places a bid on a listed or unlisted NFT. The bid is escrowed in the contract.
pub fn execute_set_bid(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    bid_info: BidInfo,
) -> Result<Response, ContractError> {
    let BidInfo { token_id, expires } = bid_info;
    let params = SUDO_PARAMS.load(deps.storage)?;

    let bid_price = must_pay(&info, NATIVE_DENOM)?;
    if bid_price < params.min_price {
        return Err(ContractError::PriceTooSmall(bid_price));
    }
    params.bid_expiry.is_valid(&env.block, expires)?;

    let bidder = info.sender;
    let mut res = Response::new();
    let bid_key = bid_key(token_id, &bidder);
    let ask_key = ask_key(token_id);

    if let Some(existing_bid) = bids().may_load(deps.storage, bid_key.clone())? {
        bids().remove(deps.storage, bid_key)?;
        let refund_bidder = BankMsg::Send {
            to_address: bidder.to_string(),
            amount: vec![coin(existing_bid.price.u128(), NATIVE_DENOM)],
        };
        res = res.add_message(refund_bidder)
    }

    let existing_ask = asks().may_load(deps.storage, ask_key.clone())?;

    if let Some(ask) = existing_ask.clone() {
        if !ask.is_active {
            return Err(ContractError::AskNotActive {});
        }
    }

    let save_bid = |store| -> StdResult<_> {
        let bid = Bid::new(token_id, bidder.clone(), bid_price, expires);
        store_bid(store, &bid)?;
        Ok(Some(bid))
    };

    let bid = save_bid(deps.storage)?;

    let event = Event::new("set-bid")
        .add_attribute("token_id", token_id.to_string())
        .add_attribute("bidder", bidder)
        .add_attribute("bid_price", bid_price.to_string())
        .add_attribute("expires", expires.to_string());

    Ok(res.add_event(event))
}

/// Removes a bid made by the bidder. Bidders can only remove their own bids
pub fn execute_remove_bid(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    token_id: TokenId,
) -> Result<Response, ContractError> {
    nonpayable(&info)?;
    let bidder = info.sender;

    let key = bid_key(token_id, &bidder);
    let bid = bids().load(deps.storage, key.clone())?;
    bids().remove(deps.storage, key)?;

    let refund_bidder_msg = BankMsg::Send {
        to_address: bid.bidder.to_string(),
        amount: vec![coin(bid.price.u128(), NATIVE_DENOM)],
    };

    let event = Event::new("remove-bid")
        .add_attribute("token_id", token_id.to_string())
        .add_attribute("bidder", bidder);

    let res = Response::new()
        .add_message(refund_bidder_msg)
        .add_event(event);

    Ok(res)
}

/// Seller can accept a bid which transfers funds as well as the token. The bid may or may not be associated with an ask.
pub fn execute_accept_bid(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    token_id: TokenId,
    bidder: Addr,
) -> Result<Response, ContractError> {
    nonpayable(&info)?;
    let params = SUDO_PARAMS.load(deps.storage)?;
    let collection = params.collection;
    only_owner(deps.as_ref(), &info, &collection, token_id)?;

    let bid_key = bid_key(token_id, &bidder);
    let ask_key = ask_key(token_id);

    let bid = bids().load(deps.storage, bid_key.clone())?;
    if bid.is_expired(&env.block) {
        return Err(ContractError::BidExpired {});
    }

    let ask = if let Some(existing_ask) = asks().may_load(deps.storage, ask_key.clone())? {
        if !existing_ask.is_active {
            return Err(ContractError::AskNotActive {});
        }
        asks().remove(deps.storage, ask_key)?;
        existing_ask
    } else {
        // Create a temporary Ask
        Ask {
            token_id,
            is_active: true,
            seller: info.sender,
            funds_recipient: None,
        }
    };

    // Remove accepted bid
    bids().remove(deps.storage, bid_key)?;

    let mut res = Response::new();

    // Transfer funds and NFT
    finalize_sale(deps.as_ref(), ask, bid.price, bidder.clone(), &mut res)?;

    let event = Event::new("accept-bid")
        .add_attribute("token_id", token_id.to_string())
        .add_attribute("bidder", bidder)
        .add_attribute("price", bid.price.to_string());

    Ok(res.add_event(event))
}

/// Synchronizes the active state of an ask based on token ownership.
/// This is a privileged operation called by an operator to update an ask when a transfer happens.
pub fn execute_sync_ask(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    token_id: TokenId,
) -> Result<Response, ContractError> {
    nonpayable(&info)?;
    only_operator(deps.storage, &info)?;

    let key = ask_key(token_id);
    let mut ask = asks().load(deps.storage, key.clone())?;

    let params = SUDO_PARAMS.load(deps.storage)?;
    let collection = params.collection;

    // Check if marketplace still holds approval
    // An approval will be removed when
    // 1 - There is a transfer
    // 2 - The approval expired (approvals can have different expiration times)
    let res = Cw721Contract(collection.clone()).approval(
        &deps.querier,
        token_id.to_string(),
        env.contract.address.to_string(),
        None,
    );
    if res.is_ok() == ask.is_active {
        return Err(ContractError::AskUnchanged {});
    }
    ask.is_active = res.is_ok();
    asks().save(deps.storage, key, &ask)?;

    let event = Event::new("update-ask-state")
        .add_attribute("token_id", token_id.to_string())
        .add_attribute("is_active", ask.is_active.to_string());

    Ok(Response::new().add_event(event))
}

// TODO: is this needed?
/// Privileged operation to remove a stale ask. Operators can call this to remove asks that are still in the
/// state after they have expired or a token is no longer existing.
pub fn execute_remove_stale_ask(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    token_id: TokenId,
) -> Result<Response, ContractError> {
    nonpayable(&info)?;
    only_operator(deps.storage, &info)?;

    let key = ask_key(token_id);
    let ask = asks().load(deps.storage, key.clone())?;

    let params = SUDO_PARAMS.load(deps.storage)?;
    let collection = params.collection;

    let res =
        Cw721Contract(collection.clone()).owner_of(&deps.querier, token_id.to_string(), false);
    let has_owner = res.is_ok();

    asks().remove(deps.storage, key)?;

    let event = Event::new("remove-ask")
        .add_attribute("collection", collection.to_string())
        .add_attribute("token_id", token_id.to_string())
        .add_attribute("operator", info.sender.to_string())
        .add_attribute("has_owner", has_owner.to_string());

    Ok(Response::new().add_event(event))
}

/// Privileged operation to remove a stale bid. Operators can call this to remove and refund bids that are still in the
/// state after they have expired. As a reward they get a governance-determined percentage of the bid price.
pub fn execute_remove_stale_bid(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    token_id: TokenId,
    bidder: Addr,
) -> Result<Response, ContractError> {
    nonpayable(&info)?;
    let operator = only_operator(deps.storage, &info)?;

    let bid_key = bid_key(token_id, &bidder);
    let bid = bids().load(deps.storage, bid_key.clone())?;

    let params = SUDO_PARAMS.load(deps.storage)?;
    let stale_time = (Expiration::AtTime(bid.expires_at) + params.stale_bid_duration)?;
    if !stale_time.is_expired(&env.block) {
        return Err(ContractError::BidNotStale {});
    }

    // bid is stale, refund bidder and reward operator
    bids().remove(deps.storage, bid_key)?;

    let reward = bid.price * params.bid_removal_reward_percent / Uint128::from(100u128);

    let bidder_msg = BankMsg::Send {
        to_address: bid.bidder.to_string(),
        amount: vec![coin((bid.price - reward).u128(), NATIVE_DENOM)],
    };
    let operator_msg = BankMsg::Send {
        to_address: operator.to_string(),
        amount: vec![coin(reward.u128(), NATIVE_DENOM)],
    };

    let event = Event::new("remove-stale-bid")
        .add_attribute("token_id", token_id.to_string())
        .add_attribute("bidder", bidder.to_string())
        .add_attribute("operator", operator.to_string())
        .add_attribute("reward", reward.to_string());

    Ok(Response::new()
        .add_event(event)
        .add_message(bidder_msg)
        .add_message(operator_msg))
}

/// Transfers funds and NFT, updates bid
fn finalize_sale(
    deps: Deps,
    ask: Ask,
    price: Uint128,
    buyer: Addr,
    res: &mut Response,
) -> StdResult<()> {
    payout(
        deps,
        price,
        ask.funds_recipient
            .clone()
            .unwrap_or_else(|| ask.seller.clone()),
        res,
    )?;

    let cw721_transfer_msg = Cw721ExecuteMsg::TransferNft {
        token_id: ask.token_id.to_string(),
        recipient: buyer.to_string(),
    };

    let params = SUDO_PARAMS.load(deps.storage)?;
    let collection = params.collection;

    let exec_cw721_transfer = WasmMsg::Execute {
        contract_addr: collection.to_string(),
        msg: to_binary(&cw721_transfer_msg)?,
        funds: vec![],
    };
    res.messages.push(SubMsg::new(exec_cw721_transfer));

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

    // Append Fair Burn message
    let network_fee = payment * params.trading_fee_percent / Uint128::from(100u128);
    fair_burn(network_fee.u128(), None, res);

    if payment < network_fee {
        return Err(StdError::generic_err("Fees exceed payment"));
    }
    // If token doesn't support royalties, pay seller in full
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

fn price_validate(store: &dyn Storage, price: &Coin) -> Result<(), ContractError> {
    if price.amount.is_zero() || price.denom != NATIVE_DENOM {
        return Err(ContractError::InvalidPrice {});
    }

    if price.amount < SUDO_PARAMS.load(store)?.min_price {
        return Err(ContractError::PriceTooSmall(price.amount));
    }

    Ok(())
}

fn store_bid(store: &mut dyn Storage, bid: &Bid) -> StdResult<()> {
    bids().save(store, bid_key(bid.token_id, &bid.bidder), bid)
}

fn store_ask(store: &mut dyn Storage, ask: &Ask) -> StdResult<()> {
    asks().save(store, ask_key(ask.token_id), ask)
}

/// Checks to enfore only NFT owner can call
fn only_owner(
    deps: Deps,
    info: &MessageInfo,
    collection: &Addr,
    token_id: u32,
) -> Result<OwnerOfResponse, ContractError> {
    let res =
        Cw721Contract(collection.clone()).owner_of(&deps.querier, token_id.to_string(), false)?;
    if res.owner != info.sender {
        return Err(ContractError::UnauthorizedOwner {});
    }

    Ok(res)
}

/// Checks to enforce only privileged operators
fn only_operator(store: &dyn Storage, info: &MessageInfo) -> Result<Addr, ContractError> {
    let params = SUDO_PARAMS.load(store)?;
    if !params
        .operators
        .iter()
        .any(|a| a.as_ref() == info.sender.as_ref())
    {
        return Err(ContractError::UnauthorizedOperator {});
    }

    Ok(info.sender.clone())
}
