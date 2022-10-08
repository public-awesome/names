use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg};
use crate::state::{
    ask_key, asks, bid_key, bids, Ask, Bid, SudoParams, TokenId, NAME_COLLECTION, SUDO_PARAMS,
};
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    coin, to_binary, Addr, BankMsg, Coin, Decimal, Deps, DepsMut, Env, Event, MessageInfo,
    StdError, StdResult, Storage, Uint128, WasmMsg,
};
use cw2::set_contract_version;
use cw721::{Cw721ExecuteMsg, OwnerOfResponse};
use cw721_base::helpers::Cw721Contract;
use cw_utils::{maybe_addr, must_pay, nonpayable, Duration};
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
        trading_fee_percent: Decimal::percent(msg.trading_fee_bps),
        min_price: msg.min_price,
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
        ExecuteMsg::SetBid { token_id } => execute_set_bid(deps, env, info, BidInfo { token_id }),
        ExecuteMsg::RemoveBid { token_id } => execute_remove_bid(deps, env, info, token_id),
        ExecuteMsg::AcceptBid { token_id, bidder } => {
            execute_accept_bid(deps, env, info, token_id, api.addr_validate(&bidder)?)
        }
        ExecuteMsg::ProcessFees {} => todo!(),
    }
}

/// A seller may set an Ask on their NFT to list it on Marketplace
pub fn execute_set_ask(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    ask_info: AskInfo,
) -> Result<Response, ContractError> {
    // TODO: only name minter should be able to call this..
    let AskInfo {
        token_id,
        funds_recipient,
    } = ask_info;

    let collection = NAME_COLLECTION.load(deps.storage)?;

    // TODO: change to only_minter?
    // only_owner(deps.as_ref(), &info, &collection, token_id.clone())?;

    // // Check if this contract is approved to transfer the token
    // Cw721Contract(collection.clone()).approval(
    //     &deps.querier,
    //     token_id.clone(),
    //     env.contract.address.to_string(),
    //     None,
    // )?;

    let seller = info.sender;
    let ask = Ask {
        token_id: token_id.clone(),
        seller: seller.clone(),
        funds_recipient,
        height: env.block.height,
    };
    store_ask(deps.storage, &ask)?;

    // TODO: store reference to ask in expiration queue

    let event = Event::new("set-ask")
        .add_attribute("collection", collection.to_string())
        .add_attribute("token_id", token_id)
        .add_attribute("seller", seller);

    Ok(Response::new().add_event(event))
}

// TODO: use internally after bid is accepted?
/// Removes the ask on a particular NFT
fn _execute_remove_ask(
    deps: DepsMut,
    info: MessageInfo,
    token_id: TokenId,
) -> Result<Response, ContractError> {
    nonpayable(&info)?;
    let collection = NAME_COLLECTION.load(deps.storage)?;
    only_owner(deps.as_ref(), &info, &collection, token_id.clone())?;

    let key = ask_key(token_id.clone());
    asks().remove(deps.storage, key)?;

    let event = Event::new("remove-ask")
        .add_attribute("collection", collection.to_string())
        .add_attribute("token_id", token_id);

    Ok(Response::new().add_event(event))
}

/// Places a bid on a listed or unlisted NFT. The bid is escrowed in the contract.
pub fn execute_set_bid(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    bid_info: BidInfo,
) -> Result<Response, ContractError> {
    let BidInfo { token_id } = bid_info;
    let params = SUDO_PARAMS.load(deps.storage)?;

    let bid_price = must_pay(&info, NATIVE_DENOM)?;
    if bid_price < params.min_price {
        return Err(ContractError::PriceTooSmall(bid_price));
    }

    let bidder = info.sender;
    let mut res = Response::new();
    let bid_key = bid_key(token_id.clone(), &bidder);
    let ask_key = ask_key(token_id.clone());

    if let Some(existing_bid) = bids().may_load(deps.storage, bid_key.clone())? {
        bids().remove(deps.storage, bid_key)?;
        let refund_bidder = BankMsg::Send {
            to_address: bidder.to_string(),
            amount: vec![coin(existing_bid.price.u128(), NATIVE_DENOM)],
        };
        res = res.add_message(refund_bidder)
    }

    let existing_ask = asks().may_load(deps.storage, ask_key.clone())?;

    let save_bid = |store| -> StdResult<_> {
        let bid = Bid::new(
            token_id.clone(),
            bidder.clone(),
            bid_price,
            env.block.height,
        );
        store_bid(store, &bid)?;
        Ok(Some(bid))
    };

    let bid = save_bid(deps.storage)?;

    let event = Event::new("set-bid")
        .add_attribute("token_id", token_id)
        .add_attribute("bidder", bidder)
        .add_attribute("bid_price", bid_price.to_string());

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

    let key = bid_key(token_id.clone(), &bidder);
    let bid = bids().load(deps.storage, key.clone())?;
    bids().remove(deps.storage, key)?;

    let refund_bidder_msg = BankMsg::Send {
        to_address: bid.bidder.to_string(),
        amount: vec![coin(bid.price.u128(), NATIVE_DENOM)],
    };

    let event = Event::new("remove-bid")
        .add_attribute("token_id", token_id)
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
    let collection = NAME_COLLECTION.load(deps.storage)?;
    only_owner(deps.as_ref(), &info, &collection, token_id.clone())?;

    let bid_key = bid_key(token_id.clone(), &bidder);
    let ask_key = ask_key(token_id.clone());

    let bid = bids().load(deps.storage, bid_key.clone())?;

    let ask = if let Some(existing_ask) = asks().may_load(deps.storage, ask_key.clone())? {
        asks().remove(deps.storage, ask_key)?;
        existing_ask
    } else {
        // Create a temporary Ask
        Ask {
            token_id: token_id.clone(),
            seller: info.sender,
            funds_recipient: None,
            height: env.block.height,
        }
    };

    // Remove accepted bid
    bids().remove(deps.storage, bid_key)?;

    let mut res = Response::new();

    // Transfer funds and NFT
    finalize_sale(deps.as_ref(), ask, bid.price, bidder.clone(), &mut res)?;

    let event = Event::new("accept-bid")
        .add_attribute("token_id", token_id)
        .add_attribute("bidder", bidder)
        .add_attribute("price", bid.price.to_string());

    Ok(res.add_event(event))
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

    let collection = NAME_COLLECTION.load(deps.storage)?;

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
    // fair_burn(network_fee.u128(), None, res);

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

fn store_bid(store: &mut dyn Storage, bid: &Bid) -> StdResult<()> {
    bids().save(store, bid_key(bid.token_id.clone(), &bid.bidder), bid)
}

fn store_ask(store: &mut dyn Storage, ask: &Ask) -> StdResult<()> {
    asks().save(store, ask_key(ask.token_id.clone()), ask)
}

/// Checks to enfore only NFT owner can call
fn only_owner(
    deps: Deps,
    info: &MessageInfo,
    collection: &Addr,
    token_id: TokenId,
) -> Result<OwnerOfResponse, ContractError> {
    let res = Cw721Contract(collection.clone()).owner_of(&deps.querier, token_id, false)?;
    if res.owner != info.sender {
        return Err(ContractError::UnauthorizedOwner {});
    }

    Ok(res)
}
