#[cfg(test)]
mod tests {
    use crate::msg::*;

    use cosmwasm_std::Addr;
    use name_minter::msg::InstantiateMsg as NameMinterInstantiateMsg;
    use sg_std::StargazeMsgWrapper;

    use cw_multi_test::{Contract, ContractWrapper, Executor};

    use sg_multi_test::StargazeApp;

    const CREATOR: &str = "creator";
    const OTHER_ADMIN: &str = "other_admin";
    const PER_ADDRESS_LIMIT: u32 = 10;

    fn custom_mock_app() -> StargazeApp {
        StargazeApp::default()
    }

    pub fn wl_contract() -> Box<dyn Contract<StargazeMsgWrapper>> {
        let contract = ContractWrapper::new_with_empty(
            crate::contract::execute,
            crate::contract::instantiate,
            crate::contract::query,
        );
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

    pub fn name_minter_contract() -> Box<dyn Contract<StargazeMsgWrapper>> {
        let contract = ContractWrapper::new(
            name_minter::contract::execute,
            name_minter::contract::instantiate,
            name_minter::query::query,
        )
        .with_reply(name_minter::contract::reply)
        .with_sudo(name_minter::sudo::sudo);
        Box::new(contract)
    }

    // pub fn mock_params() ->

    #[test]
    pub fn init() {
        let addrs: Vec<String> = vec![
            "addr0001".to_string(),
            "addr0002".to_string(),
            "addr0003".to_string(),
            "addr0004".to_string(),
            "addr0005".to_string(),
        ];

        let msg = InstantiateMsg {
            per_address_limit: PER_ADDRESS_LIMIT,
            addresses: addrs.clone(),
            mint_discount_bps: None,
        };

        let mut app = custom_mock_app();
        let wl_id = app.store_code(wl_contract());
        let sg721_id = app.store_code(contract_collection());
        let minter_id = app.store_code(name_minter_contract());

        let wl_addr = app
            .instantiate_contract(
                wl_id,
                Addr::unchecked(CREATOR),
                &msg,
                &[],
                "wl-contract".to_string(),
                None,
            )
            .unwrap();

        let msg = NameMinterInstantiateMsg {
            admin: Some(CREATOR.to_string()),
            collection_code_id: sg721_id,
            marketplace_addr: "marketplace".to_string(),
            base_price: 100u128.into(),
            min_name_length: 3,
            max_name_length: 63,
            whitelists: vec![],
        };

        let minter_addr = app
            .instantiate_contract(
                minter_id,
                Addr::unchecked(CREATOR),
                &msg,
                &[],
                "name-minter-contract".to_string(),
                None,
            )
            .unwrap();

        let res: String = app
            .wrap()
            .query_wasm_smart(&wl_addr, &QueryMsg::Admin {})
            .unwrap();
        assert_eq!(res, CREATOR.to_string());

        let res: u64 = app
            .wrap()
            .query_wasm_smart(&wl_addr, &QueryMsg::Count {})
            .unwrap();
        assert_eq!(res, addrs.len() as u64);

        let res: bool = app
            .wrap()
            .query_wasm_smart(
                &wl_addr,
                &QueryMsg::IncludesAddress {
                    address: addrs[0].clone(),
                },
            )
            .unwrap();
        assert!(res);

        let res: u32 = app
            .wrap()
            .query_wasm_smart(
                &wl_addr,
                &QueryMsg::MintCount {
                    address: addrs[0].clone(),
                },
            )
            .unwrap();
        assert_eq!(res, 0);

        let res: u32 = app
            .wrap()
            .query_wasm_smart(&wl_addr, &QueryMsg::PerAddressLimit {})
            .unwrap();
        assert_eq!(res, 10);

        // set minter_addr in whitelist
        let msg = ExecuteMsg::UpdateMinterContract {
            minter_contract: minter_addr.to_string(),
        };
        let res = app.execute_contract(Addr::unchecked(CREATOR), wl_addr.clone(), &msg, &[]);
        assert!(res.is_ok());
        // process_address to increase mint count and check mint count incremented
        // execute_process_address
        let msg = ExecuteMsg::ProcessAddress {
            address: addrs[0].clone(),
        };
        let res = app.execute_contract(Addr::unchecked(minter_addr), wl_addr.clone(), &msg, &[]);
        assert!(res.is_ok());
        let res: u32 = app
            .wrap()
            .query_wasm_smart(
                &wl_addr,
                &QueryMsg::MintCount {
                    address: addrs[0].clone(),
                },
            )
            .unwrap();
        assert_eq!(res, 1);
    }

    #[test]
    fn exec() {
        let addrs: Vec<String> = vec![
            "addr0001".to_string(),
            "addr0002".to_string(),
            "addr0003".to_string(),
            "addr0004".to_string(),
            "addr0005".to_string(),
        ];

        let msg = InstantiateMsg {
            per_address_limit: 10,
            addresses: addrs,
            mint_discount_bps: None,
        };

        let mut app = custom_mock_app();
        let wl_id = app.store_code(wl_contract());
        let sg721_id = app.store_code(contract_collection());
        let minter_id = app.store_code(name_minter_contract());

        let wl_addr = app
            .instantiate_contract(
                wl_id,
                Addr::unchecked(CREATOR),
                &msg,
                &[],
                "wl-contract".to_string(),
                None,
            )
            .unwrap();

        let msg = NameMinterInstantiateMsg {
            admin: Some(CREATOR.to_string()),
            collection_code_id: sg721_id,
            marketplace_addr: "marketplace".to_string(),
            base_price: 100u128.into(),
            min_name_length: 3,
            max_name_length: 63,
            whitelists: vec![],
        };

        let minter_addr = app
            .instantiate_contract(
                minter_id,
                Addr::unchecked(CREATOR),
                &msg,
                &[],
                "name-minter-contract".to_string(),
                None,
            )
            .unwrap();

        let msg = ExecuteMsg::UpdateAdmin {
            new_admin: OTHER_ADMIN.to_string(),
        };
        let res = app.execute_contract(Addr::unchecked(CREATOR), wl_addr.clone(), &msg, &[]);
        assert!(res.is_ok());
        let res: String = app
            .wrap()
            .query_wasm_smart(&wl_addr, &QueryMsg::Admin {})
            .unwrap();
        assert_eq!(res, OTHER_ADMIN.to_string());

        // add addresses
        let msg = ExecuteMsg::AddAddresses {
            addresses: vec![
                "addr0001".to_string(),
                "addr0002".to_string(),
                "addr0003".to_string(),
                "addr0004".to_string(),
                "addr0006".to_string(),
            ],
        };
        let res = app.execute_contract(Addr::unchecked(OTHER_ADMIN), wl_addr.clone(), &msg, &[]);
        assert!(res.is_err());
        let res: bool = app
            .wrap()
            .query_wasm_smart(
                &wl_addr,
                &QueryMsg::IncludesAddress {
                    address: "addr0006".to_string(),
                },
            )
            .unwrap();
        assert!(!res);
        let res: u64 = app
            .wrap()
            .query_wasm_smart(&wl_addr, &QueryMsg::Count {})
            .unwrap();
        assert_eq!(res, 5);
        let msg = ExecuteMsg::AddAddresses {
            addresses: vec!["addr0007".to_string(), "addr0006".to_string()],
        };
        let res = app.execute_contract(Addr::unchecked(OTHER_ADMIN), wl_addr.clone(), &msg, &[]);
        assert!(res.is_ok());
        let res: bool = app
            .wrap()
            .query_wasm_smart(
                &wl_addr,
                &QueryMsg::IncludesAddress {
                    address: "addr0006".to_string(),
                },
            )
            .unwrap();
        assert!(res);
        let res: u64 = app
            .wrap()
            .query_wasm_smart(&wl_addr, &QueryMsg::Count {})
            .unwrap();
        assert_eq!(res, 7);

        // remove addresses
        let msg = ExecuteMsg::RemoveAddresses {
            addresses: vec![
                "addr0000".to_string(),
                "addr0001".to_string(),
                "addr0002".to_string(),
                "addr0003".to_string(),
                "addr0004".to_string(),
                "addr0006".to_string(),
            ],
        };
        let res = app.execute_contract(Addr::unchecked(OTHER_ADMIN), wl_addr.clone(), &msg, &[]);
        assert!(res.is_err());
        let msg = ExecuteMsg::RemoveAddresses {
            addresses: vec![
                "addr0001".to_string(),
                "addr0002".to_string(),
                "addr0003".to_string(),
                "addr0004".to_string(),
                "addr0006".to_string(),
            ],
        };
        let res = app.execute_contract(Addr::unchecked(OTHER_ADMIN), wl_addr.clone(), &msg, &[]);
        assert!(res.is_ok());
        let res: bool = app
            .wrap()
            .query_wasm_smart(
                &wl_addr,
                &QueryMsg::IncludesAddress {
                    address: "addr0006".to_string(),
                },
            )
            .unwrap();
        assert!(!res);
        let res: u64 = app
            .wrap()
            .query_wasm_smart(&wl_addr, &QueryMsg::Count {})
            .unwrap();
        assert_eq!(res, 2);

        // per address limit
        let new_per_address_limit = 1;
        let msg = ExecuteMsg::UpdatePerAddressLimit {
            limit: new_per_address_limit,
        };
        let res = app.execute_contract(Addr::unchecked(OTHER_ADMIN), wl_addr.clone(), &msg, &[]);
        assert!(res.is_ok());
        let res: u32 = app
            .wrap()
            .query_wasm_smart(&wl_addr, &QueryMsg::PerAddressLimit {})
            .unwrap();
        assert_eq!(res, 1);

        // set minter_addr in whitelist
        let msg = ExecuteMsg::UpdateMinterContract {
            minter_contract: minter_addr.to_string(),
        };
        let res = app.execute_contract(Addr::unchecked(OTHER_ADMIN), wl_addr.clone(), &msg, &[]);
        assert!(res.is_ok());

        // surpass limit
        let res: bool = app
            .wrap()
            .query_wasm_smart(
                &wl_addr,
                &QueryMsg::IsProcessable {
                    address: "addr0007".to_string(),
                },
            )
            .unwrap();
        assert!(res);
        let msg = ExecuteMsg::ProcessAddress {
            address: "addr0007".to_string(),
        };
        let res = app.execute_contract(
            Addr::unchecked(minter_addr.clone()),
            wl_addr.clone(),
            &msg,
            &[],
        );
        assert!(res.is_ok());
        let res: bool = app
            .wrap()
            .query_wasm_smart(
                &wl_addr,
                &QueryMsg::IsProcessable {
                    address: "addr0007".to_string(),
                },
            )
            .unwrap();
        assert!(!res);
        let msg = ExecuteMsg::ProcessAddress {
            address: "addr0007".to_string(),
        };
        let res = app.execute_contract(
            Addr::unchecked(minter_addr.clone()),
            wl_addr.clone(),
            &msg,
            &[],
        );
        assert!(res.is_err());

        // purge
        let msg = ExecuteMsg::Purge {};
        let res = app.execute_contract(Addr::unchecked(OTHER_ADMIN), wl_addr.clone(), &msg, &[]);
        assert!(res.is_ok());
        let res: u32 = app
            .wrap()
            .query_wasm_smart(&wl_addr, &QueryMsg::Count {})
            .unwrap();
        assert_eq!(res, 0);
        // does not include addr0007
        let res: bool = app
            .wrap()
            .query_wasm_smart(
                &wl_addr,
                &QueryMsg::IncludesAddress {
                    address: "addr0007".to_string(),
                },
            )
            .unwrap();
        assert!(!res);

        // query config
        let res: ConfigResponse = app
            .wrap()
            .query_wasm_smart(&wl_addr, &QueryMsg::Config {})
            .unwrap();
        assert_eq!(res.config.admin, Addr::unchecked(OTHER_ADMIN).to_string());
        assert_eq!(res.config.minter_contract, Some(minter_addr));
        assert_eq!(res.config.per_address_limit, new_per_address_limit);
    }
}
