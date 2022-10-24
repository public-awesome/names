use crate::{
    msg::{BidResponse, ExecuteMsg, QueryMsg},
    state::Bid,
};
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{to_binary, Addr, QuerierWrapper, QueryRequest, StdResult, WasmMsg, WasmQuery};
use sg_std::CosmosMsg;

/// MarketplaceContract is a wrapper around Addr that provides a lot of helpers
#[cw_serde]
pub struct NameMarketplaceContract(pub Addr);

impl NameMarketplaceContract {
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

    pub fn remove_ask(&self, token_id: &str) -> StdResult<CosmosMsg> {
        self.call(ExecuteMsg::RemoveAsk {
            token_id: token_id.to_string(),
        })
    }

    pub fn highest_bid(&self, querier: &QuerierWrapper, token_id: &str) -> StdResult<Option<Bid>> {
        let res: BidResponse = querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: self.addr().into(),
            msg: to_binary(&QueryMsg::HighestBid {
                token_id: token_id.to_string(),
            })?,
        }))?;

        Ok(res.bid)
    }
}
