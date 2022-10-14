#[cfg(test)]
mod tests {
    use crate::msg::*;

    use cosmwasm_std::Addr;
    use cosmwasm_std::Coin;
    use cosmwasm_std::Empty;

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
        assert_eq!(res, true);

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
        let msg = ExecuteMsg::ProcessAddress {
            address: addrs[0].clone(),
        };
        let res = app.execute_contract(Addr::unchecked(CREATOR), wl_addr.clone(), &msg, &[]);
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
        // AddAddresses { addresses: Vec<String> },
        // RemoveAddresses { addresses: Vec<String> },
        // // Add message to increment mint count on whitelist map. if mint succeeds, map increment will also succeed.
        // ProcessAddress { address: String },
        // UpdatePerAddressLimit { limit: u32 },
        // Purge {},
    }
}
