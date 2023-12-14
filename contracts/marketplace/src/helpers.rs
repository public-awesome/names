use crate::{
    msg::{ExecuteMsg, QueryMsg},
    state::{Ask, Bid},
    ContractError,
};
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{
    ensure, to_binary, Addr, DepsMut, Env, QuerierWrapper, QueryRequest, StdError, StdResult,
    WasmMsg, WasmQuery,
};
use sg_std::{CosmosMsg, Response};

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
        let res: Option<Bid> = querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: self.addr().into(),
            msg: to_binary(&QueryMsg::HighestBid {
                token_id: token_id.to_string(),
            })?,
        }))?;

        Ok(res)
    }

    // contract needs approval from nft owner before accepting bid
    pub fn accept_bid(
        &self,
        querier: &QuerierWrapper,
        token_id: &str,
        bidder: &str,
    ) -> StdResult<CosmosMsg> {
        let highest_bid: Option<Bid> = self.highest_bid(querier, token_id)?;
        let bid = highest_bid.ok_or_else(|| {
            StdError::generic_err(format!("No bid found for token_id {}", token_id))
        })?;

        if bid.bidder != bidder {
            return Err(StdError::generic_err(format!(
                "Bidder {} is not the highest bidder",
                bidder
            )));
        }

        self.call(ExecuteMsg::AcceptBid {
            token_id: token_id.to_string(),
            bidder: bidder.to_string(),
        })
    }
}

pub fn process_renewal(
    deps: DepsMut,
    env: &Env,
    ask: Ask,
    response: Response,
) -> Result<Response, ContractError> {
    ensure!(
        ask.renewal_time.seconds() <= env.block.time.seconds(),
        ContractError::CannotProcessFutureRenewal {}
    );

    Ok(Response::new())
}
