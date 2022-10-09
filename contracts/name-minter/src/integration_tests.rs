use crate::msg::{ExecuteMsg, InstantiateMsg};
use cosmwasm_std::{coins, Addr, Uint128};
use cw721::{NumTokensResponse, OwnerOfResponse};
use cw_multi_test::{BankSudo, Contract, ContractWrapper, Executor, SudoMsg as CwSudoMsg};
use name_marketplace::msg::{
    AskResponse, BidResponse, ExecuteMsg as MarketplaceExecuteMsg, QueryMsg as MarketplaceQueryMsg,
    RenewalQueueResponse,
};
use sg721_name::ExecuteMsg as Sg721NameExecuteMsg;
use sg_multi_test::StargazeApp;
use sg_std::{StargazeMsgWrapper, NATIVE_DENOM};

pub fn contract_minter() -> Box<dyn Contract<StargazeMsgWrapper>> {
    let contract = ContractWrapper::new(
        crate::contract::execute,
        crate::contract::instantiate,
        crate::contract::query,
    )
    .with_reply(crate::contract::reply);
    Box::new(contract)
}

pub fn contract_marketplace() -> Box<dyn Contract<StargazeMsgWrapper>> {
    let contract = ContractWrapper::new(
        name_marketplace::execute::execute,
        name_marketplace::execute::instantiate,
        name_marketplace::query::query,
    )
    .with_sudo(name_marketplace::sudo::sudo);
    Box::new(contract)
}

pub fn contract_collection() -> Box<dyn Contract<StargazeMsgWrapper>> {
    let contract = ContractWrapper::new(
        sg721_name::entry::execute,
        sg721_name::entry::instantiate,
        sg721_name::entry::query,
    );
    Box::new(contract)
}

const USER: &str = "user";
const BIDDER: &str = "bidder";
const BIDDER2: &str = "bidder2";
const ADMIN: &str = "admin";
const NAME: &str = "bobo";

const TRADING_FEE_BPS: u64 = 200; // 2%
const BASE_PRICE: u128 = 100_000_000;
const BID_AMOUNT: u128 = 1_000_000_000;

const BLOCKS_PER_YEAR: u64 = 60 * 60 * 8766 / 5;

pub fn custom_mock_app() -> StargazeApp {
    StargazeApp::default()
}

// 1. Instantiate Name Marketplace
// 2. Instantiate Name Minter (which instantiates Name Collection)
// 3. Update Name Marketplace with Name Minter address
// 4. Update Name Marketplace with Name Collection address
fn instantiate_contracts() -> (StargazeApp, Addr, Addr, Addr) {
    let mut app = custom_mock_app();
    let mkt_id = app.store_code(contract_marketplace());
    let minter_id = app.store_code(contract_minter());
    let sg721_id = app.store_code(contract_collection());

    // 1. Instantiate Name Marketplace
    let msg = name_marketplace::msg::InstantiateMsg {
        trading_fee_bps: TRADING_FEE_BPS,
        min_price: Uint128::from(5u128),
    };
    let marketplace = app
        .instantiate_contract(
            mkt_id,
            Addr::unchecked(ADMIN),
            &msg,
            &[],
            "Name-Marketplace",
            None,
        )
        .unwrap();

    // 2. Instantiate Name Minter (which instantiates Name Collection)
    let msg = InstantiateMsg {
        collection_code_id: sg721_id,
        marketplace_addr: marketplace.to_string(),
    };
    let minter = app
        .instantiate_contract(
            minter_id,
            Addr::unchecked(ADMIN),
            &msg,
            &[],
            "Name-Minter",
            None,
        )
        .unwrap();

    let collection = "contract2";

    // 3. Update Name Marketplace with Name Minter address
    let msg = name_marketplace::msg::SudoMsg::UpdateNameMinter {
        minter: minter.to_string(),
    };
    let res = app.wasm_sudo(marketplace.clone(), &msg);
    assert!(res.is_ok());

    // 4. Update Name Marketplace with Name Collection address
    let msg = name_marketplace::msg::SudoMsg::UpdateNameCollection {
        collection: collection.to_string(),
    };
    let res = app.wasm_sudo(marketplace.clone(), &msg);
    assert!(res.is_ok());

    (app, marketplace, minter, Addr::unchecked(collection))
}

fn owner_of(app: &StargazeApp, collection: Addr, token_id: String) -> String {
    let res: OwnerOfResponse = app
        .wrap()
        .query_wasm_smart(
            collection,
            &sg721_base::msg::QueryMsg::OwnerOf {
                token_id,
                include_expired: None,
            },
        )
        .unwrap();

    res.owner
}

fn mint_and_list() -> (StargazeApp, Addr, Addr, Addr) {
    let (mut app, mkt, minter, collection) = instantiate_contracts();

    let user = Addr::unchecked(USER);

    // set approval for user, for all tokens
    // approve_all is needed because we don't know the token_id before-hand
    let approve_all_msg = Sg721NameExecuteMsg::ApproveAll {
        operator: mkt.to_string(),
        expires: None,
    };
    let res = app.execute_contract(user.clone(), collection.clone(), &approve_all_msg, &[]);
    assert!(res.is_ok());

    let four_letter_name_cost = BASE_PRICE * 10;

    // give user some funds
    let name_fee = coins(four_letter_name_cost, NATIVE_DENOM);
    app.sudo(CwSudoMsg::Bank({
        BankSudo::Mint {
            to_address: user.to_string(),
            amount: name_fee.clone(),
        }
    }))
    .map_err(|err| println!("{:?}", err))
    .ok();

    let msg = ExecuteMsg::MintAndList {
        name: NAME.to_string(),
    };
    let res = app.execute_contract(user.clone(), minter.clone(), &msg, &name_fee);
    assert!(res.is_ok());

    // check if name is listed in marketplace
    let res: AskResponse = app
        .wrap()
        .query_wasm_smart(
            mkt.clone(),
            &MarketplaceQueryMsg::Ask {
                token_id: NAME.to_string(),
            },
        )
        .unwrap();
    assert_eq!(res.ask.unwrap().token_id, NAME);

    // check if name is in the renewal queue
    let res: RenewalQueueResponse = app
        .wrap()
        .query_wasm_smart(
            mkt.clone(),
            &MarketplaceQueryMsg::RenewalQueue {
                height: 12345 + BLOCKS_PER_YEAR,
            },
        )
        .unwrap();
    assert_eq!(res.queue.len(), 1);
    assert_eq!(res.queue[0], NAME);

    // check if token minted
    let res: NumTokensResponse = app
        .wrap()
        .query_wasm_smart(collection.clone(), &sg721_base::msg::QueryMsg::NumTokens {})
        .unwrap();
    assert_eq!(res.count, 1);

    assert_eq!(
        owner_of(&app, collection.clone(), NAME.to_string()),
        user.to_string()
    );

    (app, mkt, minter, collection)
}

fn bid(
    mut app: StargazeApp,
    mkt: Addr,
    collection: Addr,
    bidder: &str,
    amount: u128,
) -> StargazeApp {
    let bidder = Addr::unchecked(bidder);

    // give bidder some funds
    let amount = coins(amount, NATIVE_DENOM);
    app.sudo(CwSudoMsg::Bank({
        BankSudo::Mint {
            to_address: bidder.to_string(),
            amount: amount.clone(),
        }
    }))
    .map_err(|err| println!("{:?}", err))
    .ok();

    // TODO: why doesn't this work?
    // // set approval for bidder, for specific token
    // let msg = Sg721NameExecuteMsg::Approve {
    //     spender: mkt.to_string(),
    //     token_id: NAME.to_string(),
    //     expires: None,
    // };
    let msg = Sg721NameExecuteMsg::ApproveAll {
        operator: mkt.to_string(),
        expires: None,
    };
    let res = app.execute_contract(bidder.clone(), collection, &msg, &[]);
    assert!(res.is_ok());

    let msg = MarketplaceExecuteMsg::SetBid {
        token_id: NAME.to_string(),
    };
    let res = app.execute_contract(bidder.clone(), mkt.clone(), &msg, &amount);
    assert!(res.is_ok());

    // query if bid exists
    let res: BidResponse = app
        .wrap()
        .query_wasm_smart(
            mkt,
            &MarketplaceQueryMsg::Bid {
                token_id: NAME.to_string(),
                bidder: bidder.to_string(),
            },
        )
        .unwrap();
    let bid = res.bid.unwrap();
    assert_eq!(bid.token_id, NAME.to_string());
    assert_eq!(bid.bidder, bidder.to_string());
    assert_eq!(bid.amount, amount[0].amount);

    app
}

mod execute {
    use cw721::OperatorsResponse;

    use super::*;

    #[test]
    fn check_approvals() {
        let (app, _, _, collection) = mint_and_list();

        // check operators
        let res: OperatorsResponse = app
            .wrap()
            .query_wasm_smart(
                collection,
                &sg721_base::msg::QueryMsg::AllOperators {
                    owner: USER.to_string(),
                    include_expired: None,
                    start_after: None,
                    limit: None,
                },
            )
            .unwrap();
        assert_eq!(res.operators.len(), 1);
    }

    #[test]
    fn test_mint() {
        mint_and_list();
    }

    #[test]
    fn test_bid() {
        let (app, mkt, _, collection) = mint_and_list();
        bid(app, mkt, collection, BIDDER, BID_AMOUNT);
    }

    #[test]
    fn test_accept_bid() {
        let (app, mkt, _, collection) = mint_and_list();
        let mut app = bid(app, mkt.clone(), collection.clone(), BIDDER, BID_AMOUNT);

        // user (owner) starts off with 0 internet funny money
        let res = app
            .wrap()
            .query_balance(USER.to_string(), NATIVE_DENOM)
            .unwrap();
        assert_eq!(res.amount, Uint128::new(0));

        let msg = MarketplaceExecuteMsg::AcceptBid {
            token_id: NAME.to_string(),
            bidder: BIDDER.to_string(),
        };
        let res = app.execute_contract(Addr::unchecked(USER), mkt.clone(), &msg, &[]);
        assert!(res.is_ok());

        // check if bid is removed
        let res: BidResponse = app
            .wrap()
            .query_wasm_smart(
                mkt.clone(),
                &MarketplaceQueryMsg::Bid {
                    token_id: NAME.to_string(),
                    bidder: BIDDER.to_string(),
                },
            )
            .unwrap();
        assert!(res.bid.is_none());

        // verify that the bidder is the new owner
        assert_eq!(
            owner_of(&app, collection, NAME.to_string()),
            BIDDER.to_string()
        );

        // check if user got the bid amount
        let res = app
            .wrap()
            .query_balance(USER.to_string(), NATIVE_DENOM)
            .unwrap();
        let protocol_fee = 20_000_000u128;
        assert_eq!(res.amount, Uint128::from(BID_AMOUNT - protocol_fee));

        // confirm that a new ask was created
        let res: AskResponse = app
            .wrap()
            .query_wasm_smart(
                mkt,
                &MarketplaceQueryMsg::Ask {
                    token_id: NAME.to_string(),
                },
            )
            .unwrap();
        let ask = res.ask.unwrap();
        assert_eq!(ask.token_id, NAME);
        assert_eq!(ask.seller, BIDDER.to_string());
    }

    //  test two sales cycles in a row to check if approvals work
    #[test]
    fn test_two_sales_cycles() {
        let (app, mkt, _, collection) = mint_and_list();
        let mut app = bid(app, mkt.clone(), collection.clone(), BIDDER, BID_AMOUNT);

        let msg = MarketplaceExecuteMsg::AcceptBid {
            token_id: NAME.to_string(),
            bidder: BIDDER.to_string(),
        };
        let res = app.execute_contract(Addr::unchecked(USER), mkt.clone(), &msg, &[]);
        assert!(res.is_ok());

        let mut app = bid(app, mkt.clone(), collection, BIDDER2, BID_AMOUNT);

        let msg = MarketplaceExecuteMsg::AcceptBid {
            token_id: NAME.to_string(),
            bidder: BIDDER2.to_string(),
        };
        let res = app.execute_contract(Addr::unchecked(BIDDER), mkt, &msg, &[]);
        assert!(res.is_ok());
    }
}

mod query {
    use name_marketplace::msg::{AsksResponse, BidsResponse};

    use super::*;

    #[test]
    fn query_recent_asks() {
        let (app, mkt, _, _) = mint_and_list();

        let msg = MarketplaceQueryMsg::RecentAsks {
            start_after: None,
            limit: None,
        };
        let res: AsksResponse = app.wrap().query_wasm_smart(mkt, &msg).unwrap();
        assert_eq!(res.asks.len(), 1);
        println!("{:?}", res.asks);
    }

    #[test]
    fn query_top_bids() {
        let (app, mkt, _, collection) = mint_and_list();
        let app = bid(app, mkt.clone(), collection.clone(), BIDDER, BID_AMOUNT);
        let app = bid(app, mkt.clone(), collection, BIDDER2, BID_AMOUNT * 5);

        let msg = MarketplaceQueryMsg::ReverseBidsSortedByPrice {
            start_before: None,
            limit: None,
        };
        let res: BidsResponse = app.wrap().query_wasm_smart(mkt, &msg).unwrap();
        assert_eq!(res.bids.len(), 2);
        assert_eq!(res.bids[0].amount.u128(), BID_AMOUNT * 5);
    }
}
