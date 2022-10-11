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

const MKT: &str = "contract0";
const MINTER: &str = "contract1";
const COLLECTION: &str = "contract2";

pub fn custom_mock_app() -> StargazeApp {
    StargazeApp::default()
}

// 1. Instantiate Name Marketplace
// 2. Instantiate Name Minter (which instantiates Name Collection)
// 3. Update Name Marketplace with Name Minter address
// 4. Update Name Marketplace with Name Collection address
fn instantiate_contracts() -> StargazeApp {
    let mut app = custom_mock_app();
    let mkt_id = app.store_code(contract_marketplace());
    let minter_id = app.store_code(contract_minter());
    let sg721_id = app.store_code(contract_collection());

    // 1. Instantiate Name Marketplace
    let msg = name_marketplace::msg::InstantiateMsg {
        trading_fee_bps: TRADING_FEE_BPS,
        min_price: Uint128::from(5u128),
        blocks_per_year: BLOCKS_PER_YEAR,
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

    // 3. Update Name Marketplace with Name Minter address
    let msg = name_marketplace::msg::SudoMsg::UpdateNameMinter {
        minter: minter.to_string(),
    };
    let res = app.wasm_sudo(marketplace.clone(), &msg);
    assert!(res.is_ok());

    // 4. Update Name Marketplace with Name Collection address
    let msg = name_marketplace::msg::SudoMsg::UpdateNameCollection {
        collection: COLLECTION.to_string(),
    };
    let res = app.wasm_sudo(marketplace, &msg);
    assert!(res.is_ok());

    app
}

fn owner_of(app: &StargazeApp, token_id: String) -> String {
    let res: OwnerOfResponse = app
        .wrap()
        .query_wasm_smart(
            COLLECTION,
            &sg721_base::msg::QueryMsg::OwnerOf {
                token_id,
                include_expired: None,
            },
        )
        .unwrap();

    res.owner
}

fn update_block_height(app: &mut StargazeApp, height: u64) {
    let mut block = app.block_info();
    block.height = height;
    app.set_block(block);
}

fn mint_and_list(app: &mut StargazeApp, name: &str, user: &str) {
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
        name: name.to_string(),
    };
    let res = app.execute_contract(
        Addr::unchecked(user),
        Addr::unchecked(MINTER),
        &msg,
        &name_fee,
    );
    assert!(res.is_ok());

    // check if name is listed in marketplace
    let res: AskResponse = app
        .wrap()
        .query_wasm_smart(
            MKT,
            &MarketplaceQueryMsg::Ask {
                token_id: name.to_string(),
            },
        )
        .unwrap();
    assert_eq!(res.ask.unwrap().token_id, name);

    // check if name is in the renewal queue
    let res: RenewalQueueResponse = app
        .wrap()
        .query_wasm_smart(
            MKT,
            &MarketplaceQueryMsg::RenewalQueue {
                height: app.block_info().height + BLOCKS_PER_YEAR,
            },
        )
        .unwrap();
    assert_eq!(res.queue.len(), 1);
    assert_eq!(res.queue[0], name);

    // check if token minted
    let _res: NumTokensResponse = app
        .wrap()
        .query_wasm_smart(
            Addr::unchecked(COLLECTION),
            &sg721_base::msg::QueryMsg::NumTokens {},
        )
        .unwrap();

    assert_eq!(owner_of(app, name.to_string()), user.to_string());
}

fn bid(app: &mut StargazeApp, bidder: &str, amount: u128) {
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
        operator: MKT.to_string(),
        expires: None,
    };
    let res = app.execute_contract(bidder.clone(), Addr::unchecked(COLLECTION), &msg, &[]);
    assert!(res.is_ok());

    let msg = MarketplaceExecuteMsg::SetBid {
        token_id: NAME.to_string(),
    };
    let res = app.execute_contract(bidder.clone(), Addr::unchecked(MKT), &msg, &amount);
    assert!(res.is_ok());

    // query if bid exists
    let res: BidResponse = app
        .wrap()
        .query_wasm_smart(
            MKT,
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
}

mod execute {
    use cw721::OperatorsResponse;

    use super::*;

    #[test]
    fn check_approvals() {
        let mut app = instantiate_contracts();

        mint_and_list(&mut app, NAME, USER);

        // check operators
        let res: OperatorsResponse = app
            .wrap()
            .query_wasm_smart(
                COLLECTION,
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
        let mut app = instantiate_contracts();

        mint_and_list(&mut app, NAME, USER);
    }

    #[test]
    fn test_bid() {
        let mut app = instantiate_contracts();

        mint_and_list(&mut app, NAME, USER);
        bid(&mut app, BIDDER, BID_AMOUNT);
    }

    #[test]
    fn test_accept_bid() {
        let mut app = instantiate_contracts();

        mint_and_list(&mut app, NAME, USER);
        bid(&mut app, BIDDER, BID_AMOUNT);

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
        let res: BidResponse = app
            .wrap()
            .query_wasm_smart(
                MKT,
                &MarketplaceQueryMsg::Bid {
                    token_id: NAME.to_string(),
                    bidder: BIDDER.to_string(),
                },
            )
            .unwrap();
        assert!(res.bid.is_none());

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
        let res: AskResponse = app
            .wrap()
            .query_wasm_smart(
                MKT,
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
        let mut app = instantiate_contracts();

        mint_and_list(&mut app, NAME, USER);
        bid(&mut app, BIDDER, BID_AMOUNT);

        let msg = MarketplaceExecuteMsg::AcceptBid {
            token_id: NAME.to_string(),
            bidder: BIDDER.to_string(),
        };
        let res = app.execute_contract(Addr::unchecked(USER), Addr::unchecked(MKT), &msg, &[]);
        assert!(res.is_ok());

        bid(&mut app, BIDDER2, BID_AMOUNT);

        let msg = MarketplaceExecuteMsg::AcceptBid {
            token_id: NAME.to_string(),
            bidder: BIDDER2.to_string(),
        };
        let res = app.execute_contract(Addr::unchecked(BIDDER), Addr::unchecked(MKT), &msg, &[]);
        assert!(res.is_ok());
    }
}

mod query {
    use name_marketplace::msg::{AskCountResponse, AsksResponse, BidsResponse};

    use super::*;

    #[test]
    fn query_ask() {
        let mut app = instantiate_contracts();

        mint_and_list(&mut app, NAME, USER);

        let msg = MarketplaceQueryMsg::Ask {
            token_id: NAME.to_string(),
        };
        let res: AskResponse = app.wrap().query_wasm_smart(MKT, &msg).unwrap();
        assert_eq!(res.ask.unwrap().token_id, NAME.to_string());
    }

    #[test]
    fn query_asks() {
        let mut app = instantiate_contracts();

        mint_and_list(&mut app, NAME, USER);

        let height = app.block_info().height;
        update_block_height(&mut app, height + 1);
        mint_and_list(&mut app, "hack", USER);

        let msg = MarketplaceQueryMsg::Asks {
            start_after: None,
            limit: None,
        };
        let res: AsksResponse = app.wrap().query_wasm_smart(MKT, &msg).unwrap();
        assert_eq!(res.asks[0].id, 1);
    }

    #[test]
    fn query_reverse_asks() {
        let mut app = instantiate_contracts();

        mint_and_list(&mut app, NAME, USER);

        let height = app.block_info().height;
        update_block_height(&mut app, height + 1);
        mint_and_list(&mut app, "hack", USER);

        let msg = MarketplaceQueryMsg::ReverseAsks {
            start_before: Some(5),
            limit: None,
        };
        let res: AsksResponse = app.wrap().query_wasm_smart(MKT, &msg).unwrap();
        assert_eq!(res.asks[0].id, 2);
    }

    #[test]
    fn query_asks_by_seller() {
        let mut app = instantiate_contracts();

        mint_and_list(&mut app, NAME, USER);

        let height = app.block_info().height;
        update_block_height(&mut app, height + 1);
        mint_and_list(&mut app, "hack", "user2");

        let msg = MarketplaceQueryMsg::AsksBySeller {
            seller: USER.to_string(),
            start_after: None,
            limit: None,
        };
        let res: AsksResponse = app.wrap().query_wasm_smart(MKT, &msg).unwrap();
        assert_eq!(res.asks.len(), 1);
    }

    #[test]
    fn query_ask_count() {
        let mut app = instantiate_contracts();

        mint_and_list(&mut app, NAME, USER);

        let height = app.block_info().height;
        update_block_height(&mut app, height + 1);
        mint_and_list(&mut app, "hack", USER);

        let msg = MarketplaceQueryMsg::AskCount {};
        let res: AskCountResponse = app.wrap().query_wasm_smart(MKT, &msg).unwrap();
        assert_eq!(res.count, 2);
    }

    #[test]
    fn query_top_bids() {
        let mut app = instantiate_contracts();

        mint_and_list(&mut app, NAME, USER);
        bid(&mut app, BIDDER, BID_AMOUNT);
        bid(&mut app, BIDDER2, BID_AMOUNT * 5);

        let msg = MarketplaceQueryMsg::ReverseBidsSortedByPrice {
            start_before: None,
            limit: None,
        };
        let res: BidsResponse = app.wrap().query_wasm_smart(MKT, &msg).unwrap();
        assert_eq!(res.bids.len(), 2);
        assert_eq!(res.bids[0].amount.u128(), BID_AMOUNT * 5);
    }

    #[test]
    fn query_renewal_queue() {
        let mut app = instantiate_contracts();

        mint_and_list(&mut app, NAME, USER);

        let height = app.block_info().height;
        update_block_height(&mut app, height + 1);
        mint_and_list(&mut app, "hack", USER);

        let res: RenewalQueueResponse = app
            .wrap()
            .query_wasm_smart(
                MKT,
                &MarketplaceQueryMsg::RenewalQueue {
                    height: app.block_info().height + BLOCKS_PER_YEAR,
                },
            )
            .unwrap();
        assert_eq!(res.queue.len(), 1);
        assert_eq!(res.queue[0], "hack".to_string());
    }
}

mod profile {
    // requires a lot of boilerplate from vending-factory, vending-minter, sg721 to set up the test scenario
    use super::*;

    use cosmwasm_std::{coin, Timestamp};
    use cw721::NftInfoResponse;
    use cw721_base::{state::TokenInfo, Extension, MintMsg};
    use sg2::{msg::Sg2ExecuteMsg, tests::mock_collection_params};
    use sg721::ExecuteMsg as Sg721ExecuteMsg;
    use sg721_base::msg::QueryMsg::NftInfo;
    use sg_name::{Metadata, NFT};
    use sg_std::{StargazeMsgWrapper, GENESIS_MINT_START_TIME, NATIVE_DENOM};
    use vending_factory::{
        msg::{VendingMinterCreateMsg, VendingMinterInitMsgExtension},
        state::{ParamsExtension, VendingMinterParams},
    };
    use vending_minter::msg::{ConfigResponse as MinterConfigResponse, QueryMsg};

    pub fn contract_factory() -> Box<dyn Contract<StargazeMsgWrapper>> {
        let contract = ContractWrapper::new(
            vending_factory::contract::execute,
            vending_factory::contract::instantiate,
            vending_factory::contract::query,
        );
        Box::new(contract)
    }

    pub fn contract_minter() -> Box<dyn Contract<StargazeMsgWrapper>> {
        let contract = ContractWrapper::new(
            vending_minter::contract::execute,
            vending_minter::contract::instantiate,
            vending_minter::contract::query,
        )
        .with_reply(vending_minter::contract::reply);
        Box::new(contract)
    }

    pub fn contract_sg721() -> Box<dyn Contract<StargazeMsgWrapper>> {
        let contract = ContractWrapper::new(
            sg721_base::entry::execute,
            sg721_base::entry::instantiate,
            sg721_base::entry::query,
        );
        Box::new(contract)
    }

    const CREATION_FEE: u128 = 5_000_000_000;
    const INITIAL_BALANCE: u128 = 2_000_000_000;
    const MINT_PRICE: u128 = 100_000_000;
    const MAX_TOKEN_LIMIT: u32 = 10000;

    pub const MIN_MINT_PRICE: u128 = 50_000_000;
    pub const AIRDROP_MINT_PRICE: u128 = 0;
    pub const MINT_FEE_BPS: u64 = 1_000; // 10%
    pub const AIRDROP_MINT_FEE_BPS: u64 = 10_000; // 100%
    pub const SHUFFLE_FEE: u128 = 500_000_000;
    pub const MAX_PER_ADDRESS_LIMIT: u32 = 50;

    pub fn mock_params() -> VendingMinterParams {
        VendingMinterParams {
            code_id: 1,
            creation_fee: coin(CREATION_FEE, NATIVE_DENOM),
            min_mint_price: coin(MIN_MINT_PRICE, NATIVE_DENOM),
            mint_fee_bps: MINT_FEE_BPS,
            max_trading_offset_secs: 60 * 60 * 24 * 7,
            extension: ParamsExtension {
                max_token_limit: MAX_TOKEN_LIMIT,
                max_per_address_limit: MAX_PER_ADDRESS_LIMIT,
                airdrop_mint_price: coin(AIRDROP_MINT_PRICE, NATIVE_DENOM),
                airdrop_mint_fee_bps: AIRDROP_MINT_FEE_BPS,
                shuffle_fee: coin(SHUFFLE_FEE, NATIVE_DENOM),
            },
        }
    }

    pub fn mock_init_extension(splits_addr: Option<String>) -> VendingMinterInitMsgExtension {
        VendingMinterInitMsgExtension {
            base_token_uri: "ipfs://aldkfjads".to_string(),
            payment_address: splits_addr,
            start_time: Timestamp::from_nanos(GENESIS_MINT_START_TIME),
            num_tokens: 100,
            mint_price: coin(MIN_MINT_PRICE, NATIVE_DENOM),
            per_address_limit: 5,
            whitelist: None,
        }
    }

    pub fn mock_create_minter(splits_addr: Option<String>) -> VendingMinterCreateMsg {
        VendingMinterCreateMsg {
            init_msg: mock_init_extension(splits_addr),
            collection_params: mock_collection_params(),
        }
    }

    // Upload contract code and instantiate minter contract
    fn setup_minter_contract(
        router: &mut StargazeApp,
        creator: &Addr,
        num_tokens: u32,
        splits_addr: Option<String>,
    ) -> (Addr, MinterConfigResponse) {
        let minter_code_id = router.store_code(contract_minter());
        println!("minter_code_id: {}", minter_code_id);
        let creation_fee = coins(CREATION_FEE, NATIVE_DENOM);

        let factory_code_id = router.store_code(contract_factory());
        println!("factory_code_id: {}", factory_code_id);

        let mut params = mock_params();
        params.code_id = minter_code_id;

        let factory_addr = router
            .instantiate_contract(
                factory_code_id,
                creator.clone(),
                &vending_factory::msg::InstantiateMsg { params },
                &[],
                "factory",
                None,
            )
            .unwrap();

        let sg721_code_id = router.store_code(contract_sg721());
        println!("sg721_code_id: {}", sg721_code_id);

        let mut msg = mock_create_minter(splits_addr);
        msg.init_msg.mint_price = coin(MINT_PRICE, NATIVE_DENOM);
        msg.init_msg.num_tokens = num_tokens;
        msg.collection_params.code_id = sg721_code_id;
        msg.collection_params.info.creator = creator.to_string();

        let msg = Sg2ExecuteMsg::CreateMinter(msg);

        let res = router.execute_contract(creator.clone(), factory_addr, &msg, &creation_fee);

        // could get the minter address from the response above, but we know its contract1
        let minter_addr = Addr::unchecked("contract1");

        let config: MinterConfigResponse = router
            .wrap()
            .query_wasm_smart(minter_addr.clone(), &QueryMsg::Config {})
            .unwrap();

        (minter_addr, config)
    }
    fn setup_accounts(router: &mut StargazeApp) -> (Addr, Addr) {
        let buyer = Addr::unchecked(USER);
        let creator = Addr::unchecked("creator");
        // 3,000 tokens
        let creator_funds = coins(INITIAL_BALANCE + CREATION_FEE, NATIVE_DENOM);
        // 2,000 tokens
        let buyer_funds = coins(INITIAL_BALANCE, NATIVE_DENOM);
        router
            .sudo(CwSudoMsg::Bank({
                BankSudo::Mint {
                    to_address: creator.to_string(),
                    amount: creator_funds.clone(),
                }
            }))
            .map_err(|err| println!("{:?}", err))
            .ok();

        router
            .sudo(CwSudoMsg::Bank({
                BankSudo::Mint {
                    to_address: buyer.to_string(),
                    amount: buyer_funds.clone(),
                }
            }))
            .map_err(|err| println!("{:?}", err))
            .ok();

        // Check native balances
        let creator_native_balances = router.wrap().query_all_balances(creator.clone()).unwrap();
        assert_eq!(creator_native_balances, creator_funds);

        // Check native balances
        let buyer_native_balances = router.wrap().query_all_balances(buyer.clone()).unwrap();
        assert_eq!(buyer_native_balances, buyer_funds);

        (creator, buyer)
    }

    // Set blockchain time to after mint by default
    fn setup_block_time(router: &mut StargazeApp, nanos: u64, height: Option<u64>) {
        let mut block = router.block_info();
        block.time = Timestamp::from_nanos(nanos);
        if let Some(h) = height {
            block.height = h;
        }
        router.set_block(block);
    }

    #[test]
    fn update_profile() {
        let mut app = instantiate_contracts();

        let (creator, user) = setup_accounts(&mut app);
        let num_tokens = 2;
        let (minter_addr, config) = setup_minter_contract(&mut app, &creator, num_tokens, None);

        let token_id = "king arthur".to_string();
        let mint_msg = MintMsg::<Extension> {
            token_id: token_id.to_string(),
            owner: user.to_string(),
            token_uri: None,
            extension: None,
        };

        setup_block_time(&mut app, GENESIS_MINT_START_TIME + 1, None);
        let exec_msg = Sg721ExecuteMsg::Mint(mint_msg.clone());
        app.execute_contract(
            creator.clone(),
            Addr::unchecked(config.sg721_address),
            &exec_msg,
            &coins(MINT_PRICE, NATIVE_DENOM),
        )
        .unwrap();

        mint_and_list(&mut app, NAME, USER);

        // update profile
        // let profile = Some(NFT {
        //     collection: Addr::unchecked(config.sg721_address),
        //     token_id,
        // });
        // let msg = Sg721NameExecuteMsg::UpdateProfile {
        //     name: NAME.to_string(),
        //     profile,
        // };

        let msg = Sg721NameExecuteMsg::UpdateBio {
            name: NAME.to_string(),
            bio: Some("something".to_string()),
        };

        let res: NftInfoResponse<Metadata<Extension>> = app
            .wrap()
            .query_wasm_smart(
                COLLECTION.to_string(),
                &NftInfo {
                    token_id: NAME.to_string(),
                },
            )
            .unwrap();
        println!("{:?}", res.extension.bio);
        // assert!(false);

        let res = app
            .execute_contract(
                Addr::unchecked(USER),
                Addr::unchecked(COLLECTION.to_string()),
                &Sg721NameExecuteMsg::UpdateBio {
                    name: NAME.to_string(),
                    bio: Some("something".to_string()),
                },
                &[],
            )
            .unwrap_err();
        println!("{:?}", res);
        assert!(false);
        // assert!(res.is_ok());
    }
}
