use cosmwasm_schema::cw_serde;
use cosmwasm_std::{
    to_json_binary, Addr, QuerierWrapper, QueryRequest, StdResult, WasmMsg, WasmQuery,
};
use sg_std::CosmosMsg;

use crate::{
    msg::{ExecuteMsg, QueryMsg},
    state::Config,
};

/// WhitelistUpdatableContract is a wrapper around Addr that provides a lot of helpers
#[cw_serde]
pub struct WhitelistUpdatableFlatrateContract(pub Addr);

impl WhitelistUpdatableFlatrateContract {
    pub fn addr(&self) -> Addr {
        self.0.clone()
    }

    pub fn call<T: Into<ExecuteMsg>>(&self, msg: T) -> StdResult<CosmosMsg> {
        let msg = to_json_binary(&msg.into())?;
        Ok(WasmMsg::Execute {
            contract_addr: self.addr().into(),
            msg,
            funds: vec![],
        }
        .into())
    }

    pub fn process_address(&self, address: &str) -> StdResult<CosmosMsg> {
        self.call(ExecuteMsg::ProcessAddress {
            address: address.to_string(),
        })
    }

    pub fn includes(&self, querier: &QuerierWrapper, address: String) -> StdResult<bool> {
        let includes: bool = querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: self.addr().into(),
            msg: to_json_binary(&QueryMsg::IncludesAddress { address })?,
        }))?;
        Ok(includes)
    }

    pub fn config(&self, querier: &QuerierWrapper) -> StdResult<Config> {
        let res: Config = querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: self.addr().into(),
            msg: to_json_binary(&QueryMsg::Config {})?,
        }))?;

        Ok(res)
    }

    pub fn mint_discount_amount(&self, querier: &QuerierWrapper) -> StdResult<Option<u64>> {
        querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: self.addr().into(),
            msg: to_json_binary(&QueryMsg::MintDiscountAmount {})?,
        }))
    }
}
