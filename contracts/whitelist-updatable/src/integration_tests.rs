#[cfg(test)]
mod tests {
    use crate::msg::*;

    use cosmwasm_std::Addr;
    use cosmwasm_std::Coin;
    use cosmwasm_std::Empty;
    use sg_std::StargazeMsgWrapper;

    use cw_multi_test::{App, AppBuilder, Contract, ContractWrapper, Executor};

    const CREATOR: &str = "creator";
    const OTHER_ADMIN: &str = "other_admin";

    pub fn wl_contract() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new(
            crate::contract::execute,
            crate::contract::instantiate,
            crate::contract::query,
        );
        Box::new(contract)
    }

    pub fn name_minter_contract() -> Box<dyn Contract<StargazeMsgWrapper>> {
        let contract = ContractWrapper::new(
            name_minter::contract::execute,
            name_minter::contract::instantiate,
            name_minter::query::query,
        );
        Box::new(contract)
    }

    fn mock_app(init_funds: &[Coin]) -> App {
        AppBuilder::new().build(|router, _, storage| {
            router
                .bank
                .init_balance(storage, &Addr::unchecked(CREATOR), init_funds.to_vec())
                .unwrap();
        })
    }

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
            per_address_limit: 10,
            addresses: addrs.clone(),
        };

        let mut app = mock_app(&[]);
        let wl_id = app.store_code(wl_contract());

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

        // process_address to increase mint count and check mint count incremented
        // execute_process_address
        // TODO fix process address
        // let msg = ExecuteMsg::ProcessAddress {
        //     address: addrs[0].clone(),
        // };
        // let res = app.execute_contract(Addr::unchecked(CREATOR), wl_addr.clone(), &msg, &[]);
        // assert!(res.is_ok());
        // let res: u32 = app
        //     .wrap()
        //     .query_wasm_smart(
        //         &wl_addr,
        //         &QueryMsg::MintCount {
        //             address: addrs[0].clone(),
        //         },
        //     )
        //     .unwrap();
        // assert_eq!(res, 1);
    }

    // TODO fix process address
    // #[test]
    // fn exec() {
    //     let addrs: Vec<String> = vec![
    //         "addr0001".to_string(),
    //         "addr0002".to_string(),
    //         "addr0003".to_string(),
    //         "addr0004".to_string(),
    //         "addr0005".to_string(),
    //     ];

    //     let msg = InstantiateMsg {
    //         per_address_limit: 10,
    //         addresses: addrs,
    //     };

    //     let mut app = mock_app(&[]);
    //     let wl_id = app.store_code(wl_contract());

    //     let wl_addr = app
    //         .instantiate_contract(
    //             wl_id,
    //             Addr::unchecked(CREATOR),
    //             &msg,
    //             &[],
    //             "wl-contract".to_string(),
    //             None,
    //         )
    //         .unwrap();

    //     let msg = ExecuteMsg::UpdateAdmin {
    //         new_admin: OTHER_ADMIN.to_string(),
    //     };
    //     let res = app.execute_contract(Addr::unchecked(CREATOR), wl_addr.clone(), &msg, &[]);
    //     assert!(res.is_ok());
    //     let res: String = app
    //         .wrap()
    //         .query_wasm_smart(&wl_addr, &QueryMsg::Admin {})
    //         .unwrap();
    //     assert_eq!(res, OTHER_ADMIN.to_string());

    //     // add addresses
    //     let msg = ExecuteMsg::AddAddresses {
    //         addresses: vec![
    //             "addr0001".to_string(),
    //             "addr0002".to_string(),
    //             "addr0003".to_string(),
    //             "addr0004".to_string(),
    //             "addr0006".to_string(),
    //         ],
    //     };
    //     let res = app.execute_contract(Addr::unchecked(OTHER_ADMIN), wl_addr.clone(), &msg, &[]);
    //     assert!(res.is_err());
    //     let res: bool = app
    //         .wrap()
    //         .query_wasm_smart(
    //             &wl_addr,
    //             &QueryMsg::IncludesAddress {
    //                 address: "addr0006".to_string(),
    //             },
    //         )
    //         .unwrap();
    //     assert!(!res);
    //     let res: u64 = app
    //         .wrap()
    //         .query_wasm_smart(&wl_addr, &QueryMsg::Count {})
    //         .unwrap();
    //     assert_eq!(res, 5);
    //     let msg = ExecuteMsg::AddAddresses {
    //         addresses: vec!["addr0007".to_string(), "addr0006".to_string()],
    //     };
    //     let res = app.execute_contract(Addr::unchecked(OTHER_ADMIN), wl_addr.clone(), &msg, &[]);
    //     assert!(res.is_ok());
    //     let res: bool = app
    //         .wrap()
    //         .query_wasm_smart(
    //             &wl_addr,
    //             &QueryMsg::IncludesAddress {
    //                 address: "addr0006".to_string(),
    //             },
    //         )
    //         .unwrap();
    //     assert!(res);
    //     let res: u64 = app
    //         .wrap()
    //         .query_wasm_smart(&wl_addr, &QueryMsg::Count {})
    //         .unwrap();
    //     assert_eq!(res, 7);

    //     // remove addresses
    //     let msg = ExecuteMsg::RemoveAddresses {
    //         addresses: vec![
    //             "addr0000".to_string(),
    //             "addr0001".to_string(),
    //             "addr0002".to_string(),
    //             "addr0003".to_string(),
    //             "addr0004".to_string(),
    //             "addr0006".to_string(),
    //         ],
    //     };
    //     let res = app.execute_contract(Addr::unchecked(OTHER_ADMIN), wl_addr.clone(), &msg, &[]);
    //     assert!(res.is_err());
    //     let msg = ExecuteMsg::RemoveAddresses {
    //         addresses: vec![
    //             "addr0001".to_string(),
    //             "addr0002".to_string(),
    //             "addr0003".to_string(),
    //             "addr0004".to_string(),
    //             "addr0006".to_string(),
    //         ],
    //     };
    //     let res = app.execute_contract(Addr::unchecked(OTHER_ADMIN), wl_addr.clone(), &msg, &[]);
    //     assert!(res.is_ok());
    //     let res: bool = app
    //         .wrap()
    //         .query_wasm_smart(
    //             &wl_addr,
    //             &QueryMsg::IncludesAddress {
    //                 address: "addr0006".to_string(),
    //             },
    //         )
    //         .unwrap();
    //     assert!(!res);
    //     let res: u64 = app
    //         .wrap()
    //         .query_wasm_smart(&wl_addr, &QueryMsg::Count {})
    //         .unwrap();
    //     assert_eq!(res, 2);

    //     // per address limit
    //     let msg = ExecuteMsg::UpdatePerAddressLimit { limit: 1 };
    //     let res = app.execute_contract(Addr::unchecked(OTHER_ADMIN), wl_addr.clone(), &msg, &[]);
    //     assert!(res.is_ok());
    //     let res: u32 = app
    //         .wrap()
    //         .query_wasm_smart(&wl_addr, &QueryMsg::PerAddressLimit {})
    //         .unwrap();
    //     assert_eq!(res, 1);
    //     // surpass limit
    //     let msg = ExecuteMsg::ProcessAddress {
    //         address: "addr0007".to_string(),
    //     };
    //     let res = app.execute_contract(Addr::unchecked(OTHER_ADMIN), wl_addr.clone(), &msg, &[]);
    //     assert!(res.is_ok());
    //     let msg = ExecuteMsg::ProcessAddress {
    //         address: "addr0007".to_string(),
    //     };
    //     let res = app.execute_contract(Addr::unchecked(OTHER_ADMIN), wl_addr.clone(), &msg, &[]);
    //     assert!(res.is_err());

    //     // purge
    //     let msg = ExecuteMsg::Purge {};
    //     let res = app.execute_contract(Addr::unchecked(OTHER_ADMIN), wl_addr.clone(), &msg, &[]);
    //     assert!(res.is_ok());
    //     let res: u32 = app
    //         .wrap()
    //         .query_wasm_smart(&wl_addr, &QueryMsg::Count {})
    //         .unwrap();
    //     assert_eq!(res, 0);
    //     // does not include addr0007
    //     let res: bool = app
    //         .wrap()
    //         .query_wasm_smart(
    //             &wl_addr,
    //             &QueryMsg::IncludesAddress {
    //                 address: "addr0007".to_string(),
    //             },
    //         )
    //         .unwrap();
    //     assert!(!res);
    // }

    // TODO test adding minter addr
    // TODO query config
}
