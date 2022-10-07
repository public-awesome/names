#![cfg(test)]
use crate::error::ContractError;
use crate::msg::{
    AskCountResponse, AskOffset, AskResponse, AsksResponse, BidOffset, BidResponse, ParamsResponse,
    SudoMsg,
};
use crate::msg::{BidsResponse, ExecuteMsg, QueryMsg};
use crate::state::{Bid, SUDO_PARAMS};
use cosmwasm_std::testing::{mock_dependencies, mock_env};
use cosmwasm_std::{Addr, Empty, Timestamp};
use cw721::{Cw721QueryMsg, OwnerOfResponse};
use cw721_base::msg::{ExecuteMsg as Cw721ExecuteMsg, MintMsg};
use cw_multi_test::{BankSudo, Contract, ContractWrapper, Executor, SudoMsg as CwSudoMsg};
use sg721::CollectionInfo;
use sg_controllers::HooksResponse;
use sg_multi_test::StargazeApp;
use sg_std::StargazeMsgWrapper;

use cosmwasm_std::{coin, coins, Coin, Decimal, Uint128};
use cw_utils::{Duration, Expiration};
// use sg721_name::CollectionInfo;
use sg721_name::InstantiateMsg as Sg721InstantiateMsg;
use sg_std::NATIVE_DENOM;

pub const TOKEN_ID: u32 = 123;
pub const CREATION_FEE: u128 = 1_000_000_000;
pub const INITIAL_BALANCE: u128 = 2000;

// Governance parameters
pub const TRADING_FEE_BPS: u64 = 200; // 2%

pub fn custom_mock_app() -> StargazeApp {
    StargazeApp::default()
}

pub fn contract_marketplace() -> Box<dyn Contract<StargazeMsgWrapper>> {
    let contract = ContractWrapper::new(
        crate::execute::execute,
        crate::execute::instantiate,
        crate::query::query,
    )
    .with_sudo(crate::sudo::sudo);
    Box::new(contract)
}

pub fn contract_sg721_name() -> Box<dyn Contract<StargazeMsgWrapper>> {
    let contract = ContractWrapper::new(
        sg721_name::entry::execute,
        sg721_name::entry::instantiate,
        sg721_name::entry::query,
    );
    Box::new(contract)
}

pub fn setup_block_time(router: &mut StargazeApp, seconds: u64) {
    let mut block = router.block_info();
    block.time = Timestamp::from_seconds(seconds);
    router.set_block(block);
}

// Instantiates all needed contracts for testing
pub fn setup_contracts(
    router: &mut StargazeApp,
    creator: &Addr,
) -> Result<(Addr, Addr), ContractError> {
    // Instantiate marketplace contract
    let marketplace_id = router.store_code(contract_marketplace());
    let msg = crate::msg::InstantiateMsg {
        trading_fee_bps: TRADING_FEE_BPS,
        min_price: Uint128::from(5u128),
    };
    let marketplace = router
        .instantiate_contract(
            marketplace_id,
            creator.clone(),
            &msg,
            &[],
            "Marketplace",
            None,
        )
        .unwrap();
    // println!("marketplace: {:?}", marketplace);

    // Setup media contract
    let sg721_id = router.store_code(contract_sg721_name());
    let msg = Sg721InstantiateMsg {
        name: String::from("Test Coin"),
        symbol: String::from("TEST"),
        minter: creator.to_string(),
        collection_info: CollectionInfo {
            creator: creator.to_string(),
            description: String::from("Stargaze Monkeys"),
            image:
                "ipfs://bafybeigi3bwpvyvsmnbj46ra4hyffcxdeaj6ntfk5jpic5mx27x6ih2qvq/images/1.png"
                    .to_string(),
            external_link: Some("https://example.com/external.html".to_string()),
            royalty_info: None,
            explicit_content: false,
            trading_start_time: None,
        },
    };
    let collection = router
        .instantiate_contract(sg721_id, creator.clone(), &msg, &[], "NFT", None)
        .unwrap();
    println!("collection: {:?}", collection);

    Ok((marketplace, collection))
}

pub fn setup_collection(router: &mut StargazeApp, creator: &Addr) -> Result<Addr, ContractError> {
    // Setup media contract
    let sg721_id = router.store_code(contract_sg721_name());
    let msg = Sg721InstantiateMsg {
        name: String::from("Test Collection 2"),
        symbol: String::from("TEST 2"),
        minter: creator.to_string(),
        collection_info: CollectionInfo {
            creator: creator.to_string(),
            description: String::from("Stargaze Monkeys 2"),
            image:
                "ipfs://bafybeigi3bwpvyvsmnbj46ra4hyffcxdeaj6ntfk5jpic5mx27x6ih2qvq/images/1.png"
                    .to_string(),
            external_link: Some("https://example.com/external.html".to_string()),
            royalty_info: None,
            explicit_content: false,
            trading_start_time: None,
        },
    };
    let collection = router
        .instantiate_contract(sg721_id, creator.clone(), &msg, &[], "NFT", None)
        .unwrap();
    // println!("collection 2: {:?}", collection);
    Ok(collection)
}

// Intializes accounts with balances
pub fn setup_accounts(router: &mut StargazeApp) -> Result<(Addr, Addr, Addr), ContractError> {
    let owner: Addr = Addr::unchecked("owner");
    let bidder: Addr = Addr::unchecked("bidder");
    let creator: Addr = Addr::unchecked("creator");
    let creator_funds: Vec<Coin> = coins(CREATION_FEE, NATIVE_DENOM);
    let funds: Vec<Coin> = coins(INITIAL_BALANCE, NATIVE_DENOM);
    router
        .sudo(CwSudoMsg::Bank({
            BankSudo::Mint {
                to_address: owner.to_string(),
                amount: funds.clone(),
            }
        }))
        .map_err(|err| println!("{:?}", err))
        .ok();
    router
        .sudo(CwSudoMsg::Bank({
            BankSudo::Mint {
                to_address: bidder.to_string(),
                amount: funds.clone(),
            }
        }))
        .map_err(|err| println!("{:?}", err))
        .ok();
    router
        .sudo(CwSudoMsg::Bank({
            BankSudo::Mint {
                to_address: creator.to_string(),
                amount: creator_funds.clone(),
            }
        }))
        .map_err(|err| println!("{:?}", err))
        .ok();

    // Check native balances
    let owner_native_balances = router.wrap().query_all_balances(owner.clone()).unwrap();
    assert_eq!(owner_native_balances, funds);
    let bidder_native_balances = router.wrap().query_all_balances(bidder.clone()).unwrap();
    assert_eq!(bidder_native_balances, funds);
    let creator_native_balances = router.wrap().query_all_balances(creator.clone()).unwrap();
    assert_eq!(creator_native_balances, creator_funds);

    Ok((owner, bidder, creator))
}

pub fn add_funds_for_incremental_fee(
    router: &mut StargazeApp,
    receiver: &Addr,
    fee_amount: u128,
    fee_count: u128,
) -> Result<(), ContractError> {
    let fee_funds = coins(fee_amount * fee_count, NATIVE_DENOM);
    router
        .sudo(CwSudoMsg::Bank({
            BankSudo::Mint {
                to_address: receiver.to_string(),
                amount: fee_funds,
            }
        }))
        .map_err(|err| println!("{:?}", err))
        .ok();
    Ok(())
}

pub fn setup_second_bidder_account(router: &mut StargazeApp) -> Result<Addr, ContractError> {
    let bidder2: Addr = Addr::unchecked("bidder2");
    let funds: Vec<Coin> = coins(CREATION_FEE + INITIAL_BALANCE, NATIVE_DENOM);
    router
        .sudo(CwSudoMsg::Bank({
            BankSudo::Mint {
                to_address: bidder2.to_string(),
                amount: funds.clone(),
            }
        }))
        .map_err(|err| println!("{:?}", err))
        .ok();

    // Check native balances
    let bidder_native_balances = router.wrap().query_all_balances(bidder2.clone()).unwrap();
    assert_eq!(bidder_native_balances, funds);

    Ok(bidder2)
}

// Mints an NFT for a creator
pub fn mint(router: &mut StargazeApp, creator: &Addr, collection: &Addr, token_id: u32) {
    let mint_for_creator_msg = Cw721ExecuteMsg::Mint(MintMsg {
        token_id: token_id.to_string(),
        owner: creator.clone().to_string(),
        token_uri: Some("https://starships.example.com/Starship/Enterprise.json".into()),
        extension: Empty {},
    });
    let res = router.execute_contract(
        creator.clone(),
        collection.clone(),
        &mint_for_creator_msg,
        &[],
    );
    assert!(res.is_ok());
}

pub fn mint_for(
    router: &mut StargazeApp,
    owner: &Addr,
    creator: &Addr,
    collection: &Addr,
    token_id: u32,
) {
    let mint_for_creator_msg = Cw721ExecuteMsg::Mint(MintMsg {
        token_id: token_id.to_string(),
        owner: owner.to_string(),
        token_uri: Some("https://starships.example.com/Starship/Enterprise.json".into()),
        extension: Empty {},
    });
    let res = router.execute_contract(
        creator.clone(),
        collection.clone(),
        &mint_for_creator_msg,
        &[],
    );
    assert!(res.is_ok());
}

pub fn approve(
    router: &mut StargazeApp,
    creator: &Addr,
    collection: &Addr,
    marketplace: &Addr,
    token_id: u32,
) {
    let approve_msg = Cw721ExecuteMsg::<Empty>::Approve {
        spender: marketplace.to_string(),
        token_id: token_id.to_string(),
        expires: None,
    };
    let res = router.execute_contract(creator.clone(), collection.clone(), &approve_msg, &[]);
    assert!(res.is_ok());
}

fn transfer(
    router: &mut StargazeApp,
    creator: &Addr,
    recipient: &Addr,
    collection: &Addr,
    token_id: u32,
) {
    let transfer_msg = Cw721ExecuteMsg::<Empty>::TransferNft {
        recipient: recipient.to_string(),
        token_id: token_id.to_string(),
    };
    let res = router.execute_contract(creator.clone(), collection.clone(), &transfer_msg, &[]);
    assert!(res.is_ok());
}

pub fn burn(router: &mut StargazeApp, creator: &Addr, collection: &Addr, token_id: u32) {
    let transfer_msg = Cw721ExecuteMsg::<Empty>::Burn {
        token_id: token_id.to_string(),
    };
    let res = router.execute_contract(creator.clone(), collection.clone(), &transfer_msg, &[]);
    assert!(res.is_ok());
}

#[test]
fn try_set_accept_bid() {
    let mut router = custom_mock_app();

    // Setup intial accounts
    let (owner, bidder, creator) = setup_accounts(&mut router).unwrap();

    // Instantiate and configure contracts
    let (marketplace, collection) = setup_contracts(&mut router, &creator).unwrap();

    // // Mint NFT for creator
    // mint(&mut router, &creator, &collection, TOKEN_ID);
    // approve(&mut router, &creator, &collection, &marketplace, TOKEN_ID);

    // // Should error with expiry lower than min
    // let set_ask = ExecuteMsg::SetAsk {
    //     token_id: TOKEN_ID.to_string(),
    //     funds_recipient: None,
    // };
    // let res = router.execute_contract(creator.clone(), marketplace.clone(), &set_ask, &[]);
    // assert!(res.is_err());

    // // An asking price is made by the creator
    // let set_ask = ExecuteMsg::SetAsk {
    //     token_id: TOKEN_ID.to_string(),
    //     funds_recipient: None,
    // };
    // let res = router.execute_contract(creator.clone(), marketplace.clone(), &set_ask, &[]);
    // assert!(res.is_ok());

    // // Transfer nft from creator to owner. Creates a stale ask that needs to be updated
    // transfer(&mut router, &creator, &owner, &collection, TOKEN_ID);

    // // Bidder makes bid
    // let set_bid_msg = ExecuteMsg::SetBid {
    //     token_id: TOKEN_ID.to_string(),
    // };
    // let res = router.execute_contract(
    //     bidder.clone(),
    //     marketplace.clone(),
    //     &set_bid_msg,
    //     &coins(100, NATIVE_DENOM),
    // );
    // assert!(res.is_ok());

    // // Check contract has been paid
    // let contract_balances = router
    //     .wrap()
    //     .query_all_balances(marketplace.clone())
    //     .unwrap();
    // assert_eq!(contract_balances, coins(100, NATIVE_DENOM));

    // // Check creator hasn't been paid yet
    // let creator_native_balances = router.wrap().query_all_balances(creator.clone()).unwrap();
    // assert_eq!(creator_native_balances, vec![]);

    // // Creator accepts bid
    // let accept_bid_msg = ExecuteMsg::AcceptBid {
    //     token_id: TOKEN_ID.to_string(),
    //     bidder: bidder.to_string(),
    // };
    // let res = router.execute_contract(creator.clone(), marketplace.clone(), &accept_bid_msg, &[]);
    // assert!(res.is_ok());

    // // Check money is transferred
    // let creator_native_balances = router.wrap().query_all_balances(creator).unwrap();
    // // 100  - 2 (fee)
    // assert_eq!(creator_native_balances, coins(100 - 2, NATIVE_DENOM));
    // let bidder_native_balances = router.wrap().query_all_balances(bidder.clone()).unwrap();
    // assert_eq!(
    //     bidder_native_balances,
    //     coins(INITIAL_BALANCE - 100, NATIVE_DENOM)
    // );

    // // Check NFT is transferred
    // let query_owner_msg = Cw721QueryMsg::OwnerOf {
    //     token_id: TOKEN_ID.to_string(),
    //     include_expired: None,
    // };
    // let res: OwnerOfResponse = router
    //     .wrap()
    //     .query_wasm_smart(collection, &query_owner_msg)
    //     .unwrap();
    // assert_eq!(res.owner, bidder.to_string());

    // // Check contract has zero balance
    // let contract_balances = router.wrap().query_all_balances(marketplace).unwrap();
    // assert_eq!(contract_balances, []);
}

// #[test]
// fn try_set_accept_bid_no_ask() {
//     let mut router = custom_mock_app();

//     // Setup intial accounts
//     let (_owner, bidder, creator) = setup_accounts(&mut router).unwrap();

//     // Instantiate and configure contracts
//     let (marketplace, collection) = setup_contracts(&mut router, &creator).unwrap();

//     // Mint NFT for creator
//     mint(&mut router, &creator, &collection, TOKEN_ID);
//     approve(&mut router, &creator, &collection, &marketplace, TOKEN_ID);

//     // Bidder makes bid
//     let set_bid_msg = ExecuteMsg::SetBid {
//         token_id: TOKEN_ID,
//         expires: router.block_info().time.plus_seconds(MIN_EXPIRY + 1),
//     };
//     let res = router.execute_contract(
//         bidder.clone(),
//         marketplace.clone(),
//         &set_bid_msg,
//         &coins(100, NATIVE_DENOM),
//     );
//     assert!(res.is_ok());

//     // Check contract has been paid
//     let contract_balances = router
//         .wrap()
//         .query_all_balances(marketplace.clone())
//         .unwrap();
//     assert_eq!(contract_balances, coins(100, NATIVE_DENOM));

//     // Check creator hasn't been paid yet
//     let creator_native_balances = router.wrap().query_all_balances(creator.clone()).unwrap();
//     assert_eq!(creator_native_balances, vec![]);

//     // Creator accepts bid
//     let accept_bid_msg = ExecuteMsg::AcceptBid {
//         token_id: TOKEN_ID,
//         bidder: bidder.to_string(),
//     };
//     let res = router.execute_contract(creator.clone(), marketplace.clone(), &accept_bid_msg, &[]);
//     assert!(res.is_ok());

//     // Check money is transferred
//     let creator_native_balances = router.wrap().query_all_balances(creator).unwrap();
//     // 100  - 2 (fee)
//     assert_eq!(creator_native_balances, coins(100 - 2, NATIVE_DENOM));
//     let bidder_native_balances = router.wrap().query_all_balances(bidder.clone()).unwrap();
//     assert_eq!(
//         bidder_native_balances,
//         coins(INITIAL_BALANCE - 100, NATIVE_DENOM)
//     );

//     // Check NFT is transferred
//     let query_owner_msg = Cw721QueryMsg::OwnerOf {
//         token_id: TOKEN_ID.to_string(),
//         include_expired: None,
//     };
//     let res: OwnerOfResponse = router
//         .wrap()
//         .query_wasm_smart(collection, &query_owner_msg)
//         .unwrap();
//     assert_eq!(res.owner, bidder.to_string());

//     // Check contract has zero balance
//     let contract_balances = router.wrap().query_all_balances(marketplace).unwrap();
//     assert_eq!(contract_balances, []);
// }

// #[test]
// fn try_query_asks() {
//     let mut router = custom_mock_app();

//     // Setup intial accounts
//     let (_owner, _, creator) = setup_accounts(&mut router).unwrap();

//     // Instantiate and configure contracts
//     let (marketplace, collection) = setup_contracts(&mut router, &creator).unwrap();

//     // Add funds to creator for listing fees
//     add_funds_for_incremental_fee(&mut router, &creator, LISTING_FEE, 1u128).unwrap();

//     // Mint NFT for creator
//     mint(&mut router, &creator, &collection, TOKEN_ID);
//     approve(&mut router, &creator, &collection, &marketplace, TOKEN_ID);

//     // test before ask is made, without using pagination
//     let query_asks_msg = QueryMsg::Asks {
//         include_inactive: Some(true),
//         start_after: None,
//         limit: None,
//     };
//     let res: AsksResponse = router
//         .wrap()
//         .query_wasm_smart(marketplace.clone(), &query_asks_msg)
//         .unwrap();
//     assert_eq!(res.asks, vec![]);

//     // An asking price is made by the creator
//     let set_ask = ExecuteMsg::SetAsk {
//         token_id: TOKEN_ID,
//         funds_recipient: None,
//     };
//     let res = router.execute_contract(
//         creator.clone(),
//         marketplace.clone(),
//         &set_ask,
//         &listing_funds(LISTING_FEE).unwrap(),
//     );
//     assert!(res.is_ok());

//     // test after ask is made
//     let res: AsksResponse = router
//         .wrap()
//         .query_wasm_smart(marketplace.clone(), &query_asks_msg)
//         .unwrap();
//     assert_eq!(res.asks[0].token_id, TOKEN_ID);
//     assert_eq!(res.asks[0].price.u128(), 110);

//     // test pagination, starting when tokens exist
//     let query_asks_msg = QueryMsg::Asks {
//         include_inactive: Some(true),
//         start_after: Some(TOKEN_ID - 1),
//         limit: None,
//     };
//     let res: AsksResponse = router
//         .wrap()
//         .query_wasm_smart(marketplace.clone(), &query_asks_msg)
//         .unwrap();
//     assert_eq!(res.asks[0].token_id, TOKEN_ID);

//     // test pagination, starting when token don't exist
//     let query_asks_msg = QueryMsg::Asks {
//         include_inactive: Some(true),
//         start_after: Some(TOKEN_ID),
//         limit: None,
//     };
//     let res: AsksResponse = router
//         .wrap()
//         .query_wasm_smart(marketplace.clone(), &query_asks_msg)
//         .unwrap();
//     assert_eq!(res.asks.len(), 0);

//     // test pagination starting before token exists
//     let query_reverse_asks_msg = QueryMsg::ReverseAsks {
//         include_inactive: Some(true),
//         start_before: Some(TOKEN_ID + 1),
//         limit: None,
//     };
//     let res: AsksResponse = router
//         .wrap()
//         .query_wasm_smart(marketplace.clone(), &query_reverse_asks_msg)
//         .unwrap();
//     assert_eq!(res.asks.len(), 1);

// }

// #[test]
// fn try_query_sorted_asks() {
//     let mut router = custom_mock_app();

//     // Setup intial accounts
//     let (_owner, _, creator) = setup_accounts(&mut router).unwrap();

//     // Add funds to creator for listing fees
//     add_funds_for_incremental_fee(&mut router, &creator, LISTING_FEE, 3u128).unwrap();

//     // Instantiate and configure contracts
//     let (marketplace, collection) = setup_contracts(&mut router, &creator).unwrap();

//     // Mint NFTs for creator
//     mint(&mut router, &creator, &collection, TOKEN_ID);
//     approve(&mut router, &creator, &collection, &marketplace, TOKEN_ID);
//     mint(&mut router, &creator, &collection, TOKEN_ID + 1);
//     approve(
//         &mut router,
//         &creator,
//         &collection,
//         &marketplace,
//         TOKEN_ID + 1,
//     );
//     mint(&mut router, &creator, &collection, TOKEN_ID + 2);
//     approve(
//         &mut router,
//         &creator,
//         &collection,
//         &marketplace,
//         TOKEN_ID + 2,
//     );

//     // An asking price is made by the creator
//     let set_ask = ExecuteMsg::SetAsk {
//         sale_type: SaleType::FixedPrice,
//         collection: collection.to_string(),
//         token_id: TOKEN_ID,
//         price: coin(110, NATIVE_DENOM),
//         funds_recipient: None,
//         reserve_for: None,
//         expires: router.block_info().time.plus_seconds(MIN_EXPIRY + 1),
//         finders_fee_bps: Some(0),
//     };
//     let res = router.execute_contract(
//         creator.clone(),
//         marketplace.clone(),
//         &set_ask,
//         &listing_funds(LISTING_FEE).unwrap(),
//     );
//     assert!(res.is_ok());
//     // An asking price is made by the creator
//     let set_ask = ExecuteMsg::SetAsk {
//         sale_type: SaleType::FixedPrice,
//         collection: collection.to_string(),
//         token_id: TOKEN_ID + 1,
//         price: coin(109, NATIVE_DENOM),
//         funds_recipient: None,
//         reserve_for: None,
//         expires: router.block_info().time.plus_seconds(MIN_EXPIRY + 1),
//         finders_fee_bps: Some(0),
//     };
//     let res = router.execute_contract(
//         creator.clone(),
//         marketplace.clone(),
//         &set_ask,
//         &listing_funds(LISTING_FEE).unwrap(),
//     );
//     assert!(res.is_ok());
//     // An asking price is made by the creator
//     let set_ask = ExecuteMsg::SetAsk {
//         sale_type: SaleType::FixedPrice,
//         collection: collection.to_string(),
//         token_id: TOKEN_ID + 2,
//         price: coin(111, NATIVE_DENOM),
//         funds_recipient: None,
//         reserve_for: None,
//         expires: router.block_info().time.plus_seconds(MIN_EXPIRY + 1),
//         finders_fee_bps: Some(0),
//     };
//     let res = router.execute_contract(
//         creator.clone(),
//         marketplace.clone(),
//         &set_ask,
//         &listing_funds(LISTING_FEE).unwrap(),
//     );
//     assert!(res.is_ok());

//     let query_asks_msg = QueryMsg::AsksSortedByPrice {
//         collection: collection.to_string(),
//         include_inactive: Some(true),
//         start_after: None,
//         limit: None,
//     };
//     let res: AsksResponse = router
//         .wrap()
//         .query_wasm_smart(marketplace.clone(), &query_asks_msg)
//         .unwrap();
//     assert_eq!(res.asks.len(), 3);
//     assert_eq!(res.asks[0].price.u128(), 109u128);
//     assert_eq!(res.asks[1].price.u128(), 110u128);
//     assert_eq!(res.asks[2].price.u128(), 111u128);

//     let start_after = AskOffset::new(res.asks[0].price, res.asks[0].token_id);
//     let query_msg = QueryMsg::AsksSortedByPrice {
//         collection: collection.to_string(),
//         include_inactive: Some(true),
//         start_after: Some(start_after),
//         limit: None,
//     };

//     let res: AsksResponse = router
//         .wrap()
//         .query_wasm_smart(marketplace.clone(), &query_msg)
//         .unwrap();
//     assert_eq!(res.asks.len(), 2);
//     assert_eq!(res.asks[0].price.u128(), 110u128);
//     assert_eq!(res.asks[1].price.u128(), 111u128);

//     let reverse_query_asks_msg = QueryMsg::ReverseAsksSortedByPrice {
//         collection: collection.to_string(),
//         include_inactive: Some(true),
//         start_before: None,
//         limit: None,
//     };

//     let res: AsksResponse = router
//         .wrap()
//         .query_wasm_smart(marketplace.clone(), &reverse_query_asks_msg)
//         .unwrap();
//     assert_eq!(res.asks.len(), 3);
//     assert_eq!(res.asks[0].price.u128(), 111u128);
//     assert_eq!(res.asks[1].price.u128(), 110u128);
//     assert_eq!(res.asks[2].price.u128(), 109u128);

//     let start_before = AskOffset::new(res.asks[0].price, res.asks[0].token_id);
//     let reverse_query_asks_start_before_first_desc_msg = QueryMsg::ReverseAsksSortedByPrice {
//         collection: collection.to_string(),
//         include_inactive: Some(true),
//         start_before: Some(start_before),
//         limit: None,
//     };

//     let res: AsksResponse = router
//         .wrap()
//         .query_wasm_smart(
//             marketplace.clone(),
//             &reverse_query_asks_start_before_first_desc_msg,
//         )
//         .unwrap();
//     assert_eq!(res.asks.len(), 2);
//     assert_eq!(res.asks[0].price.u128(), 110u128);
//     assert_eq!(res.asks[1].price.u128(), 109u128);

//     let res: AskCountResponse = router
//         .wrap()
//         .query_wasm_smart(
//             marketplace.clone(),
//             &QueryMsg::AskCount {
//                 collection: collection.to_string(),
//             },
//         )
//         .unwrap();
//     assert_eq!(res.count, 3);
// }

// #[test]
// fn try_query_asks_by_seller() {
//     let mut router = custom_mock_app();

//     // Setup intial accounts
//     let (owner, _, creator) = setup_accounts(&mut router).unwrap();

//     let owner2: Addr = Addr::unchecked("owner2");
//     // Add funds to owner2 for creation fees
//     add_funds_for_incremental_fee(&mut router, &owner2, CREATION_FEE, 1u128).unwrap();
//     // Add funds to owner2 for listing fees
//     add_funds_for_incremental_fee(&mut router, &owner2, LISTING_FEE, 2u128).unwrap();

//     // Instantiate and configure contracts
//     let (marketplace, collection) = setup_contracts(&mut router, &creator).unwrap();

//     // Mint NFT for creator
//     mint_for(&mut router, &owner, &creator, &collection, TOKEN_ID);
//     approve(&mut router, &owner, &collection, &marketplace, TOKEN_ID);
//     mint_for(&mut router, &owner2, &creator, &collection, TOKEN_ID + 1);
//     approve(
//         &mut router,
//         &owner2,
//         &collection,
//         &marketplace,
//         TOKEN_ID + 1,
//     );
//     mint_for(&mut router, &owner2, &creator, &collection, TOKEN_ID + 2);
//     approve(
//         &mut router,
//         &owner2,
//         &collection,
//         &marketplace,
//         TOKEN_ID + 2,
//     );

//     // Owner1 lists their token for sale
//     let set_ask = ExecuteMsg::SetAsk {
//         sale_type: SaleType::FixedPrice,
//         collection: collection.to_string(),
//         token_id: TOKEN_ID,
//         price: coin(110, NATIVE_DENOM),
//         funds_recipient: None,
//         reserve_for: None,
//         expires: router.block_info().time.plus_seconds(MIN_EXPIRY + 1),
//         finders_fee_bps: Some(0),
//     };
//     let res = router.execute_contract(
//         owner.clone(),
//         marketplace.clone(),
//         &set_ask,
//         &listing_funds(LISTING_FEE).unwrap(),
//     );
//     assert!(res.is_ok());

//     // Owner2 lists their token for sale
//     let set_ask = ExecuteMsg::SetAsk {
//         sale_type: SaleType::FixedPrice,
//         collection: collection.to_string(),
//         token_id: TOKEN_ID + 1,
//         price: coin(109, NATIVE_DENOM),
//         funds_recipient: None,
//         reserve_for: None,
//         expires: router.block_info().time.plus_seconds(MIN_EXPIRY + 1),
//         finders_fee_bps: Some(0),
//     };
//     let res = router.execute_contract(
//         owner2.clone(),
//         marketplace.clone(),
//         &set_ask,
//         &listing_funds(LISTING_FEE).unwrap(),
//     );
//     assert!(res.is_ok());

//     // Owner2 lists another token for sale
//     let set_ask = ExecuteMsg::SetAsk {
//         sale_type: SaleType::FixedPrice,
//         collection: collection.to_string(),
//         token_id: TOKEN_ID + 2,
//         price: coin(111, NATIVE_DENOM),
//         funds_recipient: None,
//         reserve_for: None,
//         expires: router.block_info().time.plus_seconds(MIN_EXPIRY + 1),
//         finders_fee_bps: Some(0),
//     };
//     let res = router.execute_contract(
//         owner2.clone(),
//         marketplace.clone(),
//         &set_ask,
//         &listing_funds(LISTING_FEE).unwrap(),
//     );
//     assert!(res.is_ok());

//     let res: AskCountResponse = router
//         .wrap()
//         .query_wasm_smart(
//             marketplace.clone(),
//             &QueryMsg::AskCount {
//                 collection: collection.to_string(),
//             },
//         )
//         .unwrap();
//     assert_eq!(res.count, 3);

//     // owner1 should only have 1 token
//     let query_asks_msg = QueryMsg::AsksBySeller {
//         seller: owner.to_string(),
//         include_inactive: Some(true),
//         start_after: None,
//         limit: None,
//     };
//     let res: AsksResponse = router
//         .wrap()
//         .query_wasm_smart(marketplace.to_string(), &query_asks_msg)
//         .unwrap();
//     assert_eq!(res.asks.len(), 1);

//     // owner2 should have 2 token
//     let query_asks_msg = QueryMsg::AsksBySeller {
//         seller: owner2.to_string(),
//         include_inactive: Some(true),
//         start_after: None,
//         limit: None,
//     };
//     let res: AsksResponse = router
//         .wrap()
//         .query_wasm_smart(marketplace.to_string(), &query_asks_msg)
//         .unwrap();
//     assert_eq!(res.asks.len(), 2);

//     // owner2 should have 0 tokens when paginated by a non-existing collection
//     let query_asks_msg = QueryMsg::AsksBySeller {
//         seller: owner2.to_string(),
//         include_inactive: Some(true),
//         start_after: Some(CollectionOffset::new(
//             "non-existing-collection".to_string(),
//             TOKEN_ID,
//         )),
//         limit: None,
//     };
//     let res: AsksResponse = router
//         .wrap()
//         .query_wasm_smart(marketplace.to_string(), &query_asks_msg)
//         .unwrap();
//     assert_eq!(res.asks.len(), 0);

//     // owner2 should have 2 tokens when paginated by a existing collection
//     let query_asks_msg = QueryMsg::AsksBySeller {
//         seller: owner2.to_string(),
//         include_inactive: Some(true),
//         start_after: Some(CollectionOffset::new(collection.to_string(), 0)),
//         limit: None,
//     };
//     let res: AsksResponse = router
//         .wrap()
//         .query_wasm_smart(marketplace.to_string(), &query_asks_msg)
//         .unwrap();
//     assert_eq!(res.asks.len(), 2);

//     // owner2 should have 1 token when paginated by a existing collection starting after a token
//     let query_asks_msg = QueryMsg::AsksBySeller {
//         seller: owner2.to_string(),
//         include_inactive: Some(true),
//         start_after: Some(CollectionOffset::new(collection.to_string(), TOKEN_ID + 1)),
//         limit: None,
//     };
//     let res: AsksResponse = router
//         .wrap()
//         .query_wasm_smart(marketplace.to_string(), &query_asks_msg)
//         .unwrap();
//     assert_eq!(res.asks.len(), 1);
// }

// #[test]
// fn try_query_sorted_bids() {
//     let mut router = custom_mock_app();

//     // Setup intial accounts
//     let (_owner, bidder, creator) = setup_accounts(&mut router).unwrap();

//     // Instantiate and configure contracts
//     let (marketplace, collection) = setup_contracts(&mut router, &creator).unwrap();

//     // Add funds to creator for listing fees
//     add_funds_for_incremental_fee(&mut router, &creator, LISTING_FEE, 3u128).unwrap();

//     // Mint NFT for creator
//     mint(&mut router, &creator, &collection, TOKEN_ID);
//     approve(&mut router, &creator, &collection, &marketplace, TOKEN_ID);
//     mint(&mut router, &creator, &collection, TOKEN_ID + 1);
//     approve(
//         &mut router,
//         &creator,
//         &collection,
//         &marketplace,
//         TOKEN_ID + 1,
//     );
//     mint(&mut router, &creator, &collection, TOKEN_ID + 2);
//     approve(
//         &mut router,
//         &creator,
//         &collection,
//         &marketplace,
//         TOKEN_ID + 2,
//     );

//     // An asking price is made by the creator
//     let set_ask = ExecuteMsg::SetAsk {
//         sale_type: SaleType::Auction,
//         collection: collection.to_string(),
//         token_id: TOKEN_ID,
//         price: coin(110, NATIVE_DENOM),
//         funds_recipient: None,
//         reserve_for: None,
//         expires: router.block_info().time.plus_seconds(MIN_EXPIRY + 1),
//         finders_fee_bps: Some(0),
//     };
//     let res = router.execute_contract(
//         creator.clone(),
//         marketplace.clone(),
//         &set_ask,
//         &listing_funds(LISTING_FEE).unwrap(),
//     );
//     assert!(res.is_ok());
//     // An asking price is made by the creator
//     let set_ask = ExecuteMsg::SetAsk {
//         sale_type: SaleType::Auction,
//         collection: collection.to_string(),
//         token_id: TOKEN_ID + 1,
//         price: coin(109, NATIVE_DENOM),
//         funds_recipient: None,
//         reserve_for: None,
//         expires: router.block_info().time.plus_seconds(MIN_EXPIRY + 1),
//         finders_fee_bps: Some(0),
//     };
//     let res = router.execute_contract(
//         creator.clone(),
//         marketplace.clone(),
//         &set_ask,
//         &listing_funds(LISTING_FEE).unwrap(),
//     );
//     assert!(res.is_ok());
//     // An asking price is made by the creator
//     let set_ask = ExecuteMsg::SetAsk {
//         sale_type: SaleType::Auction,
//         collection: collection.to_string(),
//         token_id: TOKEN_ID + 2,
//         price: coin(111, NATIVE_DENOM),
//         funds_recipient: None,
//         reserve_for: None,
//         expires: router.block_info().time.plus_seconds(MIN_EXPIRY + 1),
//         finders_fee_bps: Some(0),
//     };
//     let res = router.execute_contract(
//         creator.clone(),
//         marketplace.clone(),
//         &set_ask,
//         &listing_funds(LISTING_FEE).unwrap(),
//     );
//     assert!(res.is_ok());

//     // Bidder makes bid
//     let set_bid_msg = ExecuteMsg::SetBid {
//         sale_type: SaleType::FixedPrice,
//         collection: collection.to_string(),
//         token_id: TOKEN_ID,
//         finders_fee_bps: None,
//         expires: router.block_info().time.plus_seconds(MIN_EXPIRY + 1),
//         finder: None,
//     };
//     let res = router.execute_contract(
//         bidder.clone(),
//         marketplace.clone(),
//         &set_bid_msg,
//         &coins(50, NATIVE_DENOM),
//     );
//     assert!(res.is_ok());
//     // Bidder makes bid
//     let set_bid_msg = ExecuteMsg::SetBid {
//         sale_type: SaleType::FixedPrice,
//         collection: collection.to_string(),
//         token_id: TOKEN_ID + 1,
//         finders_fee_bps: None,
//         expires: router.block_info().time.plus_seconds(MIN_EXPIRY + 1),
//         finder: None,
//     };
//     let res = router.execute_contract(
//         bidder.clone(),
//         marketplace.clone(),
//         &set_bid_msg,
//         &coins(70, NATIVE_DENOM),
//     );
//     assert!(res.is_ok());
//     // Bidder makes bid
//     let set_bid_msg = ExecuteMsg::SetBid {
//         sale_type: SaleType::FixedPrice,
//         collection: collection.to_string(),
//         token_id: TOKEN_ID + 2,
//         finders_fee_bps: None,
//         expires: router.block_info().time.plus_seconds(MIN_EXPIRY + 1),
//         finder: None,
//     };
//     router
//         .execute_contract(
//             bidder.clone(),
//             marketplace.clone(),
//             &set_bid_msg,
//             &coins(1, NATIVE_DENOM),
//         )
//         .unwrap_err();
//     let res = router.execute_contract(
//         bidder,
//         marketplace.clone(),
//         &set_bid_msg,
//         &coins(60, NATIVE_DENOM),
//     );
//     assert!(res.is_ok());

//     let query_bids_msg = QueryMsg::BidsSortedByPrice {
//         collection: collection.to_string(),
//         limit: None,
//         start_after: None,
//     };
//     let res: BidsResponse = router
//         .wrap()
//         .query_wasm_smart(marketplace.clone(), &query_bids_msg)
//         .unwrap();
//     assert_eq!(res.bids.len(), 3);
//     assert_eq!(res.bids[0].price.u128(), 50u128);
//     assert_eq!(res.bids[1].price.u128(), 60u128);
//     assert_eq!(res.bids[2].price.u128(), 70u128);

//     // test adding another bid to an existing ask
//     let bidder2: Addr = Addr::unchecked("bidder2");
//     let funds: Vec<Coin> = coins(INITIAL_BALANCE, NATIVE_DENOM);
//     router
//         .sudo(CwSudoMsg::Bank({
//             BankSudo::Mint {
//                 to_address: bidder2.to_string(),
//                 amount: funds,
//             }
//         }))
//         .map_err(|err| println!("{:?}", err))
//         .ok();

//     // Bidder makes bid
//     let set_bid_msg = ExecuteMsg::SetBid {
//         sale_type: SaleType::FixedPrice,
//         collection: collection.to_string(),
//         token_id: TOKEN_ID,
//         finders_fee_bps: None,
//         expires: router.block_info().time.plus_seconds(MIN_EXPIRY + 1),
//         finder: None,
//     };
//     let res = router.execute_contract(
//         bidder2,
//         marketplace.clone(),
//         &set_bid_msg,
//         &coins(40, NATIVE_DENOM),
//     );
//     assert!(res.is_ok());

//     let res: BidsResponse = router
//         .wrap()
//         .query_wasm_smart(marketplace.clone(), &query_bids_msg)
//         .unwrap();
//     assert_eq!(res.bids.len(), 4);
//     assert_eq!(res.bids[0].price.u128(), 40u128);
//     assert_eq!(res.bids[1].price.u128(), 50u128);
//     assert_eq!(res.bids[2].price.u128(), 60u128);
//     assert_eq!(res.bids[3].price.u128(), 70u128);

//     // test start_after query
//     let start_after = BidOffset {
//         price: res.bids[2].price,
//         token_id: res.bids[2].token_id,
//         bidder: res.bids[2].bidder.clone(),
//     };
//     let query_start_after_bids_msg = QueryMsg::BidsSortedByPrice {
//         collection: collection.to_string(),
//         limit: None,
//         start_after: Some(start_after),
//     };
//     let res: BidsResponse = router
//         .wrap()
//         .query_wasm_smart(marketplace.clone(), &query_start_after_bids_msg)
//         .unwrap();
//     assert_eq!(res.bids.len(), 1);
//     assert_eq!(res.bids[0].price.u128(), 70u128);

//     // test reverse bids query
//     let reverse_query_bids_msg = QueryMsg::ReverseBidsSortedByPrice {
//         collection: collection.to_string(),
//         limit: None,
//         start_before: None,
//     };
//     let res: BidsResponse = router
//         .wrap()
//         .query_wasm_smart(marketplace.clone(), &reverse_query_bids_msg)
//         .unwrap();
//     assert_eq!(res.bids.len(), 4);
//     assert_eq!(res.bids[0].price.u128(), 70u128);
//     assert_eq!(res.bids[1].price.u128(), 60u128);
//     assert_eq!(res.bids[2].price.u128(), 50u128);
//     assert_eq!(res.bids[3].price.u128(), 40u128);

//     // test start_before reverse bids query
//     let start_before = BidOffset {
//         price: res.bids[1].price,
//         token_id: res.bids[1].token_id,
//         bidder: res.bids[1].bidder.clone(),
//     };
//     let reverse_query_start_before_bids_msg = QueryMsg::ReverseBidsSortedByPrice {
//         collection: collection.to_string(),
//         limit: None,
//         start_before: Some(start_before),
//     };
//     let res: BidsResponse = router
//         .wrap()
//         .query_wasm_smart(marketplace, &reverse_query_start_before_bids_msg)
//         .unwrap();
//     assert_eq!(res.bids.len(), 2);
//     assert_eq!(res.bids[0].price.u128(), 50u128);
//     assert_eq!(res.bids[1].price.u128(), 40u128);
// }

// #[test]
// fn try_query_bids() {
//     let mut router = custom_mock_app();
//     // Setup intial accounts
//     let (_owner, bidder, creator) = setup_accounts(&mut router).unwrap();

//     // Instantiate and configure contracts
//     let (marketplace, collection) = setup_contracts(&mut router, &creator).unwrap();

//     // Add funds to creator for listing fees
//     add_funds_for_incremental_fee(&mut router, &creator, LISTING_FEE, 1u128).unwrap();

//     // Mint NFT for creator
//     mint(&mut router, &creator, &collection, TOKEN_ID);
//     approve(&mut router, &creator, &collection, &marketplace, TOKEN_ID);

//     // An asking price is made by the creator
//     let set_ask = ExecuteMsg::SetAsk {
//         sale_type: SaleType::Auction,
//         collection: collection.to_string(),
//         token_id: TOKEN_ID,
//         price: coin(110, NATIVE_DENOM),
//         funds_recipient: None,
//         reserve_for: None,
//         expires: router.block_info().time.plus_seconds(MIN_EXPIRY + 1),
//         finders_fee_bps: Some(0),
//     };
//     let res = router.execute_contract(
//         creator.clone(),
//         marketplace.clone(),
//         &set_ask,
//         &listing_funds(LISTING_FEE).unwrap(),
//     );
//     assert!(res.is_ok());

//     // test before bid is made
//     let query_bids_msg = QueryMsg::Bids {
//         collection: collection.to_string(),
//         token_id: TOKEN_ID,
//         start_after: None,
//         limit: None,
//     };
//     let res: BidsResponse = router
//         .wrap()
//         .query_wasm_smart(marketplace.clone(), &query_bids_msg)
//         .unwrap();
//     assert_eq!(res.bids, vec![]);

//     // Bidder makes bids
//     let set_bid_msg = ExecuteMsg::SetBid {
//         sale_type: SaleType::FixedPrice,
//         collection: collection.to_string(),
//         token_id: TOKEN_ID,
//         finders_fee_bps: None,
//         expires: router.block_info().time.plus_seconds(MIN_EXPIRY + 10),
//         finder: None,
//     };
//     let res = router.execute_contract(
//         bidder.clone(),
//         marketplace.clone(),
//         &set_bid_msg,
//         &coins(100, NATIVE_DENOM),
//     );
//     assert!(res.is_ok());

//     let set_bid_msg = ExecuteMsg::SetBid {
//         sale_type: SaleType::Auction,
//         collection: collection.to_string(),
//         token_id: TOKEN_ID + 1,
//         finders_fee_bps: None,
//         expires: router.block_info().time.plus_seconds(MIN_EXPIRY + 1),
//         finder: None,
//     };
//     let res = router.execute_contract(
//         bidder.clone(),
//         marketplace.clone(),
//         &set_bid_msg,
//         &coins(105, NATIVE_DENOM),
//     );
//     assert!(res.is_ok());

//     let res: BidsResponse = router
//         .wrap()
//         .query_wasm_smart(marketplace.clone(), &query_bids_msg)
//         .unwrap();
//     assert_eq!(res.bids[0].token_id, TOKEN_ID);
//     assert_eq!(res.bids[0].price.u128(), 100u128);
//     let query_bids_msg = QueryMsg::Bids {
//         collection: collection.to_string(),
//         token_id: TOKEN_ID + 1,
//         start_after: None,
//         limit: None,
//     };
//     let res: BidsResponse = router
//         .wrap()
//         .query_wasm_smart(marketplace.clone(), &query_bids_msg)
//         .unwrap();
//     assert_eq!(res.bids[0].token_id, TOKEN_ID + 1);
//     assert_eq!(res.bids[0].price.u128(), 105u128);

//     let query_bids_msg = QueryMsg::BidsByBidder {
//         bidder: bidder.to_string(),
//         start_after: Some(CollectionOffset::new(collection.to_string(), TOKEN_ID - 1)),
//         limit: None,
//     };
//     let res: BidsResponse = router
//         .wrap()
//         .query_wasm_smart(marketplace.clone(), &query_bids_msg)
//         .unwrap();
//     assert_eq!(res.bids.len(), 2);
//     let query_bids_msg = QueryMsg::BidsByBidderSortedByExpiration {
//         bidder: bidder.to_string(),
//         start_after: Some(CollectionOffset::new(collection.to_string(), TOKEN_ID - 1)),
//         limit: None,
//     };
//     let res: BidsResponse = router
//         .wrap()
//         .query_wasm_smart(marketplace, &query_bids_msg)
//         .unwrap();
//     assert_eq!(res.bids.len(), 2);
//     assert_eq!(
//         res.bids[0].expires_at.seconds(),
//         router
//             .block_info()
//             .time
//             .plus_seconds(MIN_EXPIRY + 1)
//             .seconds()
//     );
//     assert_eq!(
//         res.bids[1].expires_at.seconds(),
//         router
//             .block_info()
//             .time
//             .plus_seconds(MIN_EXPIRY + 10)
//             .seconds()
//     );
// }

// #[test]
// fn remove_bid_refund() {
//     let mut router = custom_mock_app();

//     // Setup intial accounts
//     let (_owner, bidder, creator) = setup_accounts(&mut router).unwrap();

//     // Instantiate and configure contracts
//     let (marketplace, collection) = setup_contracts(&mut router, &creator).unwrap();

//     // Add funds to creator for listing fees
//     add_funds_for_incremental_fee(&mut router, &creator, LISTING_FEE, 1u128).unwrap();

//     // Add funds to creator for listing fees
//     add_funds_for_incremental_fee(&mut router, &creator, LISTING_FEE, 1u128).unwrap();

//     // Mint NFT for creator
//     mint(&mut router, &creator, &collection, TOKEN_ID);
//     approve(&mut router, &creator, &collection, &marketplace, TOKEN_ID);

//     // An asking price is made by the creator
//     let set_ask = ExecuteMsg::SetAsk {
//         sale_type: SaleType::Auction,
//         collection: collection.to_string(),
//         token_id: TOKEN_ID,
//         price: coin(110, NATIVE_DENOM),
//         funds_recipient: None,
//         reserve_for: None,
//         expires: router.block_info().time.plus_seconds(MIN_EXPIRY + 1),
//         finders_fee_bps: Some(0),
//     };
//     let res = router.execute_contract(
//         creator.clone(),
//         marketplace.clone(),
//         &set_ask,
//         &listing_funds(LISTING_FEE).unwrap(),
//     );
//     assert!(res.is_ok());

//     // Bidder makes bid
//     let set_bid_msg = ExecuteMsg::SetBid {
//         sale_type: SaleType::FixedPrice,
//         collection: collection.to_string(),
//         token_id: TOKEN_ID,
//         finders_fee_bps: None,
//         expires: router.block_info().time.plus_seconds(MIN_EXPIRY + 1),
//         finder: None,
//     };

//     let res = router.execute_contract(
//         bidder.clone(),
//         marketplace.clone(),
//         &set_bid_msg,
//         &coins(100, NATIVE_DENOM),
//     );
//     assert!(res.is_ok());

//     // Bidder sent bid money
//     let bidder_native_balances = router.wrap().query_all_balances(bidder.clone()).unwrap();
//     assert_eq!(
//         bidder_native_balances,
//         coins(INITIAL_BALANCE - 100, NATIVE_DENOM)
//     );

//     // Contract has been paid
//     let contract_balances = router
//         .wrap()
//         .query_all_balances(marketplace.clone())
//         .unwrap();
//     assert_eq!(contract_balances, coins(100, NATIVE_DENOM));

//     // Bidder removes bid
//     let remove_bid_msg = ExecuteMsg::RemoveBid {
//         collection: collection.to_string(),
//         token_id: TOKEN_ID,
//     };
//     let res = router.execute_contract(bidder.clone(), marketplace, &remove_bid_msg, &[]);
//     assert!(res.is_ok());

//     // Bidder has money back
//     let bidder_native_balances = router.wrap().query_all_balances(bidder).unwrap();
//     assert_eq!(bidder_native_balances, coins(INITIAL_BALANCE, NATIVE_DENOM));
// }

// #[test]
// fn new_bid_refund() {
//     let mut router = custom_mock_app();

//     // Setup intial accounts
//     let (_owner, bidder, creator) = setup_accounts(&mut router).unwrap();

//     // Instantiate and configure contracts
//     let (marketplace, collection) = setup_contracts(&mut router, &creator).unwrap();

//     // Add funds to creator for listing fees
//     add_funds_for_incremental_fee(&mut router, &creator, LISTING_FEE, 1u128).unwrap();

//     // Mint NFT for creator
//     mint(&mut router, &creator, &collection, TOKEN_ID);
//     approve(&mut router, &creator, &collection, &marketplace, TOKEN_ID);

//     // An ask is made by the creator
//     let set_ask = ExecuteMsg::SetAsk {
//         sale_type: SaleType::Auction,
//         collection: collection.to_string(),
//         token_id: TOKEN_ID,
//         price: coin(200, NATIVE_DENOM),
//         funds_recipient: None,
//         reserve_for: None,
//         expires: router.block_info().time.plus_seconds(MIN_EXPIRY + 1),
//         finders_fee_bps: Some(0),
//     };
//     let res = router.execute_contract(
//         creator.clone(),
//         marketplace.clone(),
//         &set_ask,
//         &listing_funds(LISTING_FEE).unwrap(),
//     );
//     assert!(res.is_ok());

//     // Bidder makes bid
//     let set_bid_msg = ExecuteMsg::SetBid {
//         sale_type: SaleType::FixedPrice,
//         collection: collection.to_string(),
//         token_id: TOKEN_ID,
//         finders_fee_bps: None,
//         expires: router.block_info().time.plus_seconds(MIN_EXPIRY + 1),
//         finder: None,
//     };
//     let res = router.execute_contract(
//         bidder.clone(),
//         marketplace.clone(),
//         &set_bid_msg,
//         &coins(100, NATIVE_DENOM),
//     );
//     assert!(res.is_ok());

//     // Bidder sent bid money
//     let bidder_native_balances = router.wrap().query_all_balances(bidder.clone()).unwrap();
//     assert_eq!(
//         bidder_native_balances,
//         coins(INITIAL_BALANCE - 100, NATIVE_DENOM)
//     );

//     // Contract has been paid
//     let contract_balances = router
//         .wrap()
//         .query_all_balances(marketplace.clone())
//         .unwrap();
//     assert_eq!(contract_balances, coins(100, NATIVE_DENOM));

//     // Bidder makes higher bid
//     let set_bid_msg = ExecuteMsg::SetBid {
//         sale_type: SaleType::FixedPrice,
//         collection: collection.to_string(),
//         token_id: TOKEN_ID,
//         finders_fee_bps: None,
//         expires: router.block_info().time.plus_seconds(MIN_EXPIRY + 1),
//         finder: None,
//     };
//     let res = router.execute_contract(
//         bidder.clone(),
//         marketplace.clone(),
//         &set_bid_msg,
//         &coins(150, NATIVE_DENOM),
//     );
//     assert!(res.is_ok());

//     // Bidder has money back
//     let bidder_native_balances = router.wrap().query_all_balances(bidder.clone()).unwrap();
//     assert_eq!(
//         bidder_native_balances,
//         coins(INITIAL_BALANCE - 150, NATIVE_DENOM)
//     );

//     // Contract has been paid
//     let contract_balances = router
//         .wrap()
//         .query_all_balances(marketplace.clone())
//         .unwrap();
//     assert_eq!(contract_balances, coins(150, NATIVE_DENOM));

//     // Check new bid has been saved
//     let query_bid_msg = QueryMsg::Bid {
//         collection: collection.to_string(),
//         token_id: TOKEN_ID,
//         bidder: bidder.to_string(),
//     };
//     let bid = Bid {
//         collection,
//         token_id: TOKEN_ID,
//         bidder,
//         price: Uint128::from(150u128),
//         expires_at: (router.block_info().time.plus_seconds(MIN_EXPIRY + 1)),
//         finders_fee_bps: None,
//     };

//     let res: BidResponse = router
//         .wrap()
//         .query_wasm_smart(marketplace, &query_bid_msg)
//         .unwrap();
//     assert_eq!(res.bid, Some(bid));
// }

// #[test]
// fn try_sudo_update_params() {
//     let mut router = custom_mock_app();

//     // Setup intial accounts
//     let (_owner, _, creator) = setup_accounts(&mut router).unwrap();

//     // Instantiate and configure contracts
//     let (marketplace, _) = setup_contracts(&mut router, &creator).unwrap();

//     let update_params_msg = SudoMsg::UpdateParams {
//         trading_fee_bps: Some(5),
//         ask_expiry: Some(ExpiryRange::new(100, 2)),
//         bid_expiry: None,
//         operators: Some(vec!["operator".to_string()]),
//         max_finders_fee_bps: None,
//         min_price: Some(Uint128::from(5u128)),
//         stale_bid_duration: None,
//         bid_removal_reward_bps: None,
//         listing_fee: Some(Uint128::from(LISTING_FEE)),
//     };
//     router
//         .wasm_sudo(marketplace.clone(), &update_params_msg)
//         .unwrap_err();

//     let update_params_msg = SudoMsg::UpdateParams {
//         trading_fee_bps: Some(5),
//         ask_expiry: Some(ExpiryRange::new(1, 2)),
//         bid_expiry: Some(ExpiryRange::new(3, 4)),
//         operators: Some(vec!["operator".to_string()]),
//         max_finders_fee_bps: None,
//         min_price: Some(Uint128::from(5u128)),
//         stale_bid_duration: Some(10),
//         bid_removal_reward_bps: Some(20),
//         listing_fee: Some(Uint128::from(LISTING_FEE)),
//     };
//     let res = router.wasm_sudo(marketplace.clone(), &update_params_msg);
//     assert!(res.is_ok());

//     let query_params_msg = QueryMsg::Params {};
//     let res: ParamsResponse = router
//         .wrap()
//         .query_wasm_smart(marketplace.clone(), &query_params_msg)
//         .unwrap();
//     assert_eq!(res.params.trading_fee_percent, Decimal::percent(5));
//     assert_eq!(res.params.ask_expiry, ExpiryRange::new(1, 2));
//     assert_eq!(res.params.bid_expiry, ExpiryRange::new(3, 4));
//     assert_eq!(res.params.operators, vec!["operator1".to_string()]);
//     assert_eq!(res.params.min_price, Uint128::from(5u128));
//     assert_eq!(res.params.stale_bid_duration, Duration::Time(10));
//     assert_eq!(res.params.bid_removal_reward_percent, Decimal::percent(20));
//     assert_eq!(res.params.listing_fee, Uint128::from(LISTING_FEE));

//     let update_params_msg = SudoMsg::UpdateParams {
//         trading_fee_bps: None,
//         ask_expiry: None,
//         bid_expiry: None,
//         operators: Some(vec![
//             "operator3".to_string(),
//             "operator1".to_string(),
//             "operator2".to_string(),
//             "operator1".to_string(),
//             "operator4".to_string(),
//         ]),
//         max_finders_fee_bps: None,
//         min_price: None,
//         stale_bid_duration: None,
//         bid_removal_reward_bps: None,
//         listing_fee: None,
//     };
//     let res = router.wasm_sudo(marketplace.clone(), &update_params_msg);
//     assert!(res.is_ok());
//     // query params
//     let res: ParamsResponse = router
//         .wrap()
//         .query_wasm_smart(marketplace, &query_params_msg)
//         .unwrap();
//     assert_eq!(
//         res.params.operators,
//         vec![Addr::unchecked("operator1".to_string()),]
//     );
// }

// #[test]
// fn try_add_remove_sales_hooks() {
//     let mut router = custom_mock_app();
//     // Setup intial accounts
//     let (_owner, _, creator) = setup_accounts(&mut router).unwrap();
//     // Instantiate and configure contracts
//     let (marketplace, _) = setup_contracts(&mut router, &creator).unwrap();

//     let add_hook_msg = SudoMsg::AddSaleHook {
//         hook: "hook".to_string(),
//     };
//     let res = router.wasm_sudo(marketplace.clone(), &add_hook_msg);
//     assert!(res.is_ok());

//     let query_hooks_msg = QueryMsg::SaleHooks {};
//     let res: HooksResponse = router
//         .wrap()
//         .query_wasm_smart(marketplace.clone(), &query_hooks_msg)
//         .unwrap();
//     assert_eq!(res.hooks, vec!["hook".to_string()]);

//     let remove_hook_msg = SudoMsg::RemoveSaleHook {
//         hook: "hook".to_string(),
//     };
//     let res = router.wasm_sudo(marketplace.clone(), &remove_hook_msg);
//     assert!(res.is_ok());

//     let res: HooksResponse = router
//         .wrap()
//         .query_wasm_smart(marketplace, &query_hooks_msg)
//         .unwrap();
//     assert!(res.hooks.is_empty());
// }

// #[test]
// fn try_add_too_many_sales_hooks() {
//     let mut router = custom_mock_app();
//     // Setup intial accounts
//     let (_owner, _, creator) = setup_accounts(&mut router).unwrap();
//     // Instantiate and configure contracts
//     let (marketplace, _) = setup_contracts(&mut router, &creator).unwrap();

//     let add_hook_msg = SudoMsg::AddSaleHook {
//         hook: "hook1".to_string(),
//     };
//     let res = router.wasm_sudo(marketplace.clone(), &add_hook_msg);
//     assert!(res.is_ok());

//     let add_hook_msg = SudoMsg::AddSaleHook {
//         hook: "hook2".to_string(),
//     };
//     let res = router.wasm_sudo(marketplace.clone(), &add_hook_msg);
//     assert!(res.is_ok());

//     let add_hook_msg = SudoMsg::AddSaleHook {
//         hook: "hook3".to_string(),
//     };
//     let res = router.wasm_sudo(marketplace.clone(), &add_hook_msg);
//     assert!(res.is_ok());

//     let add_hook_msg = SudoMsg::AddSaleHook {
//         hook: "hook4".to_string(),
//     };
//     let res = router.wasm_sudo(marketplace.clone(), &add_hook_msg);
//     assert!(res.is_ok());

//     let add_hook_msg = SudoMsg::AddSaleHook {
//         hook: "hook5".to_string(),
//     };
//     let res = router.wasm_sudo(marketplace.clone(), &add_hook_msg);
//     assert!(res.is_ok());

//     let add_hook_msg = SudoMsg::AddSaleHook {
//         hook: "hook6".to_string(),
//     };
//     let res = router.wasm_sudo(marketplace, &add_hook_msg);
//     assert!(res.is_err());
// }

// #[test]
// fn try_add_remove_bid_hooks() {
//     let mut router = custom_mock_app();
//     // Setup intial accounts
//     let (_owner, _, creator) = setup_accounts(&mut router).unwrap();
//     // Instantiate and configure contracts
//     let (marketplace, _) = setup_contracts(&mut router, &creator).unwrap();

//     let add_bid_hook_msg = SudoMsg::AddBidHook {
//         hook: "bid_hook".to_string(),
//     };
//     let res = router.wasm_sudo(marketplace.clone(), &add_bid_hook_msg);
//     assert!(res.is_ok());

//     let query_hooks_msg = QueryMsg::BidHooks {};
//     let res: HooksResponse = router
//         .wrap()
//         .query_wasm_smart(marketplace.clone(), &query_hooks_msg)
//         .unwrap();
//     assert_eq!(res.hooks, vec!["bid_hook".to_string()]);

//     let remove_hook_msg = SudoMsg::RemoveBidHook {
//         hook: "bid_hook".to_string(),
//     };
//     let res = router.wasm_sudo(marketplace.clone(), &remove_hook_msg);
//     assert!(res.is_ok());

//     let res: HooksResponse = router
//         .wrap()
//         .query_wasm_smart(marketplace, &query_hooks_msg)
//         .unwrap();
//     assert!(res.hooks.is_empty());
// }

// #[test]
// fn try_init_hook() {
//     let mut router = custom_mock_app();
//     // Setup intial accounts
//     let (_owner, _, creator) = setup_accounts(&mut router).unwrap();
//     // Instantiate marketplace contract
//     let marketplace_id = router.store_code(contract_marketplace());
//     let msg = crate::msg::InstantiateMsg {
//         operators: vec!["operator1".to_string()],
//         trading_fee_bps: TRADING_FEE_BPS,
//         ask_expiry: ExpiryRange::new(MIN_EXPIRY, MAX_EXPIRY),
//         bid_expiry: ExpiryRange::new(MIN_EXPIRY, MAX_EXPIRY),
//         sale_hook: Some("hook".to_string()),
//         max_finders_fee_bps: MAX_FINDERS_FEE_BPS,
//         min_price: Uint128::from(5u128),
//         stale_bid_duration: Duration::Time(100),
//         bid_removal_reward_bps: BID_REMOVAL_REWARD_BPS,
//         listing_fee: Uint128::from(LISTING_FEE),
//     };
//     let marketplace = router
//         .instantiate_contract(marketplace_id, creator, &msg, &[], "Marketplace", None)
//         .unwrap();

//     let query_hooks_msg = QueryMsg::SaleHooks {};
//     let res: HooksResponse = router
//         .wrap()
//         .query_wasm_smart(marketplace.clone(), &query_hooks_msg)
//         .unwrap();
//     assert_eq!(res.hooks, vec!["hook".to_string()]);

//     let remove_hook_msg = SudoMsg::RemoveSaleHook {
//         hook: "hook".to_string(),
//     };
//     let res = router.wasm_sudo(marketplace.clone(), &remove_hook_msg);
//     assert!(res.is_ok());

//     let res: HooksResponse = router
//         .wrap()
//         .query_wasm_smart(marketplace, &query_hooks_msg)
//         .unwrap();
//     assert!(res.hooks.is_empty());
// }

// #[test]
// fn try_hook_was_run() {
//     let mut router = custom_mock_app();
//     // Setup intial accounts
//     let (_owner, bidder, creator) = setup_accounts(&mut router).unwrap();
//     // Instantiate and configure contracts
//     let (marketplace, collection) = setup_contracts(&mut router, &creator).unwrap();
//     // Add funds to creator for listing fees
//     add_funds_for_incremental_fee(&mut router, &creator, LISTING_FEE, 1u128).unwrap();

//     // Add sales hook
//     let add_hook_msg = SudoMsg::AddSaleHook {
//         hook: "hook".to_string(),
//     };
//     let _res = router.wasm_sudo(marketplace.clone(), &add_hook_msg);

//     // Add listed hook
//     let add_ask_hook_msg = SudoMsg::AddAskHook {
//         hook: "ask_created_hook".to_string(),
//     };
//     let _res = router.wasm_sudo(marketplace.clone(), &add_ask_hook_msg);

//     // Add bid created hook
//     let add_ask_hook_msg = SudoMsg::AddBidHook {
//         hook: "bid_created_hook".to_string(),
//     };
//     let _res = router.wasm_sudo(marketplace.clone(), &add_ask_hook_msg);

//     // Mint NFT for creator
//     mint(&mut router, &creator, &collection, TOKEN_ID);

//     // An ask is made by the creator, but fails because NFT is not authorized
//     let set_ask = ExecuteMsg::SetAsk {
//         sale_type: SaleType::FixedPrice,
//         collection: collection.to_string(),
//         token_id: TOKEN_ID,
//         price: coin(100, NATIVE_DENOM),
//         funds_recipient: None,
//         reserve_for: None,
//         expires: router.block_info().time.plus_seconds(MIN_EXPIRY + 1),
//         finders_fee_bps: Some(0),
//     };
//     // Creator Authorizes NFT
//     let approve_msg = Cw721ExecuteMsg::<Empty>::Approve {
//         spender: marketplace.to_string(),
//         token_id: TOKEN_ID.to_string(),
//         expires: None,
//     };
//     let res = router.execute_contract(creator.clone(), collection.clone(), &approve_msg, &[]);
//     assert!(res.is_ok());
//     // Now set_ask succeeds
//     let res = router.execute_contract(
//         creator.clone(),
//         marketplace.clone(),
//         &set_ask,
//         &listing_funds(LISTING_FEE).unwrap(),
//     );
//     assert!(res.is_ok());
//     assert_eq!(
//         "ask-hook-failed",
//         res.unwrap().events[3].attributes[1].value
//     );

//     // Bidder makes bid that meets the ask criteria
//     let set_bid_msg = ExecuteMsg::SetBid {
//         sale_type: SaleType::FixedPrice,
//         collection: collection.to_string(),
//         token_id: TOKEN_ID,
//         finders_fee_bps: None,
//         expires: router.block_info().time.plus_seconds(MIN_EXPIRY + 1),
//         finder: None,
//     };

//     // Bid succeeds even though the hook contract cannot be found
//     let res = router.execute_contract(
//         bidder.clone(),
//         marketplace,
//         &set_bid_msg,
//         &coins(100, NATIVE_DENOM),
//     );
//     assert!(res.is_ok());
//     assert_eq!(
//         "sale-hook-failed",
//         res.as_ref().unwrap().events[10].attributes[1].value
//     );

//     // NFT is still transferred despite a sale finalized hook failing
//     let query_owner_msg = Cw721QueryMsg::OwnerOf {
//         token_id: TOKEN_ID.to_string(),
//         include_expired: None,
//     };
//     let res: OwnerOfResponse = router
//         .wrap()
//         .query_wasm_smart(collection, &query_owner_msg)
//         .unwrap();
//     assert_eq!(res.owner, bidder.to_string());
// }

// #[test]
// fn try_add_remove_listed_hooks() {
//     let mut router = custom_mock_app();
//     // Setup intial accounts
//     let (_owner, _, creator) = setup_accounts(&mut router).unwrap();
//     // Instantiate and configure contracts
//     let (marketplace, _) = setup_contracts(&mut router, &creator).unwrap();

//     let add_hook_msg = SudoMsg::AddAskHook {
//         hook: "hook".to_string(),
//     };
//     let res = router.wasm_sudo(marketplace.clone(), &add_hook_msg);
//     assert!(res.is_ok());

//     let query_hooks_msg = QueryMsg::AskHooks {};
//     let res: HooksResponse = router
//         .wrap()
//         .query_wasm_smart(marketplace.clone(), &query_hooks_msg)
//         .unwrap();
//     assert_eq!(res.hooks, vec!["hook".to_string()]);

//     let remove_hook_msg = SudoMsg::RemoveAskHook {
//         hook: "hook".to_string(),
//     };
//     let res = router.wasm_sudo(marketplace.clone(), &remove_hook_msg);
//     assert!(res.is_ok());

//     let res: HooksResponse = router
//         .wrap()
//         .query_wasm_smart(marketplace, &query_hooks_msg)
//         .unwrap();
//     assert!(res.hooks.is_empty());
// }
