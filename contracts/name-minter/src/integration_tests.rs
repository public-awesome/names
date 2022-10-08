use crate::msg::{ExecuteMsg, InstantiateMsg};
use cosmwasm_std::{coins, Addr, Uint128};
use cw_multi_test::{BankSudo, Contract, ContractWrapper, Executor, SudoMsg as CwSudoMsg};
use name_marketplace::msg::{
    AskResponse, ExecuteMsg as MarketplaceExecuteMsg, QueryMsg as MarketplaceQueryMsg,
};
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
const ADMIN: &str = "admin";
const NAME: &str = "bobo";
const TRADING_FEE_BPS: u64 = 200; // 2%

pub fn custom_mock_app() -> StargazeApp {
    StargazeApp::default()
}

// 1. Instantiate Name Marketplace
// 2. Instantiate Name Minter (which instantiates Name Collection)
// 3. Update Name Marketplace with Name Collection address
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
    println!("Marketplace: {}", marketplace);

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
    println!("Minter: {}", minter);

    let name_collection = "contract2";

    // 3. Update Name Marketplace with Name Collection address
    let msg = name_marketplace::msg::SudoMsg::UpdateNameCollection {
        collection: name_collection.to_string(),
    };
    let res = app.wasm_sudo(marketplace.clone(), &msg);
    assert!(res.is_ok());

    (app, marketplace, minter, Addr::unchecked(name_collection))
}

fn mint_and_list() -> (StargazeApp, Addr) {
    let (mut app, mkt, minter, _) = instantiate_contracts();

    let user = Addr::unchecked(USER);
    let four_letter_name_cost = 100000000 * 10;

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

    let name = "test";

    let msg = ExecuteMsg::MintAndList {
        name: name.to_string(),
    };
    let res = app.execute_contract(user, minter, &msg, &name_fee);
    assert!(res.is_ok());

    // check if name is listed in marketplace
    let res: AskResponse = app
        .wrap()
        .query_wasm_smart(
            mkt.clone(),
            &MarketplaceQueryMsg::Ask {
                token_id: name.to_string(),
            },
        )
        .unwrap();
    assert_eq!(res.ask.unwrap().token_id, name);

    (app, mkt)
}

mod execute {
    use cosmwasm_std::coins;
    use cw_multi_test::BankSudo;
    use name_marketplace::msg::BidResponse;
    use sg_std::NATIVE_DENOM;

    use super::*;

    #[test]
    fn mint() {
        mint_and_list();
    }

    #[test]
    fn bid() {
        let (mut app, mkt) = mint_and_list();

        let bidder = Addr::unchecked(BIDDER);

        // give bidder some funds
        let amount = coins(100000000, NATIVE_DENOM);
        app.sudo(CwSudoMsg::Bank({
            BankSudo::Mint {
                to_address: bidder.to_string(),
                amount: amount.clone(),
            }
        }))
        .map_err(|err| println!("{:?}", err))
        .ok();

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
        assert_eq!(bid.bidder, BIDDER.to_string());
    }
}
