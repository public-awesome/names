use crate::msg::InstantiateMsg;
use cosmwasm_std::{Addr, Uint128};
use cw_multi_test::{Contract, ContractWrapper, Executor, SudoMsg as CwSudoMsg};
use sg_multi_test::StargazeApp;
use sg_std::StargazeMsgWrapper;

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
const ADMIN: &str = "admin";
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

mod mint {
    use cosmwasm_std::{coin, coins};
    use cw721::Cw721ExecuteMsg;
    use cw_multi_test::BankSudo;
    use sg_std::NATIVE_DENOM;

    use super::*;
    use crate::msg::ExecuteMsg;

    #[test]
    fn mint() {
        let (mut app, mkt, minter, collection) = instantiate_contracts();

        let user = Addr::unchecked(USER);
        let four_letter_name_cost = 100000000 * 10;

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
        let res = app.execute_contract(user, minter, &msg, &name_fee).unwrap();
        println!("{:?}", res);
        // assert!(res.is_ok());
    }
}
