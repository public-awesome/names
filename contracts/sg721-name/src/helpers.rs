use crate::msg::QueryMsg;
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{to_binary, Addr, QuerierWrapper, QueryRequest, StdResult, WasmQuery};
use sg_name::{TextRecord, NFT};

/// NameCollectionContract is a wrapper around Addr that provides a lot of helpers
#[cw_serde]
pub struct NameCollectionContract(pub Addr);

impl NameCollectionContract {
    pub fn addr(&self) -> Addr {
        self.0.clone()
    }

    pub fn image_nft(&self, querier: &QuerierWrapper, name: &str) -> StdResult<Option<NFT>> {
        let res: Option<NFT> = querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: self.addr().into(),
            msg: to_binary(&QueryMsg::ImageNFT {
                name: name.to_string(),
            })?,
        }))?;

        Ok(res)
    }

    pub fn text_records(&self, querier: &QuerierWrapper, name: &str) -> StdResult<Vec<TextRecord>> {
        let res: Vec<TextRecord> = querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: self.addr().into(),
            msg: to_binary(&QueryMsg::TextRecords {
                name: name.to_string(),
            })?,
        }))?;

        Ok(res)
    }

    pub fn is_twitter_verified(&self, querier: &QuerierWrapper, name: &str) -> StdResult<bool> {
        let records = self.text_records(querier, name)?;

        for record in records {
            if record.name == "twitter" {
                return Ok(record.verified.unwrap_or_default());
            }
        }

        Ok(false)
    }
}
