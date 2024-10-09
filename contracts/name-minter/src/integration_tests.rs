use std::str::FromStr;

use crate::contract::{execute, instantiate, reply};
use crate::msg::{ExecuteMsg, InstantiateMsg};
use crate::query::query;
use anyhow::Result as AnyResult;
use cosmwasm_std::{coins, Addr, Decimal, Timestamp, Uint128};
use cw721::{NumTokensResponse, OwnerOfResponse};
use cw_multi_test::{
    AppResponse, BankSudo, Contract, ContractWrapper, Executor, SudoMsg as CwSudoMsg,
};
use name_marketplace::helpers::get_char_price;
use name_marketplace::msg::{
    ExecuteMsg as MarketplaceExecuteMsg, QueryMsg as MarketplaceQueryMsg,
    SudoMsg as MarketplaceSudoMsg,
};
use name_marketplace::state::Bid;
use name_marketplace::{
    msg::QueryMsg as NameMarketplaceQueryMsg, state::SudoParams as NameMarketplaceParams,
};
use sg721_name::ExecuteMsg as Sg721NameExecuteMsg;
use sg_multi_test::StargazeApp;
use sg_name::{SgNameExecuteMsg, SgNameQueryMsg};
use sg_name_common::SECONDS_PER_YEAR;
use sg_name_minter::{
    SgNameMinterQueryMsg, SudoParams as NameMinterParams, PUBLIC_MINT_START_TIME_IN_SECONDS,
};
use sg_std::{StargazeMsgWrapper, NATIVE_DENOM};
use whitelist_updatable::msg::QueryMsg as PercentWhitelistQueryMsg;
use whitelist_updatable_flatrate::msg::{
    ExecuteMsg as WhitelistExecuteMsg, QueryMsg as WhitelistQueryMsg,
};

pub fn contract_minter() -> Box<dyn Contract<StargazeMsgWrapper>> {
    let contract = ContractWrapper::new(execute, instantiate, query).with_reply(reply);
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
    )
    .with_sudo(sg721_name::sudo::sudo);
    Box::new(contract)
}

pub fn contract_whitelist() -> Box<dyn Contract<StargazeMsgWrapper>> {
    let contract = ContractWrapper::new(
        whitelist_updatable_flatrate::contract::execute,
        whitelist_updatable_flatrate::contract::instantiate,
        whitelist_updatable_flatrate::contract::query,
    );
    Box::new(contract)
}

pub fn contract_whitelist_percent() -> Box<dyn Contract<StargazeMsgWrapper>> {
    let contract = ContractWrapper::new(
        whitelist_updatable::contract::execute,
        whitelist_updatable::contract::instantiate,
        whitelist_updatable::contract::query,
    );
    Box::new(contract)
}

//
pub fn contract_nft() -> Box<dyn Contract<StargazeMsgWrapper>> {
    let contract = ContractWrapper::new(
        sg721_base::entry::execute,
        sg721_base::entry::instantiate,
        sg721_base::entry::query,
    );
    Box::new(contract)
}

const USER: &str = "user";
const USER2: &str = "user2";
const USER3: &str = "user3";
const USER4: &str = "user4";
const BIDDER: &str = "bidder";
const BIDDER2: &str = "bidder2";
const ADMIN: &str = "admin";
const ADMIN2: &str = "admin2";
const NAME: &str = "bobo";
const NAME2: &str = "mccool";
const VERIFIER: &str = "verifier";
const OPERATOR: &str = "operator";

const TRADING_FEE_BPS: u64 = 200; // 2%
const BASE_PRICE: u128 = 100_000_000;
const BID_AMOUNT: u128 = 1_000_000_000;
const PER_ADDRESS_LIMIT: u32 = 2;
const TRADING_START_TIME_OFFSET_IN_SECONDS: u64 = 2 * SECONDS_PER_YEAR;

const MKT: &str = "contract0";
const MINTER: &str = "contract1";
const COLLECTION: &str = "contract2";
const WHITELIST: &str = "contract3";

// NOTE: This are mostly Marketplace integration tests. They could possibly be moved into the marketplace contract.

pub fn custom_mock_app(start_time: Option<Timestamp>) -> StargazeApp {
    let time = start_time.unwrap_or(PUBLIC_MINT_START_TIME_IN_SECONDS);
    set_block_time(StargazeApp::default(), time)
}

pub fn set_block_time(mut app: StargazeApp, time: Timestamp) -> StargazeApp {
    let mut block_info = app.block_info();
    block_info.time = time;
    app.set_block(block_info);
    app
}

// 1. Instantiate Name Marketplace
// 2. Instantiate Name Minter (which instantiates Name Collection)
// 3. Setup Name Marketplace with Name Minter and Collection addresses
// 4. Instantiate Whitelist
// 5. Update Whitelist with Name Minter
// 6. Add Whitelist to Name Minter
fn instantiate_contracts(
    creator: Option<String>,
    admin: Option<String>,
    start_time: Option<Timestamp>,
) -> StargazeApp {
    let mut app = custom_mock_app(start_time);
    let mkt_id = app.store_code(contract_marketplace());
    let minter_id = app.store_code(contract_minter());
    let sg721_id = app.store_code(contract_collection());
    let wl_id = app.store_code(contract_whitelist());

    // 1. Instantiate Name Marketplace
    let msg = name_marketplace::msg::InstantiateMsg {
        trading_fee_bps: TRADING_FEE_BPS,
        min_price: Uint128::from(5u128),
        ask_interval: 60,
        max_renewals_per_block: 20,
        valid_bid_query_limit: 10,
        renew_window: 60 * 60 * 24 * 30,
        renewal_bid_percentage: Decimal::from_str("0.005").unwrap(),
        operator: OPERATOR.to_string(),
    };
    let marketplace = app
        .instantiate_contract(
            mkt_id,
            Addr::unchecked(creator.clone().unwrap_or_else(|| ADMIN.to_string())),
            &msg,
            &[],
            "Name-Marketplace",
            admin.clone(),
        )
        .unwrap();

    // 2. Instantiate Name Minter (which instantiates Name Collection)
    let msg = InstantiateMsg {
        admin: admin.clone(),
        verifier: Some(VERIFIER.to_string()),
        collection_code_id: sg721_id,
        marketplace_addr: marketplace.to_string(),
        base_price: Uint128::from(BASE_PRICE),
        min_name_length: 3,
        max_name_length: 63,
        fair_burn_bps: 5000, // 50%
        whitelists: vec![],
    };
    let minter = app
        .instantiate_contract(
            minter_id,
            Addr::unchecked(creator.unwrap_or_else(|| ADMIN2.to_string())),
            &msg,
            &[],
            "Name-Minter",
            None,
        )
        .unwrap();

    // 3. Setup Name Marketplace
    let msg = name_marketplace::msg::ExecuteMsg::Setup {
        minter: minter.to_string(),
        collection: COLLECTION.to_string(),
    };
    let res = app.execute_contract(
        Addr::unchecked(ADMIN.to_string()),
        marketplace.clone(),
        &msg,
        &[],
    );
    assert!(res.is_ok());

    let res: Addr = app
        .wrap()
        .query_wasm_smart(COLLECTION, &(SgNameQueryMsg::NameMarketplace {}))
        .unwrap();
    assert_eq!(res, marketplace.to_string());

    // 4. Instantiate Whitelist
    let msg = whitelist_updatable_flatrate::msg::InstantiateMsg {
        per_address_limit: PER_ADDRESS_LIMIT,
        addresses: vec![
            "addr0001".to_string(),
            "addr0002".to_string(),
            USER.to_string(),
            USER4.to_string(),
            ADMIN2.to_string(),
        ],
        mint_discount_amount: None,
        admin_list: None,
    };
    let wl = app
        .instantiate_contract(wl_id, Addr::unchecked(ADMIN2), &msg, &[], "Whitelist", None)
        .unwrap();

    // 5. Add Whitelist to Name Minter
    if let Some(admin) = admin {
        let msg = ExecuteMsg::AddWhitelist {
            address: wl.to_string(),
            whitelist_type: "FlatrateDiscount".to_string(),
        };
        let res = app.execute_contract(Addr::unchecked(admin), Addr::unchecked(minter), &msg, &[]);
        assert!(res.is_ok());
    }

    app
}

fn owner_of(app: &StargazeApp, token_id: String) -> String {
    let res: OwnerOfResponse = app
        .wrap()
        .query_wasm_smart(
            COLLECTION,
            &(sg721_base::msg::QueryMsg::OwnerOf {
                token_id,
                include_expired: None,
            }),
        )
        .unwrap();

    res.owner
}

fn update_block_time(app: &mut StargazeApp, add_secs: u64) {
    let mut block = app.block_info();
    block.time = block.time.plus_seconds(add_secs);
    app.set_block(block);
}

fn mint_and_list(
    app: &mut StargazeApp,
    name: &str,
    user: &str,
    discount: Option<Decimal>,
) -> AnyResult<AppResponse> {
    // set approval for user, for all tokens
    // approve_all is needed because we don't know the token_id before-hand
    let approve_all_msg = Sg721NameExecuteMsg::ApproveAll {
        operator: MKT.to_string(),
        expires: None,
    };
    let res = app.execute_contract(
        Addr::unchecked(user),
        Addr::unchecked(COLLECTION),
        &approve_all_msg,
        &[],
    );
    assert!(res.is_ok());

    let amount: Uint128 = (match name.len() {
        0..=2 => BASE_PRICE,
        3 => BASE_PRICE * 100,
        4 => BASE_PRICE * 10,
        _ => BASE_PRICE,
    })
    .into();

    let amount = discount
        .map(|d| amount * (Decimal::one() - d))
        .unwrap_or(amount);

    // give user some funds
    let name_fee = coins(amount.into(), NATIVE_DENOM);
    if amount > Uint128::from(0u128) {
        app.sudo(CwSudoMsg::Bank({
            BankSudo::Mint {
                to_address: user.to_string(),
                amount: name_fee.clone(),
            }
        }))
        .map_err(|err| println!("{:?}", err))
        .ok();
    }

    let msg = ExecuteMsg::MintAndList {
        name: name.to_string(),
    };

    app.execute_contract(
        Addr::unchecked(user),
        Addr::unchecked(MINTER),
        &msg,
        &name_fee,
    )
}

fn bid(app: &mut StargazeApp, name: &str, bidder: &str, amount: u128) {
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

    let msg = MarketplaceExecuteMsg::SetBid {
        token_id: name.to_string(),
    };
    let res = app.execute_contract(bidder.clone(), Addr::unchecked(MKT), &msg, &amount);
    assert!(res.is_ok());

    // query if bid exists
    let res: Option<Bid> = app
        .wrap()
        .query_wasm_smart(
            MKT,
            &(MarketplaceQueryMsg::Bid {
                token_id: name.to_string(),
                bidder: bidder.to_string(),
            }),
        )
        .unwrap();
    let bid = res.unwrap();
    assert_eq!(bid.token_id, name.to_string());
    assert_eq!(bid.bidder, bidder.to_string());
    assert_eq!(bid.amount, amount[0].amount);
}

mod execute {
    use cosmwasm_std::{attr, StdError};
    use cw721::{NftInfoResponse, OperatorsResponse};
    use name_marketplace::state::{Ask, SudoParams};
    use sg721_name::msg::QueryMsg as Sg721NameQueryMsg;
    use sg_name::Metadata;
    use whitelist_updatable_flatrate::msg::QueryMsg::IncludesAddress;

    use crate::msg::QueryMsg;

    use super::*;

    #[test]
    fn check_approvals() {
        let mut app = instantiate_contracts(None, None, None);

        let res = mint_and_list(&mut app, NAME, USER, None);
        assert!(res.is_ok());

        // check operators
        let res: OperatorsResponse = app
            .wrap()
            .query_wasm_smart(
                COLLECTION,
                &(sg721_base::msg::QueryMsg::AllOperators {
                    owner: USER.to_string(),
                    include_expired: None,
                    start_after: None,
                    limit: None,
                }),
            )
            .unwrap();
        assert_eq!(res.operators.len(), 1);
    }

    #[test]
    fn test_mint() {
        let mut app = instantiate_contracts(None, None, None);

        let res = mint_and_list(&mut app, NAME, USER, None);
        assert!(res.is_ok());

        // check if name is listed in marketplace
        let res: Option<Ask> = app
            .wrap()
            .query_wasm_smart(
                MKT,
                &(MarketplaceQueryMsg::Ask {
                    token_id: NAME.to_string(),
                }),
            )
            .unwrap();
        assert_eq!(res.unwrap().token_id, NAME);

        // check if token minted
        let _res: NumTokensResponse = app
            .wrap()
            .query_wasm_smart(
                Addr::unchecked(COLLECTION),
                &(sg721_base::msg::QueryMsg::NumTokens {}),
            )
            .unwrap();

        assert_eq!(owner_of(&app, NAME.to_string()), USER.to_string());
    }

    #[test]
    fn test_bid() {
        let mut app = instantiate_contracts(None, None, None);

        let res = mint_and_list(&mut app, NAME, USER, None);
        assert!(res.is_ok());
        bid(&mut app, NAME, BIDDER, BID_AMOUNT);
    }

    #[test]
    fn test_accept_bid() {
        let mut app = instantiate_contracts(None, None, None);

        let res = mint_and_list(&mut app, NAME, USER, None);
        assert!(res.is_ok());

        bid(&mut app, NAME, BIDDER, BID_AMOUNT);

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
        let res = app.execute_contract(Addr::unchecked(USER), Addr::unchecked(MKT), &msg, &[]);
        assert!(res.is_ok());

        // check if bid is removed
        let res: Option<Bid> = app
            .wrap()
            .query_wasm_smart(
                MKT,
                &(MarketplaceQueryMsg::Bid {
                    token_id: NAME.to_string(),
                    bidder: BIDDER.to_string(),
                }),
            )
            .unwrap();
        assert!(res.is_none());

        // verify that the bidder is the new owner
        assert_eq!(owner_of(&app, NAME.to_string()), BIDDER.to_string());

        // check if user got the bid amount
        let res = app
            .wrap()
            .query_balance(USER.to_string(), NATIVE_DENOM)
            .unwrap();
        let protocol_fee = 20_000_000u128;
        assert_eq!(res.amount, Uint128::from(BID_AMOUNT - protocol_fee));

        // confirm that a new ask was created
        let res: Option<Ask> = app
            .wrap()
            .query_wasm_smart(
                MKT,
                &(MarketplaceQueryMsg::Ask {
                    token_id: NAME.to_string(),
                }),
            )
            .unwrap();
        let ask = res.unwrap();
        assert_eq!(ask.token_id, NAME);
        assert_eq!(ask.seller, BIDDER.to_string());
    }

    //  test two sales cycles in a row to check if approvals work
    #[test]
    fn test_two_sales_cycles() {
        let mut app = instantiate_contracts(None, None, None);

        let res = mint_and_list(&mut app, NAME, USER, None);
        assert!(res.is_ok());

        bid(&mut app, NAME, BIDDER, BID_AMOUNT);

        let msg = MarketplaceExecuteMsg::AcceptBid {
            token_id: NAME.to_string(),
            bidder: BIDDER.to_string(),
        };
        let res = app.execute_contract(Addr::unchecked(USER), Addr::unchecked(MKT), &msg, &[]);
        assert!(res.is_ok());

        bid(&mut app, NAME, BIDDER2, BID_AMOUNT);

        // have to approve marketplace spend for bid acceptor (bidder)
        let msg = Sg721NameExecuteMsg::Approve {
            spender: MKT.to_string(),
            token_id: NAME.to_string(),
            expires: None,
        };
        let res = app.execute_contract(
            Addr::unchecked(BIDDER),
            Addr::unchecked(COLLECTION),
            &msg,
            &[],
        );
        assert!(res.is_ok());

        let msg = MarketplaceExecuteMsg::AcceptBid {
            token_id: NAME.to_string(),
            bidder: BIDDER2.to_string(),
        };
        let res = app.execute_contract(Addr::unchecked(BIDDER), Addr::unchecked(MKT), &msg, &[]);
        assert!(res.is_ok());
    }

    #[test]
    fn test_reverse_map() {
        let mut app = instantiate_contracts(None, None, None);
        // needs to use valid address for querying addresses
        let user = "stars1hsk6jryyqjfhp5dhc55tc9jtckygx0eprx6sym";

        let res = mint_and_list(&mut app, NAME, user, None);
        assert!(res.is_ok());

        // when no associated address, query should throw error
        let res: Result<String, cosmwasm_std::StdError> = app.wrap().query_wasm_smart(
            COLLECTION,
            &(SgNameQueryMsg::AssociatedAddress {
                name: NAME.to_string(),
            }),
        );
        assert!(res.is_err());

        let msg = SgNameExecuteMsg::AssociateAddress {
            name: NAME.to_string(),
            address: Some(user.to_string()),
        };
        let res = app.execute_contract(
            Addr::unchecked(user),
            Addr::unchecked(COLLECTION),
            &msg,
            &[],
        );
        assert!(res.is_ok());

        // query associated address should return user
        let res: String = app
            .wrap()
            .query_wasm_smart(
                COLLECTION,
                &(SgNameQueryMsg::AssociatedAddress {
                    name: NAME.to_string(),
                }),
            )
            .unwrap();
        assert_eq!(res, user.to_string());

        // added to get around rate limiting
        update_block_time(&mut app, 60);

        // associate another
        let name2 = "exam";
        let res = mint_and_list(&mut app, name2, user, None);
        assert!(res.is_ok());

        let msg = SgNameExecuteMsg::AssociateAddress {
            name: name2.to_string(),
            address: Some(user.to_string()),
        };
        let res = app.execute_contract(
            Addr::unchecked(user),
            Addr::unchecked(COLLECTION),
            &msg,
            &[],
        );
        assert!(res.is_ok());

        let res: String = app
            .wrap()
            .query_wasm_smart(
                Addr::unchecked(COLLECTION),
                &(SgNameQueryMsg::Name {
                    address: user.to_string(),
                }),
            )
            .unwrap();
        assert_eq!(res, name2.to_string());

        // prev token_id should reset token_uri to None
        let res: NftInfoResponse<Metadata> = app
            .wrap()
            .query_wasm_smart(
                Addr::unchecked(COLLECTION),
                &(Sg721NameQueryMsg::NftInfo {
                    token_id: NAME.to_string(),
                }),
            )
            .unwrap();
        assert_eq!(res.token_uri, None);

        // token uri should be user address
        let res: NftInfoResponse<Metadata> = app
            .wrap()
            .query_wasm_smart(
                Addr::unchecked(COLLECTION),
                &(Sg721NameQueryMsg::NftInfo {
                    token_id: name2.to_string(),
                }),
            )
            .unwrap();
        assert_eq!(res.token_uri, Some(user.to_string()));

        // remove address
        let msg = SgNameExecuteMsg::AssociateAddress {
            name: NAME.to_string(),
            address: None,
        };
        let res = app.execute_contract(
            Addr::unchecked(user),
            Addr::unchecked(COLLECTION),
            &msg,
            &[],
        );
        assert!(res.is_ok());

        // confirm removed from nft info
        let res: NftInfoResponse<Metadata> = app
            .wrap()
            .query_wasm_smart(
                Addr::unchecked(COLLECTION),
                &(Sg721NameQueryMsg::NftInfo {
                    token_id: NAME.to_string(),
                }),
            )
            .unwrap();
        assert_eq!(res.token_uri, None);

        // remove address
        let msg = SgNameExecuteMsg::AssociateAddress {
            name: name2.to_string(),
            address: None,
        };
        let res = app.execute_contract(
            Addr::unchecked(user),
            Addr::unchecked(COLLECTION),
            &msg,
            &[],
        );
        assert!(res.is_ok());

        // confirm removed from reverse names map
        let res: Result<String, StdError> = app.wrap().query_wasm_smart(
            Addr::unchecked(COLLECTION),
            &(SgNameQueryMsg::Name {
                address: user.to_string(),
            }),
        );
        assert!(res.is_err());
    }

    #[test]
    fn test_reverse_map_contract_address() {
        let mut app = instantiate_contracts(None, None, None);

        let res = mint_and_list(&mut app, NAME, ADMIN2, None);
        assert!(res.is_ok());

        let msg = SgNameExecuteMsg::AssociateAddress {
            name: NAME.to_string(),
            address: Some(MINTER.to_string()),
        };
        let res = app.execute_contract(
            Addr::unchecked(ADMIN2),
            Addr::unchecked(COLLECTION),
            &msg,
            &[],
        );
        assert!(res.is_ok());

        res.unwrap().events.iter().for_each(|e| {
            if e.ty == "wasm-associate-address" {
                assert_eq!(e.attributes[1], attr("name", NAME));
                assert_eq!(e.attributes[2], attr("owner", ADMIN2));
                assert_eq!(e.attributes[3], attr("address", MINTER));
            }
        });
    }

    #[test]
    fn test_reverse_map_not_contract_address_admin() {
        let mut app = instantiate_contracts(None, None, None);

        let res = mint_and_list(&mut app, NAME, ADMIN2, None);
        assert!(res.is_ok());

        let msg = SgNameExecuteMsg::AssociateAddress {
            name: NAME.to_string(),
            address: Some(MINTER.to_string()),
        };
        let res = app.execute_contract(
            Addr::unchecked(USER),
            Addr::unchecked(COLLECTION),
            &msg,
            &[],
        );
        assert!(res.is_err());
    }

    #[test]
    fn test_reverse_map_not_owner() {
        let mut app = instantiate_contracts(None, None, None);

        let res = mint_and_list(&mut app, NAME, USER, None);
        assert!(res.is_ok());

        let msg = SgNameExecuteMsg::AssociateAddress {
            name: NAME.to_string(),
            address: Some(USER2.to_string()),
        };
        let res = app.execute_contract(
            Addr::unchecked(USER2),
            Addr::unchecked(COLLECTION),
            &msg,
            &[],
        );
        assert!(res.is_err());
    }

    #[test]
    fn test_pause() {
        let mut app = instantiate_contracts(None, Some(ADMIN.to_string()), None);

        // verify addr in wl
        let whitelists: Vec<Addr> = app
            .wrap()
            .query_wasm_smart(MINTER, &(QueryMsg::Whitelists {}))
            .unwrap();

        assert_eq!(whitelists.len(), 1);

        whitelists.iter().find(|whitelist| {
            let included: bool = app
                .wrap()
                .query_wasm_smart(
                    Addr::unchecked(whitelist.to_string()),
                    &(IncludesAddress {
                        address: USER.to_string(),
                    }),
                )
                .unwrap();
            dbg!(included, whitelist);
            included
        });

        let res = mint_and_list(&mut app, NAME, USER, None);
        assert!(res.is_ok());

        let msg = ExecuteMsg::Pause { pause: true };
        let res = app.execute_contract(Addr::unchecked(ADMIN), Addr::unchecked(MINTER), &msg, &[]);
        assert!(res.is_ok());

        let err = mint_and_list(&mut app, "name2", USER, None);
        assert!(err.is_err());
    }

    #[test]
    fn update_mkt_sudo() {
        let mut app = instantiate_contracts(None, None, None);

        let res = mint_and_list(&mut app, NAME, USER, None);
        assert!(res.is_ok());

        let msg = MarketplaceSudoMsg::UpdateParams {
            trading_fee_bps: Some(1000u64),
            min_price: Some(Uint128::from(1000u128)),
            ask_interval: Some(1000),
        };

        let res = app.wasm_sudo(Addr::unchecked(MKT), &msg);
        assert!(res.is_ok());

        let res: SudoParams = app
            .wrap()
            .query_wasm_smart(Addr::unchecked(MKT), &(QueryMsg::Params {}))
            .unwrap();
        let params = res;

        assert_eq!(params.trading_fee_percent, Decimal::percent(10));
        assert_eq!(params.min_price, Uint128::from(1000u128));
        assert_eq!(params.ask_interval, 1000);
    }
}

mod admin {
    use whitelist_updatable_flatrate::state::Config;

    use crate::msg::QueryMsg;

    use super::*;

    #[test]
    fn update_admin() {
        let mut app = instantiate_contracts(None, Some(ADMIN.to_string()), None);

        let msg = ExecuteMsg::UpdateAdmin { admin: None };
        let res = app.execute_contract(Addr::unchecked(USER), Addr::unchecked(MINTER), &msg, &[]);
        assert!(res.is_err());

        let msg = ExecuteMsg::UpdateAdmin {
            admin: Some(USER2.to_string()),
        };
        let res = app.execute_contract(Addr::unchecked(ADMIN), Addr::unchecked(MINTER), &msg, &[]);
        assert!(res.is_ok());

        let msg = ExecuteMsg::UpdateAdmin { admin: None };
        let res = app.execute_contract(Addr::unchecked(USER2), Addr::unchecked(MINTER), &msg, &[]);
        assert!(res.is_ok());

        // cannot update admin after its been removed
        let msg = ExecuteMsg::UpdateAdmin { admin: None };
        let res = app.execute_contract(Addr::unchecked(USER2), Addr::unchecked(MINTER), &msg, &[]);
        assert!(res.is_err());
    }

    #[test]
    fn mint_from_whitelist() {
        let mut app = instantiate_contracts(None, Some(ADMIN.to_string()), None);

        let msg = QueryMsg::Whitelists {};
        let whitelists: Vec<Addr> = app.wrap().query_wasm_smart(MINTER, &msg).unwrap();
        assert_eq!(whitelists.len(), 1);

        let msg = WhitelistQueryMsg::Config {};
        let res: Config = app.wrap().query_wasm_smart(WHITELIST, &msg).unwrap();
        assert_eq!(res.admins, [ADMIN2.to_string()]);

        let msg = WhitelistQueryMsg::AddressCount {};
        let count: u64 = app.wrap().query_wasm_smart(WHITELIST, &msg).unwrap();
        assert_eq!(count, 5);

        let msg = WhitelistQueryMsg::IncludesAddress {
            address: USER.to_string(),
        };
        let includes: bool = app.wrap().query_wasm_smart(WHITELIST, &msg).unwrap();
        assert!(includes);

        let res = mint_and_list(&mut app, NAME, USER, None);
        assert!(res.is_ok());

        // make rate limiter happy
        update_block_time(&mut app, 100);

        let res = mint_and_list(&mut app, "ser1", USER, None);
        assert!(res.is_ok());

        // make rate limiter happy
        update_block_time(&mut app, 100);

        let res = mint_and_list(&mut app, "ser2", USER, None);
        assert!(res.is_err());
    }
}

mod query {
    use cosmwasm_std::coin;
    use cosmwasm_std::Coin;
    use cosmwasm_std::StdResult;
    use name_marketplace::msg::AskRenewPriceResponse;
    use name_marketplace::msg::BidOffset;
    use name_marketplace::state::Ask;
    use sg721_base::msg::CollectionInfoResponse;
    use sg721_base::msg::QueryMsg as Sg721QueryMsg;

    use super::*;

    #[test]
    fn query_ask() {
        let mut app = instantiate_contracts(None, None, None);

        let res = mint_and_list(&mut app, NAME, USER, None);
        assert!(res.is_ok());

        let msg = MarketplaceQueryMsg::Ask {
            token_id: NAME.to_string(),
        };
        let res: Option<Ask> = app.wrap().query_wasm_smart(MKT, &msg).unwrap();
        assert_eq!(res.unwrap().token_id, NAME.to_string());
    }

    #[test]
    fn query_asks() {
        let mut app = instantiate_contracts(None, None, None);

        let res = mint_and_list(&mut app, NAME, USER, None);
        assert!(res.is_ok());

        let res = mint_and_list(&mut app, "hack", ADMIN2, None);
        assert!(res.is_ok());

        let msg = MarketplaceQueryMsg::Asks {
            start_after: None,
            limit: None,
        };
        let res: Vec<Ask> = app.wrap().query_wasm_smart(MKT, &msg).unwrap();
        assert_eq!(res[0].id, 1);
    }

    #[test]
    fn query_asks_by_seller() {
        let mut app = instantiate_contracts(None, None, None);

        let res = mint_and_list(&mut app, NAME, USER, None);
        assert!(res.is_ok());

        let res = mint_and_list(&mut app, "hack", USER2, None);
        assert!(res.is_ok());

        let msg = MarketplaceQueryMsg::AsksBySeller {
            seller: USER.to_string(),
            start_after: None,
            limit: None,
        };
        let res: Vec<Ask> = app.wrap().query_wasm_smart(MKT, &msg).unwrap();
        assert_eq!(res.len(), 1);
    }

    #[test]
    fn query_ask_count() {
        let mut app = instantiate_contracts(None, None, None);

        let res = mint_and_list(&mut app, NAME, USER, None);
        assert!(res.is_ok());

        let res = mint_and_list(&mut app, "hack", ADMIN2, None);
        assert!(res.is_ok());

        let msg = MarketplaceQueryMsg::AskCount {};
        let res: u64 = app.wrap().query_wasm_smart(MKT, &msg).unwrap();
        assert_eq!(res, 2);
    }

    #[test]
    fn query_top_bids() {
        let mut app = instantiate_contracts(None, None, None);

        let res = mint_and_list(&mut app, NAME, USER, None);
        assert!(res.is_ok());

        bid(&mut app, NAME, BIDDER, BID_AMOUNT);
        bid(&mut app, NAME, BIDDER2, BID_AMOUNT * 5);

        let msg = MarketplaceQueryMsg::ReverseBidsSortedByPrice {
            start_before: None,
            limit: None,
        };
        let res: Vec<Bid> = app.wrap().query_wasm_smart(MKT, &msg).unwrap();
        assert_eq!(res.len(), 2);
        assert_eq!(res[0].amount.u128(), BID_AMOUNT * 5);
    }

    #[test]
    fn query_bids_by_seller() {
        let mut app = instantiate_contracts(None, None, None);

        let res = mint_and_list(&mut app, NAME, USER, None);
        assert!(res.is_ok());

        bid(&mut app, NAME, BIDDER, BID_AMOUNT);
        bid(&mut app, NAME, BIDDER2, BID_AMOUNT * 5);

        let msg = MarketplaceQueryMsg::BidsForSeller {
            seller: USER.to_string(),
            start_after: None,
            limit: None,
        };
        let res: Vec<Bid> = app.wrap().query_wasm_smart(MKT, &msg).unwrap();
        assert_eq!(res.len(), 2);
        assert_eq!(res[0].amount.u128(), BID_AMOUNT);

        // test pagination
        let bid_offset = BidOffset {
            price: Uint128::from(BID_AMOUNT),
            bidder: Addr::unchecked(BIDDER),
            token_id: NAME.to_string(),
        };
        let msg = MarketplaceQueryMsg::BidsForSeller {
            seller: USER.to_string(),
            start_after: Some(bid_offset),
            limit: None,
        };
        let res: Vec<Bid> = app.wrap().query_wasm_smart(MKT, &msg).unwrap();
        // should be length 0 because there are no token_ids besides NAME.to_string()
        assert_eq!(res.len(), 0);

        // added to get around rate limiting
        update_block_time(&mut app, 60);

        // test pagination with multiple names and bids
        let name = "jump";
        let res = mint_and_list(&mut app, name, USER, None);
        assert!(res.is_ok());
        bid(&mut app, name, BIDDER, BID_AMOUNT * 3);
        bid(&mut app, name, BIDDER2, BID_AMOUNT * 2);
        let res: Vec<Bid> = app.wrap().query_wasm_smart(MKT, &msg).unwrap();
        // should be length 2 because there is token_id "jump" with 2 bids
        assert_eq!(res.len(), 2);
    }

    #[test]
    fn query_highest_bid() {
        let mut app = instantiate_contracts(None, None, None);

        let res = mint_and_list(&mut app, NAME, USER, None);
        assert!(res.is_ok());

        bid(&mut app, NAME, BIDDER, BID_AMOUNT);
        bid(&mut app, NAME, BIDDER2, BID_AMOUNT * 5);

        let msg = MarketplaceQueryMsg::HighestBid {
            token_id: NAME.to_string(),
        };
        let res: Option<Bid> = app.wrap().query_wasm_smart(MKT, &msg).unwrap();
        assert_eq!(res.unwrap().amount.u128(), BID_AMOUNT * 5);
    }

    #[test]
    fn query_renewal_queue() {
        let mut app = instantiate_contracts(None, None, None);

        // mint two names at the same time
        let res = mint_and_list(&mut app, NAME, USER, None);
        assert!(res.is_ok());
        let res = mint_and_list(&mut app, "hack", ADMIN2, None);
        assert!(res.is_ok());

        let res: Vec<Ask> = app
            .wrap()
            .query_wasm_smart(
                MKT,
                &(MarketplaceQueryMsg::RenewalQueue {
                    time: app.block_info().time.plus_seconds(SECONDS_PER_YEAR),
                }),
            )
            .unwrap();
        assert_eq!(res.len(), 2);
        assert_eq!(res[1].token_id, "hack".to_string());
    }

    #[test]
    fn renewal_fee_transfer_refund() {
        let mut app = instantiate_contracts(None, None, None);

        mint_and_list(&mut app, NAME, USER, None).unwrap();

        // mint 1000 funds to user
        let renewal_fee = coins(1000_u128, NATIVE_DENOM);

        app.sudo(CwSudoMsg::Bank({
            BankSudo::Mint {
                to_address: USER.to_string(),
                amount: renewal_fee.clone(),
            }
        }))
        .unwrap();

        // user renew domain name
        let msg = MarketplaceExecuteMsg::FundRenewal {
            token_id: NAME.to_string(),
        };

        app.execute_contract(
            Addr::unchecked(USER),
            Addr::unchecked(MKT),
            &msg,
            &renewal_fee,
        )
        .unwrap();

        // verify user have no money
        let res = app
            .wrap()
            .query_balance(USER.to_string(), NATIVE_DENOM)
            .unwrap();
        assert_eq!(res.amount, Uint128::new(0));

        // user sends the nft to bob
        let bob: &str = "bob";

        let msg = Sg721NameExecuteMsg::TransferNft {
            recipient: bob.to_string(),
            token_id: NAME.to_string(),
        };
        app.execute_contract(
            Addr::unchecked(USER),
            Addr::unchecked(COLLECTION),
            &msg,
            &[],
        )
        .unwrap();

        // the renewal fee should refunded back to user
        let user_balance = app
            .wrap()
            .query_balance(USER.to_string(), NATIVE_DENOM)
            .unwrap()
            .amount;
        assert_eq!(user_balance, renewal_fee[0].amount);

        let bob_balance = app.wrap().query_balance(bob, NATIVE_DENOM).unwrap().amount;
        assert_eq!(bob_balance, Uint128::zero());
    }

    #[test]
    fn renew_price_no_bid() {
        let mut app = instantiate_contracts(None, None, None);

        mint_and_list(&mut app, NAME, USER, None).unwrap();

        let res: StdResult<String> = app.wrap().query_wasm_smart(
            COLLECTION,
            &SgNameQueryMsg::Name {
                address: USER.to_string(),
            },
        );
        assert!(res.is_err());

        let result = app.wrap().query_wasm_smart::<(Option<Coin>, Option<Bid>)>(
            MKT,
            &MarketplaceQueryMsg::AskRenewPrice {
                current_time: app.block_info().time,
                token_id: NAME.to_string(),
            },
        );
        assert!(result.is_ok());
        // NOTE: disabling to support prefunding before renewal window
        // let (renewal_price, _renewal_bid) = result.unwrap();
        // assert!(renewal_price.is_none());

        update_block_time(&mut app, SECONDS_PER_YEAR - (60 * 60 * 24 * 30));

        let result = app.wrap().query_wasm_smart::<(Option<Coin>, Option<Bid>)>(
            MKT,
            &MarketplaceQueryMsg::AskRenewPrice {
                current_time: app.block_info().time,
                token_id: NAME.to_string(),
            },
        );
        assert!(result.is_ok());

        let (renewal_price, _renewal_bid) = result.unwrap();

        let params: NameMinterParams = app
            .wrap()
            .query_wasm_smart(MINTER, &SgNameMinterQueryMsg::Params {})
            .unwrap();

        let char_price = get_char_price(params.base_price.u128(), NAME.len());

        println!("char_price: {}", char_price);

        assert_eq!(renewal_price.unwrap().amount, char_price);
    }

    #[test]
    fn renew_price_with_bid() {
        let mut app = instantiate_contracts(None, None, None);

        mint_and_list(&mut app, NAME, USER, None).unwrap();

        // Amount to make it just above the char price
        let bid_amount = 1_000_000_000u128 * 201u128;

        update_block_time(&mut app, SECONDS_PER_YEAR - (60 * 60 * 24 * 31));

        bid(&mut app, NAME, BIDDER, bid_amount);

        update_block_time(&mut app, 60 * 60 * 24 * 31);

        let result = app.wrap().query_wasm_smart::<(Option<Coin>, Option<Bid>)>(
            MKT,
            &MarketplaceQueryMsg::AskRenewPrice {
                current_time: app.block_info().time,
                token_id: NAME.to_string(),
            },
        );
        assert!(result.is_ok());

        let (renewal_price, _renewal_bid) = result.unwrap();

        let params: NameMarketplaceParams = app
            .wrap()
            .query_wasm_smart(MKT, &NameMarketplaceQueryMsg::Params {})
            .unwrap();

        let expect_price = Uint128::from(bid_amount) * params.renewal_bid_percentage;
        assert_eq!(renewal_price.unwrap().amount, expect_price);
    }

    #[test]
    fn multiple_renew_prices() {
        // test that QueryRenewPrices returns multiple entires
        let mut app = instantiate_contracts(None, None, None);

        mint_and_list(&mut app, NAME, USER, None).unwrap();
        mint_and_list(&mut app, NAME2, USER, None).unwrap();

        // Amount to make it just above the char price
        let bid_amount = 1_000_000_000u128 * 201u128;

        update_block_time(&mut app, SECONDS_PER_YEAR - (60 * 60 * 24 * 31));

        bid(&mut app, NAME, BIDDER, bid_amount);
        bid(&mut app, NAME2, BIDDER, bid_amount);

        update_block_time(&mut app, 60 * 60 * 24 * 31);

        let result = app.wrap().query_wasm_smart::<Vec<AskRenewPriceResponse>>(
            MKT,
            &MarketplaceQueryMsg::AskRenewalPrices {
                current_time: app.block_info().time,
                token_ids: vec![NAME.to_string(), NAME2.to_string()],
            },
        );
        println!("{:?}", result);
        assert!(result.is_ok());
    }

    #[test]
    fn renew_execute_msg() {
        let mut app = instantiate_contracts(None, None, None);

        mint_and_list(&mut app, NAME, USER, None).unwrap();

        // Amount to make it just above the char price
        let bid_amount = 1_000_000_000u128 * 201u128;

        update_block_time(&mut app, SECONDS_PER_YEAR - (60 * 60 * 24 * 30));

        bid(&mut app, NAME, BIDDER, bid_amount);

        update_block_time(&mut app, 1);

        let result = app.wrap().query_wasm_smart::<(Option<Coin>, Option<Bid>)>(
            MKT,
            &MarketplaceQueryMsg::AskRenewPrice {
                current_time: app.block_info().time,
                token_id: NAME.to_string(),
            },
        );
        assert!(result.is_ok());

        let (renewal_price, _renewal_bid) = result.unwrap();
        let renewal_amount = renewal_price.unwrap().amount;

        let fund_amount = coins(renewal_amount.u128() * 100_u128, NATIVE_DENOM);
        app.sudo(CwSudoMsg::Bank({
            BankSudo::Mint {
                to_address: USER.to_string(),
                amount: fund_amount,
            }
        }))
        .map_err(|err| println!("{:?}", err))
        .ok();

        let ask = app
            .wrap()
            .query_wasm_smart::<Option<Ask>>(
                MKT,
                &MarketplaceQueryMsg::Ask {
                    token_id: NAME.to_string(),
                },
            )
            .unwrap()
            .unwrap();
        let expected_renewal_time = ask.renewal_time.plus_seconds(SECONDS_PER_YEAR);

        let result = app.execute_contract(
            Addr::unchecked(USER),
            Addr::unchecked(MKT),
            &MarketplaceExecuteMsg::Renew {
                token_id: NAME.to_string(),
            },
            &[coin(renewal_amount.u128(), NATIVE_DENOM)],
        );
        assert!(result.is_ok());

        let result = app.wrap().query_wasm_smart::<Option<Ask>>(
            MKT,
            &MarketplaceQueryMsg::Ask {
                token_id: NAME.to_string(),
            },
        );
        assert!(result.is_ok());
        let ask = result.unwrap().unwrap();

        assert_eq!(ask.renewal_fund, Uint128::zero());
        assert_eq!(ask.renewal_time, expected_renewal_time);

        let result = app.wrap().query_wasm_smart::<Vec<Ask>>(
            MKT,
            &MarketplaceQueryMsg::RenewalQueue {
                time: expected_renewal_time,
            },
        );
        assert!(result.is_ok());

        let asks = result.unwrap();
        assert_eq!(asks.len(), 1);
    }

    #[test]
    fn process_renewals_renew() {
        let mut app = instantiate_contracts(None, None, None);

        mint_and_list(&mut app, NAME, USER, None).unwrap();

        update_block_time(&mut app, SECONDS_PER_YEAR - (60 * 60 * 24 * 30));

        let response = app.wrap().query_wasm_smart::<(Option<Coin>, Option<Bid>)>(
            MKT,
            &MarketplaceQueryMsg::AskRenewPrice {
                current_time: app.block_info().time,
                token_id: NAME.to_string(),
            },
        );
        assert!(response.is_ok());
        let renewal_price = response.unwrap().0.unwrap();

        let fund_amount = coins(renewal_price.amount.u128() * 100_u128, NATIVE_DENOM);
        app.sudo(CwSudoMsg::Bank({
            BankSudo::Mint {
                to_address: USER.to_string(),
                amount: fund_amount,
            }
        }))
        .map_err(|err| println!("{:?}", err))
        .ok();
        let result = app.execute_contract(
            Addr::unchecked(USER),
            Addr::unchecked(MKT),
            &MarketplaceExecuteMsg::FundRenewal {
                token_id: NAME.to_string(),
            },
            &[coin(renewal_price.amount.u128(), NATIVE_DENOM)],
        );
        assert!(result.is_ok());

        update_block_time(&mut app, 60 * 60 * 24 * 30);

        let result = app.execute_contract(
            Addr::unchecked(OPERATOR),
            Addr::unchecked(MKT),
            &MarketplaceExecuteMsg::ProcessRenewals { limit: 1 },
            &[],
        );
        assert!(result.is_ok());

        let result = app.wrap().query_wasm_smart::<Option<Ask>>(
            MKT,
            &MarketplaceQueryMsg::Ask {
                token_id: NAME.to_string(),
            },
        );
        assert!(result.is_ok());
        let ask = result.unwrap().unwrap();

        assert_eq!(ask.seller, USER.to_string());
        assert_eq!(ask.renewal_fund, renewal_price.amount);

        let expected_renewal_time = app.block_info().time.plus_seconds(SECONDS_PER_YEAR);
        assert_eq!(ask.renewal_time, expected_renewal_time);

        let result = app.wrap().query_wasm_smart::<Vec<Ask>>(
            MKT,
            &MarketplaceQueryMsg::RenewalQueue {
                time: expected_renewal_time,
            },
        );
        assert!(result.is_ok());

        let asks = result.unwrap();
        assert_eq!(asks.len(), 1);
    }

    #[test]
    fn process_renewals_sell() {
        let mut app = instantiate_contracts(None, None, None);

        mint_and_list(&mut app, NAME, USER, None).unwrap();

        // Amount to make it just above the char price
        let bid_amount = 1_000_000_000u128 * 201u128;

        update_block_time(&mut app, SECONDS_PER_YEAR - (60 * 60 * 24 * 31));

        bid(&mut app, NAME, BIDDER, bid_amount);

        update_block_time(&mut app, 60 * 60 * 24 * 31);

        let response = app.wrap().query_wasm_smart::<(Option<Coin>, Option<Bid>)>(
            MKT,
            &MarketplaceQueryMsg::AskRenewPrice {
                current_time: app.block_info().time,
                token_id: NAME.to_string(),
            },
        );
        println!("xxx {:?}", response);
        assert!(response.is_ok());
        let response = response.unwrap();

        assert!(response.1.is_some());
        let renewal_price = response.0.unwrap();

        let fund_amount = coins(renewal_price.amount.u128() * 100_u128, NATIVE_DENOM);
        app.sudo(CwSudoMsg::Bank({
            BankSudo::Mint {
                to_address: USER.to_string(),
                amount: fund_amount,
            }
        }))
        .map_err(|err| println!("{:?}", err))
        .ok();
        let result = app.execute_contract(
            Addr::unchecked(USER),
            Addr::unchecked(MKT),
            &MarketplaceExecuteMsg::FundRenewal {
                token_id: NAME.to_string(),
            },
            &[coin(renewal_price.amount.u128() - 1u128, NATIVE_DENOM)],
        );
        assert!(result.is_ok());

        let user_balance_before = app
            .wrap()
            .query_balance(USER.to_string(), NATIVE_DENOM)
            .unwrap();

        let result = app.execute_contract(
            Addr::unchecked(OPERATOR),
            Addr::unchecked(MKT),
            &MarketplaceExecuteMsg::ProcessRenewals { limit: 1 },
            &[],
        );
        assert!(result.is_ok());

        let result = app.wrap().query_wasm_smart::<Option<Ask>>(
            MKT,
            &MarketplaceQueryMsg::Ask {
                token_id: NAME.to_string(),
            },
        );
        assert!(result.is_ok());
        let ask = result.unwrap().unwrap();

        assert_eq!(ask.seller, BIDDER.to_string());
        assert_eq!(ask.renewal_fund, Uint128::zero());

        let expected_renewal_time = app.block_info().time.plus_seconds(SECONDS_PER_YEAR);
        assert_eq!(ask.renewal_time, expected_renewal_time);

        let result = app.wrap().query_wasm_smart::<Vec<Ask>>(
            MKT,
            &MarketplaceQueryMsg::RenewalQueue {
                time: expected_renewal_time,
            },
        );
        assert!(result.is_ok());

        let asks = result.unwrap();
        assert_eq!(asks.len(), 1);

        let user_balance_after = app
            .wrap()
            .query_balance(USER.to_string(), NATIVE_DENOM)
            .unwrap();
        assert!(user_balance_before.amount < user_balance_after.amount);
    }

    #[test]
    fn process_renewals_renew_free() {
        let mut app = instantiate_contracts(None, None, None);

        mint_and_list(&mut app, NAME, USER, None).unwrap();

        update_block_time(&mut app, SECONDS_PER_YEAR - (60 * 60 * 24 * 30));

        let response = app.wrap().query_wasm_smart::<(Option<Coin>, Option<Bid>)>(
            MKT,
            &MarketplaceQueryMsg::AskRenewPrice {
                current_time: app.block_info().time,
                token_id: NAME.to_string(),
            },
        );
        assert!(response.is_ok());
        let renewal_price = response.unwrap().0.unwrap();

        // send set the fund_amount to the exact price
        let fund_amount = coins(renewal_price.amount.u128(), NATIVE_DENOM);
        app.sudo(CwSudoMsg::Bank({
            BankSudo::Mint {
                to_address: USER.to_string(),
                amount: fund_amount,
            }
        }))
        .map_err(|err| println!("{:?}", err))
        .ok();
        let result = app.execute_contract(
            Addr::unchecked(USER),
            Addr::unchecked(MKT),
            &MarketplaceExecuteMsg::FundRenewal {
                token_id: NAME.to_string(),
            },
            &[coin(renewal_price.amount.u128(), NATIVE_DENOM)],
        );
        assert!(result.is_ok());

        update_block_time(&mut app, 60 * 60 * 24 * 30);

        let result = app.execute_contract(
            Addr::unchecked(OPERATOR),
            Addr::unchecked(MKT),
            &MarketplaceExecuteMsg::ProcessRenewals { limit: 1 },
            &[],
        );
        assert!(result.is_ok());

        let result = app.wrap().query_wasm_smart::<Option<Ask>>(
            MKT,
            &MarketplaceQueryMsg::Ask {
                token_id: NAME.to_string(),
            },
        );
        assert!(result.is_ok());
        assert!(result.unwrap().is_some());
    }

    #[test]
    fn allow_partial_funded_renewal() {
        let mut app = instantiate_contracts(None, None, None);

        mint_and_list(&mut app, NAME, USER, None).unwrap();

        update_block_time(&mut app, SECONDS_PER_YEAR - (60 * 60 * 24 * 30));

        let response = app.wrap().query_wasm_smart::<(Option<Coin>, Option<Bid>)>(
            MKT,
            &MarketplaceQueryMsg::AskRenewPrice {
                current_time: app.block_info().time,
                token_id: NAME.to_string(),
            },
        );
        assert!(response.is_ok());
        let renewal_price = response.unwrap().0.unwrap();

        // try to over-fund renewal
        let fund_amount = coins(renewal_price.amount.u128(), NATIVE_DENOM);
        app.sudo(CwSudoMsg::Bank({
            BankSudo::Mint {
                to_address: USER.to_string(),
                amount: fund_amount,
            }
        }))
        .map_err(|err| println!("{:?}", err))
        .ok();
        let result = app.execute_contract(
            Addr::unchecked(USER),
            Addr::unchecked(MKT),
            &MarketplaceExecuteMsg::FundRenewal {
                token_id: NAME.to_string(),
            },
            &[coin(renewal_price.amount.u128() - 100u128, NATIVE_DENOM)],
        );
        assert!(result.is_ok());
    }

    #[test]
    fn reject_overfunded_renewal() {
        let mut app = instantiate_contracts(None, None, None);

        mint_and_list(&mut app, NAME, USER, None).unwrap();

        update_block_time(&mut app, SECONDS_PER_YEAR - (60 * 60 * 24 * 30));

        let response = app.wrap().query_wasm_smart::<(Option<Coin>, Option<Bid>)>(
            MKT,
            &MarketplaceQueryMsg::AskRenewPrice {
                current_time: app.block_info().time,
                token_id: NAME.to_string(),
            },
        );
        assert!(response.is_ok());
        let renewal_price = response.unwrap().0.unwrap();

        // try to over-fund renewal
        let fund_amount = coins(renewal_price.amount.u128(), NATIVE_DENOM);
        app.sudo(CwSudoMsg::Bank({
            BankSudo::Mint {
                to_address: USER.to_string(),
                amount: fund_amount,
            }
        }))
        .map_err(|err| println!("{:?}", err))
        .ok();
        let result = app.execute_contract(
            Addr::unchecked(USER),
            Addr::unchecked(MKT),
            &MarketplaceExecuteMsg::FundRenewal {
                token_id: NAME.to_string(),
            },
            &[coin(renewal_price.amount.u128() + 100u128, NATIVE_DENOM)],
        );
        assert!(result.is_err());
    }

    #[test]
    fn query_name() {
        let mut app = instantiate_contracts(None, None, None);

        let res = mint_and_list(&mut app, NAME, USER, None);
        assert!(res.is_ok());

        // fails with "user" string, has to be a bech32 address
        let res: StdResult<String> = app.wrap().query_wasm_smart(
            COLLECTION,
            &(SgNameQueryMsg::Name {
                address: USER.to_string(),
            }),
        );
        assert!(res.is_err());

        let user = "stars1hsk6jryyqjfhp5dhc55tc9jtckygx0eprx6sym";
        let cosmos_address = "cosmos1hsk6jryyqjfhp5dhc55tc9jtckygx0eph6dd02";

        let res = mint_and_list(&mut app, "yoyo", user, None);
        assert!(res.is_ok());

        let msg = SgNameExecuteMsg::AssociateAddress {
            name: "yoyo".to_string(),
            address: Some(user.to_string()),
        };
        let res = app.execute_contract(
            Addr::unchecked(user),
            Addr::unchecked(COLLECTION),
            &msg,
            &[],
        );
        assert!(res.is_ok());

        let res: String = app
            .wrap()
            .query_wasm_smart(
                COLLECTION,
                &(SgNameQueryMsg::Name {
                    address: cosmos_address.to_string(),
                }),
            )
            .unwrap();
        assert_eq!(res, "yoyo".to_string());
    }

    #[test]
    fn query_trading_start_time() {
        let app = instantiate_contracts(None, None, None);

        let res: CollectionInfoResponse = app
            .wrap()
            .query_wasm_smart(COLLECTION, &(Sg721QueryMsg::CollectionInfo {}))
            .unwrap();
        assert_eq!(
            res.start_trading_time.unwrap(),
            app.block_info()
                .time
                .plus_seconds(TRADING_START_TIME_OFFSET_IN_SECONDS)
        );
    }
}

mod collection {
    use cosmwasm_std::{to_json_binary, StdResult};
    use cw721::NftInfoResponse;
    use cw_controllers::AdminResponse;
    use name_marketplace::state::Ask;
    use sg721_name::{msg::QueryMsg as Sg721NameQueryMsg, state::SudoParams};
    use sg_name::{Metadata, TextRecord, NFT};

    use super::*;

    pub(crate) fn transfer(app: &mut StargazeApp, from: &str, to: &str) {
        let msg = Sg721NameExecuteMsg::TransferNft {
            recipient: to.to_string(),
            token_id: NAME.to_string(),
        };
        let res = app.execute_contract(
            Addr::unchecked(from),
            Addr::unchecked(COLLECTION),
            &msg,
            &[],
        );
        assert!(res.is_ok());

        let msg = MarketplaceQueryMsg::Ask {
            token_id: NAME.to_string(),
        };
        let res: Option<Ask> = app.wrap().query_wasm_smart(MKT, &msg).unwrap();
        assert_eq!(res.unwrap().seller.to_string(), to.to_string());
    }

    fn send(app: &mut StargazeApp, from: &str, to: &str) {
        let msg = to_json_binary("You now have the melting power").unwrap();
        let target = to.to_string();
        let send_msg = Sg721NameExecuteMsg::SendNft {
            contract: target,
            token_id: NAME.to_string(),
            msg,
        };
        let res = app.execute_contract(
            Addr::unchecked(from),
            Addr::unchecked(COLLECTION),
            &send_msg,
            &[],
        );
        assert!(res.is_ok());

        let msg = MarketplaceQueryMsg::Ask {
            token_id: NAME.to_string(),
        };
        let res: Option<Ask> = app.wrap().query_wasm_smart(MKT, &msg).unwrap();
        assert_eq!(res.unwrap().seller.to_string(), to.to_string());
    }

    #[test]
    fn verify_twitter() {
        let mut app = instantiate_contracts(None, None, None);

        let res = mint_and_list(&mut app, NAME, USER, None);
        assert!(res.is_ok());

        let name = "twitter";
        let value = "shan3v";

        let msg = SgNameExecuteMsg::AddTextRecord {
            name: NAME.to_string(),
            record: TextRecord::new(name, value),
        };
        let res = app.execute_contract(
            Addr::unchecked(USER),
            Addr::unchecked(COLLECTION),
            &msg,
            &[],
        );
        assert!(res.is_ok());

        // query text record to see if verified is not set
        let res: NftInfoResponse<Metadata> = app
            .wrap()
            .query_wasm_smart(
                COLLECTION,
                &(Sg721NameQueryMsg::NftInfo {
                    token_id: NAME.to_string(),
                }),
            )
            .unwrap();
        assert_eq!(res.extension.records[0].name, name.to_string());
        assert_eq!(res.extension.records[0].verified, None);

        let msg = SgNameExecuteMsg::VerifyTextRecord {
            name: NAME.to_string(),
            record_name: name.to_string(),
            result: true,
        };
        let res = app.execute_contract(
            Addr::unchecked(USER),
            Addr::unchecked(COLLECTION),
            &msg,
            &[],
        );
        // fails cuz caller is not oracle verifier
        assert!(res.is_err());

        let res = app.execute_contract(
            Addr::unchecked(VERIFIER),
            Addr::unchecked(COLLECTION),
            &msg,
            &[],
        );
        assert!(res.is_ok());

        let msg = Sg721NameQueryMsg::Verifier {};
        let verifier: AdminResponse = app.wrap().query_wasm_smart(COLLECTION, &msg).unwrap();
        assert_eq!(verifier.admin, Some(VERIFIER.to_string()));

        // query text record to see if verified is set
        let res: NftInfoResponse<Metadata> = app
            .wrap()
            .query_wasm_smart(
                COLLECTION,
                &(Sg721NameQueryMsg::NftInfo {
                    token_id: NAME.to_string(),
                }),
            )
            .unwrap();
        assert_eq!(res.extension.records[0].name, name.to_string());
        assert_eq!(res.extension.records[0].verified, Some(true));
    }

    #[test]
    fn verify_false() {
        let mut app = instantiate_contracts(None, None, None);

        let res = mint_and_list(&mut app, NAME, USER, None);
        assert!(res.is_ok());

        let name = "twitter";
        let value = "shan3v";

        let msg = SgNameExecuteMsg::AddTextRecord {
            name: NAME.to_string(),
            record: TextRecord::new(name, value),
        };
        let res = app.execute_contract(
            Addr::unchecked(USER),
            Addr::unchecked(COLLECTION),
            &msg,
            &[],
        );
        assert!(res.is_ok());

        // verify something as false
        let msg = SgNameExecuteMsg::VerifyTextRecord {
            name: NAME.to_string(),
            record_name: name.to_string(),
            result: false,
        };
        let res = app.execute_contract(
            Addr::unchecked(VERIFIER),
            Addr::unchecked(COLLECTION),
            &msg,
            &[],
        );
        assert!(res.is_ok());

        // query text record to see if verified is set
        let res: NftInfoResponse<Metadata> = app
            .wrap()
            .query_wasm_smart(
                COLLECTION,
                &(Sg721NameQueryMsg::NftInfo {
                    token_id: NAME.to_string(),
                }),
            )
            .unwrap();
        assert_eq!(res.extension.records[0].name, name.to_string());
        assert_eq!(res.extension.records[0].verified, Some(false));
    }

    #[test]
    fn verified_text_record() {
        let mut app = instantiate_contracts(None, None, None);

        let res = mint_and_list(&mut app, NAME, USER, None);
        assert!(res.is_ok());

        let name = "twitter";
        let value = "shan3v";

        let msg = SgNameExecuteMsg::AddTextRecord {
            name: NAME.to_string(),
            record: TextRecord {
                name: name.to_string(),
                value: value.to_string(),
                verified: Some(true),
            },
        };

        let res = app.execute_contract(
            Addr::unchecked(USER),
            Addr::unchecked(COLLECTION),
            &msg,
            &[],
        );
        assert!(res.is_ok());

        // query text record to see if verified is not set
        let res: NftInfoResponse<Metadata> = app
            .wrap()
            .query_wasm_smart(
                COLLECTION,
                &(Sg721NameQueryMsg::NftInfo {
                    token_id: NAME.to_string(),
                }),
            )
            .unwrap();
        assert_eq!(res.extension.records[0].name, name.to_string());
        assert_eq!(res.extension.records[0].verified, None);

        // attempt update text record w verified value
        let msg = SgNameExecuteMsg::UpdateTextRecord {
            name: NAME.to_string(),
            record: TextRecord {
                name: name.to_string(),
                value: "some new value".to_string(),
                verified: Some(true),
            },
        };
        let res = app.execute_contract(
            Addr::unchecked(USER),
            Addr::unchecked(COLLECTION),
            &msg,
            &[],
        );
        assert!(res.is_ok());

        // query text record to see if verified is not set
        let res: NftInfoResponse<Metadata> = app
            .wrap()
            .query_wasm_smart(
                COLLECTION,
                &(Sg721NameQueryMsg::NftInfo {
                    token_id: NAME.to_string(),
                }),
            )
            .unwrap();
        assert_eq!(res.extension.records[0].name, name.to_string());
        assert_eq!(res.extension.records[0].verified, None);

        // query text record to see if verified is not set
        let res: NftInfoResponse<Metadata> = app
            .wrap()
            .query_wasm_smart(
                COLLECTION,
                &(Sg721NameQueryMsg::NftInfo {
                    token_id: NAME.to_string(),
                }),
            )
            .unwrap();
        assert_eq!(res.extension.records[0].name, name.to_string());
        assert_eq!(res.extension.records[0].verified, None);

        // query image nft
        let res: Option<NFT> = app
            .wrap()
            .query_wasm_smart(
                COLLECTION,
                &(Sg721NameQueryMsg::ImageNFT {
                    name: NAME.to_string(),
                }),
            )
            .unwrap();
        assert!(res.is_none());
    }

    #[test]
    fn transfer_nft() {
        let mut app = instantiate_contracts(None, None, None);

        let res = mint_and_list(&mut app, NAME, USER, None);
        assert!(res.is_ok());

        transfer(&mut app, USER, USER2);
    }

    #[test]
    fn send_nft() {
        let mut app = instantiate_contracts(None, None, None);

        let res = mint_and_list(&mut app, NAME, USER, None);
        assert!(res.is_ok());

        send(&mut app, USER, USER2);
    }

    #[test]
    fn transfer_nft_and_bid() {
        let mut app = instantiate_contracts(None, None, None);

        let res = mint_and_list(&mut app, NAME, USER, None);
        assert!(res.is_ok());

        transfer(&mut app, USER, USER2);

        bid(&mut app, NAME, BIDDER, BID_AMOUNT);

        // user2 must approve the marketplace to transfer their name
        let msg = Sg721NameExecuteMsg::Approve {
            spender: MKT.to_string(),
            token_id: NAME.to_string(),
            expires: None,
        };
        let res = app.execute_contract(
            Addr::unchecked(USER2),
            Addr::unchecked(COLLECTION),
            &msg,
            &[],
        );
        assert!(res.is_ok());

        // accept bid
        let msg = MarketplaceExecuteMsg::AcceptBid {
            token_id: NAME.to_string(),
            bidder: BIDDER.to_string(),
        };
        let res = app.execute_contract(Addr::unchecked(USER2), Addr::unchecked(MKT), &msg, &[]);
        assert!(res.is_ok());
    }

    #[test]
    fn transfer_nft_with_reverse_map() {
        let mut app = instantiate_contracts(None, None, None);

        let user = "stars1hsk6jryyqjfhp5dhc55tc9jtckygx0eprx6sym";
        let user2 = "stars1wh3wjjgprxeww4cgqyaw8k75uslzh3sd3s2yfk";
        let res = mint_and_list(&mut app, NAME, user, None);
        assert!(res.is_ok());

        let msg = SgNameExecuteMsg::AssociateAddress {
            name: NAME.to_string(),
            address: Some(user.to_string()),
        };
        let res = app.execute_contract(
            Addr::unchecked(user),
            Addr::unchecked(COLLECTION),
            &msg,
            &[],
        );
        assert!(res.is_ok());

        let msg = SgNameQueryMsg::Name {
            address: user.to_string(),
        };
        let res: String = app.wrap().query_wasm_smart(COLLECTION, &msg).unwrap();
        assert_eq!(res, NAME.to_string());

        transfer(&mut app, user, user2);

        let msg = SgNameQueryMsg::Name {
            address: user.to_string(),
        };
        let err: StdResult<String> = app.wrap().query_wasm_smart(COLLECTION, &msg);
        assert!(err.is_err());

        let msg = SgNameQueryMsg::Name {
            address: user2.to_string(),
        };
        let err: StdResult<String> = app.wrap().query_wasm_smart(COLLECTION, &msg);
        assert!(err.is_err());
    }

    // test that burn nft currently does nothing. this is a placeholder for future functionality
    #[test]
    fn burn_nft() {
        let mut app = instantiate_contracts(None, None, None);

        let res = mint_and_list(&mut app, NAME, USER, None);
        assert!(res.is_ok());

        let msg = Sg721NameExecuteMsg::Burn {
            token_id: NAME.to_string(),
        };
        let res = app.execute_contract(
            Addr::unchecked(USER),
            Addr::unchecked(COLLECTION),
            &msg,
            &[],
        );
        assert!(res.is_err());

        let msg = MarketplaceQueryMsg::Ask {
            token_id: NAME.to_string(),
        };
        let res: Option<Ask> = app.wrap().query_wasm_smart(MKT, &msg).unwrap();
        assert!(res.is_some());
    }

    // test that burn nft currently does nothing. this is a placeholder for future functionality
    #[test]
    fn burn_with_existing_bids() {
        let mut app = instantiate_contracts(None, None, None);

        let res = mint_and_list(&mut app, NAME, USER, None);
        assert!(res.is_ok());

        bid(&mut app, NAME, BIDDER, BID_AMOUNT);

        let msg = Sg721NameExecuteMsg::Burn {
            token_id: NAME.to_string(),
        };
        let res = app.execute_contract(
            Addr::unchecked(USER),
            Addr::unchecked(COLLECTION),
            &msg,
            &[],
        );
        assert!(res.is_err());

        let msg = MarketplaceQueryMsg::Ask {
            token_id: NAME.to_string(),
        };
        let res: Option<Ask> = app.wrap().query_wasm_smart(MKT, &msg).unwrap();
        let ask = res.unwrap();
        assert_eq!(ask.seller.to_string(), USER.to_string());
    }

    // test that burn nft currently does nothing. this is a placeholder for future functionality
    #[test]
    fn burn_nft_with_reverse_map() {
        let mut app = instantiate_contracts(None, None, None);

        let user = "stars1hsk6jryyqjfhp5dhc55tc9jtckygx0eprx6sym";

        let res = mint_and_list(&mut app, NAME, user, None);
        assert!(res.is_ok());

        let msg = SgNameExecuteMsg::AssociateAddress {
            name: NAME.to_string(),
            address: Some(user.to_string()),
        };
        let res = app.execute_contract(
            Addr::unchecked(user),
            Addr::unchecked(COLLECTION),
            &msg,
            &[],
        );
        assert!(res.is_ok());

        let msg = SgNameQueryMsg::Name {
            address: user.to_string(),
        };
        let res: String = app.wrap().query_wasm_smart(COLLECTION, &msg).unwrap();
        assert_eq!(res, NAME.to_string());

        let msg = Sg721NameExecuteMsg::Burn {
            token_id: NAME.to_string(),
        };
        let res = app.execute_contract(
            Addr::unchecked(user),
            Addr::unchecked(COLLECTION),
            &msg,
            &[],
        );
        assert!(res.is_err());

        let msg = MarketplaceQueryMsg::Ask {
            token_id: NAME.to_string(),
        };
        let res: Option<Ask> = app.wrap().query_wasm_smart(MKT, &msg).unwrap();
        assert!(res.is_some());

        let msg = SgNameQueryMsg::Name {
            address: user.to_string(),
        };
        let res: StdResult<String> = app.wrap().query_wasm_smart(COLLECTION, &msg);
        assert!(res.is_ok());
    }

    #[test]
    fn sudo_update() {
        let mut app = instantiate_contracts(None, None, None);
        let params: SudoParams = app
            .wrap()
            .query_wasm_smart(COLLECTION, &(Sg721NameQueryMsg::Params {}))
            .unwrap();
        let max_record_count = params.max_record_count;

        let msg = sg721_name::msg::SudoMsg::UpdateParams {
            max_record_count: max_record_count + 1,
        };
        let res = app.wasm_sudo(Addr::unchecked(COLLECTION), &msg);
        assert!(res.is_ok());
        let params: SudoParams = app
            .wrap()
            .query_wasm_smart(COLLECTION, &(Sg721NameQueryMsg::Params {}))
            .unwrap();
        assert_eq!(params.max_record_count, max_record_count + 1);
    }
}

mod whitelist {
    use crate::msg::QueryMsg;
    use whitelist_updatable_flatrate::{msg::QueryMsg as WhitelistQueryMsg, state::Config};

    use super::*;

    const WHITELIST2: &str = "contract4";

    #[test]
    fn init() {
        let _ = instantiate_contracts(None, Some(ADMIN.to_string()), None);
    }

    #[test]
    fn add_remove_flatrate_whitelist() {
        let mut app = instantiate_contracts(None, Some(ADMIN.to_string()), None);

        let whitelists: Vec<Addr> = app
            .wrap()
            .query_wasm_smart(MINTER, &(QueryMsg::Whitelists {}))
            .unwrap();
        let wl_count = whitelists.len();
        let msg = ExecuteMsg::AddWhitelist {
            address: "whitelist".to_string(),
            whitelist_type: "FlatrateDiscount".to_string(),
        };

        let res = app.execute_contract(Addr::unchecked(ADMIN), Addr::unchecked(MINTER), &msg, &[]);
        assert!(res.is_ok());

        let msg = QueryMsg::Whitelists {};
        let whitelists: Vec<Addr> = app.wrap().query_wasm_smart(MINTER, &msg).unwrap();
        assert_eq!(whitelists.len(), wl_count + 1);

        let msg = ExecuteMsg::RemoveWhitelist {
            address: "whitelist".to_string(),
        };
        let res = app.execute_contract(Addr::unchecked(ADMIN), Addr::unchecked(MINTER), &msg, &[]);
        assert!(res.is_ok());

        let msg = QueryMsg::Whitelists {};
        let whitelists: Vec<Addr> = app.wrap().query_wasm_smart(MINTER, &msg).unwrap();
        assert_eq!(whitelists.len(), wl_count);
    }

    #[test]
    fn add_remove_percent_whitelist() {
        let mut app = instantiate_contracts(None, Some(ADMIN.to_string()), None);

        let whitelists: Vec<Addr> = app
            .wrap()
            .query_wasm_smart(MINTER, &(QueryMsg::Whitelists {}))
            .unwrap();
        let wl_count = whitelists.len();
        let msg = ExecuteMsg::AddWhitelist {
            address: "whitelist".to_string(),
            whitelist_type: "PercentDiscount".to_string(),
        };

        let res = app.execute_contract(Addr::unchecked(ADMIN), Addr::unchecked(MINTER), &msg, &[]);
        assert!(res.is_ok());

        let msg = QueryMsg::Whitelists {};
        let whitelists: Vec<Addr> = app.wrap().query_wasm_smart(MINTER, &msg).unwrap();
        assert_eq!(whitelists.len(), wl_count + 1);

        let msg = ExecuteMsg::RemoveWhitelist {
            address: "whitelist".to_string(),
        };
        let res = app.execute_contract(Addr::unchecked(ADMIN), Addr::unchecked(MINTER), &msg, &[]);
        assert!(res.is_ok());

        let msg = QueryMsg::Whitelists {};
        let whitelists: Vec<Addr> = app.wrap().query_wasm_smart(MINTER, &msg).unwrap();
        assert_eq!(whitelists.len(), wl_count);
    }

    #[test]
    fn multiple_wl() {
        let mut app = instantiate_contracts(None, Some(ADMIN.to_string()), None);
        let wl_id = app.store_code(contract_whitelist());

        // instantiate wl2
        let msg = whitelist_updatable_flatrate::msg::InstantiateMsg {
            per_address_limit: PER_ADDRESS_LIMIT,
            addresses: vec![
                "addr0001".to_string(),
                "addr0002".to_string(),
                USER.to_string(),
                USER2.to_string(),
                ADMIN2.to_string(),
            ],
            mint_discount_amount: None,
            admin_list: None,
        };
        let wl2 = app
            .instantiate_contract(wl_id, Addr::unchecked(ADMIN2), &msg, &[], "Whitelist", None)
            .unwrap();

        // add wl2 to minter
        let msg = ExecuteMsg::AddWhitelist {
            address: wl2.to_string(),
            whitelist_type: "FlatrateDiscount".to_string(),
        };
        let res = app.execute_contract(
            Addr::unchecked(ADMIN.to_string()),
            Addr::unchecked(MINTER.to_string()),
            &msg,
            &[],
        );
        assert!(res.is_ok());

        // mint from user on first whitelist
        let res = mint_and_list(&mut app, NAME, USER, None);
        assert!(res.is_ok());

        // check user mint count on both lists
        // whitelist 1
        let res: u64 = app
            .wrap()
            .query_wasm_smart(
                WHITELIST,
                &(WhitelistQueryMsg::MintCount {
                    address: USER.to_string(),
                }),
            )
            .unwrap();
        assert_eq!(res, 1);

        // whitelist 2
        let res: u64 = app
            .wrap()
            .query_wasm_smart(
                WHITELIST2,
                &(WhitelistQueryMsg::MintCount {
                    address: USER.to_string(),
                }),
            )
            .unwrap();
        assert_eq!(res, 0);

        // mint from user on second whitelist
        let res = mint_and_list(&mut app, "none", USER2, None);
        assert!(res.is_ok());

        update_block_time(&mut app, 1000);

        // mint over per address limit
        let res = mint_and_list(&mut app, "some", USER, None);
        assert!(res.is_ok());
        let res = mint_and_list(&mut app, "zome", USER, None);
        assert!(res.is_err());
    }

    #[test]
    fn discount() {
        let mut app = instantiate_contracts(None, Some(ADMIN.to_string()), None);
        let wl_id = app.store_code(contract_whitelist());

        // instantiate wl2
        let msg = whitelist_updatable_flatrate::msg::InstantiateMsg {
            per_address_limit: PER_ADDRESS_LIMIT,
            addresses: vec![
                "addr0001".to_string(),
                "addr0002".to_string(),
                USER.to_string(),
                USER2.to_string(),
                ADMIN2.to_string(),
            ],
            mint_discount_amount: Some(BASE_PRICE as u64),
            admin_list: None,
        };

        let wl2 = app
            .instantiate_contract(wl_id, Addr::unchecked(ADMIN2), &msg, &[], "Whitelist", None)
            .unwrap();

        // add wl2 to minter
        let msg = ExecuteMsg::AddWhitelist {
            address: wl2.to_string(),
            whitelist_type: "FlatrateDiscount".to_string(),
        };
        let res = app.execute_contract(
            Addr::unchecked(ADMIN.to_string()),
            Addr::unchecked(MINTER.to_string()),
            &msg,
            &[],
        );
        assert!(res.is_ok());

        // mint and list with discount
        // query discount, pass to mint_and_list
        let discount: u64 = app
            .wrap()
            .query_wasm_smart(wl2, &(WhitelistQueryMsg::MintDiscountAmount {}))
            .unwrap();
        let res = mint_and_list(
            &mut app,
            NAME,
            USER2,
            Some(Decimal::from_ratio(
                discount,
                Uint128::from(1_000_000_000u128),
            )),
        );
        println!("result: {:?}", res);
        assert!(res.is_ok());
    }

    #[test]
    fn discount2() {
        let mut app = instantiate_contracts(None, Some(ADMIN.to_string()), None);
        let wl_id = app.store_code(contract_whitelist_percent());

        // instantiate wl2
        let msg = whitelist_updatable::msg::InstantiateMsg {
            per_address_limit: PER_ADDRESS_LIMIT,
            addresses: vec![
                "addr0001".to_string(),
                "addr0002".to_string(),
                USER.to_string(),
                USER2.to_string(),
                ADMIN2.to_string(),
            ],
            mint_discount_bps: Some(1000u64),
        };

        let wl2 = app
            .instantiate_contract(wl_id, Addr::unchecked(ADMIN2), &msg, &[], "Whitelist", None)
            .unwrap();

        // add wl2 to minter
        let msg = ExecuteMsg::AddWhitelist {
            address: wl2.to_string(),
            whitelist_type: "PercentDiscount".to_string(),
        };
        let res = app.execute_contract(
            Addr::unchecked(ADMIN.to_string()),
            Addr::unchecked(MINTER.to_string()),
            &msg,
            &[],
        );
        assert!(res.is_ok());

        // mint and list with discount
        // query discount, pass to mint_and_list
        let discount: Decimal = app
            .wrap()
            .query_wasm_smart(wl2, &(PercentWhitelistQueryMsg::MintDiscountPercent {}))
            .unwrap();
        let res = mint_and_list(&mut app, NAME, USER2, Some(discount));
        println!("result: {:?}", res);
        assert!(res.is_ok());
    }

    #[test]
    fn mint_from_incorrect_whitelist_type() {
        let mut app = instantiate_contracts(None, Some(ADMIN.to_string()), None);

        let msg = ExecuteMsg::AddWhitelist {
            address: WHITELIST.to_string(),
            whitelist_type: "FakeDiscount".to_string(),
        };
        let res = app.execute_contract(Addr::unchecked(ADMIN), Addr::unchecked(MINTER), &msg, &[]);
        assert!(res.is_err());
    }

    #[test]
    fn mint_from_percent_whitelist() {
        let mut app = instantiate_contracts(None, Some(ADMIN.to_string()), None);

        let msg = ExecuteMsg::AddWhitelist {
            address: WHITELIST.to_string(),
            whitelist_type: "PercentDiscount".to_string(),
        };
        let res = app.execute_contract(Addr::unchecked(ADMIN), Addr::unchecked(MINTER), &msg, &[]);
        assert!(res.is_ok());

        let msg = QueryMsg::Whitelists {};
        let whitelists: Vec<Addr> = app.wrap().query_wasm_smart(MINTER, &msg).unwrap();
        assert_eq!(whitelists.len(), 2);

        let msg = WhitelistQueryMsg::AddressCount {};
        let wl_addr_count: u64 = app.wrap().query_wasm_smart(WHITELIST, &msg).unwrap();
        assert_eq!(wl_addr_count, 5);

        let msg = WhitelistExecuteMsg::AddAddresses {
            addresses: vec![USER3.to_string()],
        };
        let res = app.execute_contract(
            Addr::unchecked(ADMIN2),
            Addr::unchecked(WHITELIST),
            &msg,
            &[],
        );
        assert!(res.is_ok());

        let msg = WhitelistQueryMsg::Config {};
        let res: Config = app.wrap().query_wasm_smart(WHITELIST, &msg).unwrap();
        assert_eq!(res.admins, [ADMIN2.to_string()]);

        let msg = WhitelistQueryMsg::AddressCount {};
        let res: u64 = app.wrap().query_wasm_smart(WHITELIST, &msg).unwrap();
        assert_eq!(res, wl_addr_count + 1);

        let msg = WhitelistQueryMsg::IncludesAddress {
            address: USER3.to_string(),
        };
        let res: bool = app.wrap().query_wasm_smart(WHITELIST, &msg).unwrap();
        assert!(res);

        let res = mint_and_list(&mut app, NAME, USER3, None);
        assert!(res.is_ok());
    }

    #[test]
    fn mint_from_flatrate_whitelist() {
        let mut app = instantiate_contracts(None, Some(ADMIN.to_string()), None);

        let msg = ExecuteMsg::AddWhitelist {
            address: WHITELIST.to_string(),
            whitelist_type: "FlatrateDiscount".to_string(),
        };
        let res = app.execute_contract(Addr::unchecked(ADMIN), Addr::unchecked(MINTER), &msg, &[]);
        assert!(res.is_ok());

        let msg = QueryMsg::Whitelists {};
        let whitelists: Vec<Addr> = app.wrap().query_wasm_smart(MINTER, &msg).unwrap();
        assert_eq!(whitelists.len(), 2);

        let msg = WhitelistQueryMsg::AddressCount {};
        let wl_addr_count: u64 = app.wrap().query_wasm_smart(WHITELIST, &msg).unwrap();
        assert_eq!(wl_addr_count, 5);

        let msg = WhitelistExecuteMsg::AddAddresses {
            addresses: vec![USER3.to_string()],
        };
        let res = app.execute_contract(
            Addr::unchecked(ADMIN2),
            Addr::unchecked(WHITELIST),
            &msg,
            &[],
        );
        assert!(res.is_ok());

        let msg = WhitelistQueryMsg::Config {};
        let res: Config = app.wrap().query_wasm_smart(WHITELIST, &msg).unwrap();
        assert_eq!(res.admins, [ADMIN2.to_string()]);

        let msg = WhitelistQueryMsg::AddressCount {};
        let res: u64 = app.wrap().query_wasm_smart(WHITELIST, &msg).unwrap();
        assert_eq!(res, wl_addr_count + 1);

        let msg = WhitelistQueryMsg::IncludesAddress {
            address: USER3.to_string(),
        };
        let res: bool = app.wrap().query_wasm_smart(WHITELIST, &msg).unwrap();
        assert!(res);

        let res = mint_and_list(&mut app, NAME, USER3, None);
        assert!(res.is_ok());
    }
}

mod public_start_time {
    use sg_name_minter::Config;

    use crate::msg::QueryMsg;

    use super::*;

    #[test]
    fn mint_before_start() {
        let mut app = instantiate_contracts(
            None,
            Some(ADMIN.to_string()),
            Some(PUBLIC_MINT_START_TIME_IN_SECONDS.minus_seconds(1)),
        );

        // try pub mint with whitelists before start time
        // 1a. on whitelist and mints with whitelist price
        mint_and_list(&mut app, "some-name", USER, None).unwrap();
        // 1b. USER2 not on whitelist and errors
        let res = mint_and_list(&mut app, NAME, USER2, None);
        assert!(res.is_err());

        // remove whitelist(s)
        let msg = ExecuteMsg::RemoveWhitelist {
            address: WHITELIST.to_string(),
        };
        let res = app.execute_contract(Addr::unchecked(ADMIN), Addr::unchecked(MINTER), &msg, &[]);
        assert!(res.is_ok());

        // try pub mint before start time
        let res = mint_and_list(&mut app, NAME, USER, None);
        assert!(res.is_err());
    }

    #[test]
    fn update_start_time() {
        let mut app = instantiate_contracts(
            None,
            Some(ADMIN.to_string()),
            Some(PUBLIC_MINT_START_TIME_IN_SECONDS.minus_seconds(1)),
        );

        // remove whitelist(s)
        let msg = ExecuteMsg::RemoveWhitelist {
            address: WHITELIST.to_string(),
        };
        let res = app.execute_contract(Addr::unchecked(ADMIN), Addr::unchecked(MINTER), &msg, &[]);
        assert!(res.is_ok());

        // default start time is PUBLIC_MINT_START_TIME_IN_SECONDS
        let msg = QueryMsg::Config {};
        let res: Config = app.wrap().query_wasm_smart(MINTER, &msg).unwrap();
        assert_eq!(
            res.public_mint_start_time,
            PUBLIC_MINT_START_TIME_IN_SECONDS
        );

        // update start time to PUBLIC_MINT_START_TIME_IN_SECONDS - 1
        let msg = ExecuteMsg::UpdateConfig {
            config: Config {
                public_mint_start_time: PUBLIC_MINT_START_TIME_IN_SECONDS.minus_seconds(1),
            },
        };
        let res = app.execute_contract(Addr::unchecked(ADMIN), Addr::unchecked(MINTER), &msg, &[]);
        assert!(res.is_ok());

        // check start time
        let msg = QueryMsg::Config {};
        let res: Config = app.wrap().query_wasm_smart(MINTER, &msg).unwrap();
        assert_eq!(
            res.public_mint_start_time,
            PUBLIC_MINT_START_TIME_IN_SECONDS.minus_seconds(1)
        );

        // mint succeeds w new mint start time
        let res = mint_and_list(&mut app, NAME, USER, None);
        assert!(res.is_ok());
    }

    #[test]
    fn whitelist_with_public_mint() {
        let mut app = instantiate_contracts(
            None,
            Some(ADMIN.to_string()),
            Some(PUBLIC_MINT_START_TIME_IN_SECONDS.plus_seconds(1)),
        );

        // mint succeeds w new mint start time
        let res = mint_and_list(&mut app, NAME, USER, None);
        assert!(res.is_ok());
    }
}

mod associate_address {
    use super::*;

    use collection::transfer;

    use sg721::{CollectionInfo, InstantiateMsg as Sg721InstantiateMsg};

    #[test]
    fn transfer_to_eoa() {
        let mut app = instantiate_contracts(
            None,
            Some(ADMIN.to_string()),
            Some(PUBLIC_MINT_START_TIME_IN_SECONDS.minus_seconds(1)),
        );

        let nft_id = app.store_code(contract_nft());

        let init_msg = Sg721InstantiateMsg {
            name: "NFT".to_string(),
            symbol: "NFT".to_string(),
            minter: Addr::unchecked(MINTER).to_string(),
            collection_info: CollectionInfo {
                creator: ADMIN.to_string(),
                description: "Stargaze Names".to_string(),
                image: "ipfs://example.com".to_string(),
                external_link: None,
                explicit_content: None,
                start_trading_time: None,
                royalty_info: None,
            },
        };
        let nft_addr = app
            .instantiate_contract(
                nft_id,
                Addr::unchecked(MINTER),
                &init_msg,
                &[],
                "NFT",
                Some(ADMIN.to_string()),
            )
            .unwrap();

        // mint and transfer to collection
        mint_and_list(&mut app, NAME, USER, None).unwrap();
        transfer(&mut app, USER, nft_addr.as_ref());
        let owner = owner_of(&app, NAME.to_string());
        assert_eq!(owner, nft_addr.to_string());

        // associate contract
        let msg = SgNameExecuteMsg::AssociateAddress {
            name: NAME.to_string(),
            address: Some(nft_addr.to_string()),
        };
        let res = app.execute_contract(
            Addr::unchecked(nft_addr.clone()),
            Addr::unchecked(COLLECTION),
            &msg,
            &[],
        );
        assert!(res.is_ok());

        // transfer from collection back to personal wallet
        transfer(&mut app, nft_addr.as_ref(), USER);
        let owner = owner_of(&app, NAME.to_string());
        assert_eq!(owner, USER.to_string());
    }

    #[test]
    fn associate_with_a_contract_with_no_admin() {
        let mut app = instantiate_contracts(
            None,
            Some(ADMIN.to_string()),
            Some(PUBLIC_MINT_START_TIME_IN_SECONDS.minus_seconds(1)),
        );

        let nft_id = app.store_code(contract_nft());

        // For the purposes of this test, a collection contract with no admin needs to be instantiated (contract_with_no_admin)
        // This contract needs to have a creator that is itself a contract and this creator contract should have an admin (USER).
        // The admin (USER) of the creator contract will mint a name and associate the name with the collection contract that doesn't have an admin successfully.

        // Instantiating the creator contract with an admin (USER)
        let creator_init_msg = Sg721InstantiateMsg {
            name: "NFT".to_string(),
            symbol: "NFT".to_string(),
            minter: Addr::unchecked(MINTER).to_string(),
            collection_info: CollectionInfo {
                creator: USER.to_string(),
                description: "Stargaze Names".to_string(),
                image: "ipfs://example.com".to_string(),
                external_link: None,
                explicit_content: None,
                start_trading_time: None,
                royalty_info: None,
            },
        };
        let creator_addr = app
            .instantiate_contract(
                nft_id,
                Addr::unchecked(MINTER),
                &creator_init_msg,
                &[],
                "NFT",
                Some(USER.to_string()),
            )
            .unwrap();

        // The creator contract instantiates the collection contract with no admin
        let init_msg = Sg721InstantiateMsg {
            name: "NFT".to_string(),
            symbol: "NFT".to_string(),
            minter: creator_addr.to_string(),
            collection_info: CollectionInfo {
                creator: USER.to_string(),
                description: "Stargaze Names".to_string(),
                image: "ipfs://example.com".to_string(),
                external_link: None,
                explicit_content: None,
                start_trading_time: None,
                royalty_info: None,
            },
        };

        let collection_with_no_admin_addr = app
            .instantiate_contract(nft_id, creator_addr, &init_msg, &[], "NFT", None)
            .unwrap();

        // USER mints a name
        mint_and_list(&mut app, NAME, USER, None).unwrap();

        // USER associates the name with the collection contract that doesn't have an admin
        let msg = SgNameExecuteMsg::AssociateAddress {
            name: NAME.to_string(),
            address: Some(collection_with_no_admin_addr.to_string()),
        };
        let res = app.execute_contract(
            Addr::unchecked(USER),
            Addr::unchecked(COLLECTION),
            &msg,
            &[],
        );
        assert!(res.is_ok());
    }
    #[test]
    fn associate_with_a_contract_with_no_admin_fail() {
        let mut app = instantiate_contracts(
            None,
            Some(ADMIN.to_string()),
            Some(PUBLIC_MINT_START_TIME_IN_SECONDS.minus_seconds(1)),
        );

        let nft_id = app.store_code(contract_nft());

        // For the purposes of this test, a collection contract with no admin needs to be instantiated (contract_with_no_admin)
        // This contract needs to have a creator that is itself a contract and this creator contract should have an admin (USER).
        // An address other than the admin (USER) of the creator contract will mint a name, try to associate the name with the collection contract that doesn't have an admin and fail.

        // Instantiating the creator contract with an admin (USER)
        let creator_init_msg = Sg721InstantiateMsg {
            name: "NFT".to_string(),
            symbol: "NFT".to_string(),
            minter: Addr::unchecked(MINTER).to_string(),
            collection_info: CollectionInfo {
                creator: USER.to_string(),
                description: "Stargaze Names".to_string(),
                image: "ipfs://example.com".to_string(),
                external_link: None,
                explicit_content: None,
                start_trading_time: None,
                royalty_info: None,
            },
        };
        let creator_addr = app
            .instantiate_contract(
                nft_id,
                Addr::unchecked(MINTER),
                &creator_init_msg,
                &[],
                "NFT",
                Some(USER.to_string()),
            )
            .unwrap();

        // The creator contract instantiates the collection contract with no admin
        let init_msg = Sg721InstantiateMsg {
            name: "NFT".to_string(),
            symbol: "NFT".to_string(),
            minter: creator_addr.to_string(),
            collection_info: CollectionInfo {
                creator: USER.to_string(),
                description: "Stargaze Names".to_string(),
                image: "ipfs://example.com".to_string(),
                external_link: None,
                explicit_content: None,
                start_trading_time: None,
                royalty_info: None,
            },
        };

        let collection_with_no_admin_addr = app
            .instantiate_contract(nft_id, creator_addr, &init_msg, &[], "NFT", None)
            .unwrap();

        // USER4 mints a name
        mint_and_list(&mut app, NAME, USER4, None).unwrap();

        // USER4 tries to associate the name with the collection contract that doesn't have an admin
        let msg = SgNameExecuteMsg::AssociateAddress {
            name: NAME.to_string(),
            address: Some(collection_with_no_admin_addr.to_string()),
        };
        let res = app
            .execute_contract(
                Addr::unchecked(USER4),
                Addr::unchecked(COLLECTION),
                &msg,
                &[],
            )
            .map_err(|e| e.downcast::<sg721_name::ContractError>().unwrap())
            .unwrap_err();
        assert!(matches!(
            res,
            sg721_name::ContractError::UnauthorizedCreatorOrAdmin {}
        ))
    }

    #[test]
    fn associate_with_a_contract_with_an_admin_fail() {
        let mut app = instantiate_contracts(
            None,
            Some(ADMIN.to_string()),
            Some(PUBLIC_MINT_START_TIME_IN_SECONDS.minus_seconds(1)),
        );

        let nft_id = app.store_code(contract_nft());

        // Instantiating the creator contract with an admin (USER)
        let creator_init_msg = Sg721InstantiateMsg {
            name: "NFT".to_string(),
            symbol: "NFT".to_string(),
            minter: Addr::unchecked(MINTER).to_string(),
            collection_info: CollectionInfo {
                creator: USER.to_string(),
                description: "Stargaze Names".to_string(),
                image: "ipfs://example.com".to_string(),
                external_link: None,
                explicit_content: None,
                start_trading_time: None,
                royalty_info: None,
            },
        };
        let contract_with_an_admin = app
            .instantiate_contract(
                nft_id,
                Addr::unchecked(MINTER),
                &creator_init_msg,
                &[],
                "NFT",
                Some(USER.to_string()),
            )
            .unwrap();

        // USER4 mints a name
        mint_and_list(&mut app, NAME, USER4, None).unwrap();

        // USER4 tries to associate the name with the collection contract that has an admin (USER)
        let msg = SgNameExecuteMsg::AssociateAddress {
            name: NAME.to_string(),
            address: Some(contract_with_an_admin.to_string()),
        };
        let res = app
            .execute_contract(
                Addr::unchecked(USER4),
                Addr::unchecked(COLLECTION),
                &msg,
                &[],
            )
            .map_err(|e| e.downcast::<sg721_name::ContractError>().unwrap())
            .unwrap_err();
        assert!(matches!(
            res,
            sg721_name::ContractError::UnauthorizedCreatorOrAdmin {}
        ))
    }
}
