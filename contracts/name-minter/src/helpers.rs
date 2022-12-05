use cosmwasm_schema::cw_serde;
use cosmwasm_std::{to_binary, Addr, QuerierWrapper, QueryRequest, StdResult, WasmMsg, WasmQuery};
use sg_name_minter::{ParamsResponse, SudoParams};
use sg_std::CosmosMsg;

use crate::msg::{ExecuteMsg, QueryMsg};

/// NameMinterContract is a wrapper around Addr that provides a lot of helpers
#[cw_serde]
pub struct NameMinterContract(pub Addr);

impl NameMinterContract {
    pub fn addr(&self) -> Addr {
        self.0.clone()
    }

    pub fn call<T: Into<ExecuteMsg>>(&self, msg: T) -> StdResult<CosmosMsg> {
        let msg = to_binary(&msg.into())?;
        Ok(WasmMsg::Execute {
            contract_addr: self.addr().into(),
            msg,
            funds: vec![],
        }
        .into())
    }

    pub fn params(&self, querier: &QuerierWrapper) -> StdResult<SudoParams> {
        let res: ParamsResponse = querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: self.addr().into(),
            msg: to_binary(&QueryMsg::Params {})?,
        }))?;

        Ok(res.params)
    }
}
