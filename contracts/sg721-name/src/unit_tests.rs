use cosmwasm_std::testing::{mock_env, mock_info, MockApi, MockQuerier, MockStorage};
use cosmwasm_std::{
    from_slice, to_binary, Addr, ContractInfoResponse, ContractResult, Empty, OwnedDeps, Querier,
    QuerierResult, QueryRequest, SystemError, SystemResult, WasmQuery,
};
use cw721::Cw721Query;
use cw721_base::MintMsg;
use sg721::{CollectionInfo, ExecuteMsg as Sg721ExecuteMsg, InstantiateMsg};
use sg721_base::ContractError::Unauthorized;
use sg_name::{Metadata, TextRecord, MAX_RECORD_COUNT, NFT};
use std::marker::PhantomData;

use crate::contract::transcode;
use crate::entry::{execute, instantiate};
use crate::{ContractError, ExecuteMsg};
pub type Sg721NameContract<'a> = sg721_base::Sg721Contract<'a, Metadata>;
const CREATOR: &str = "creator";
const IMPOSTER: &str = "imposter";
// const FRIEND: &str = "friend";

pub fn mock_deps() -> OwnedDeps<MockStorage, MockApi, CustomMockQuerier, Empty> {
    OwnedDeps {
        storage: MockStorage::default(),
        api: MockApi::default(),
        querier: CustomMockQuerier::new(MockQuerier::new(&[])),
        custom_query_type: PhantomData,
    }
}

pub struct CustomMockQuerier {
    base: MockQuerier,
}

impl Querier for CustomMockQuerier {
    fn raw_query(&self, bin_request: &[u8]) -> QuerierResult {
        let request: QueryRequest<Empty> = match from_slice(bin_request) {
            Ok(v) => v,
            Err(e) => {
                return SystemResult::Err(SystemError::InvalidRequest {
                    error: format!("Parsing query request: {}", e),
                    request: bin_request.into(),
                })
            }
        };

        self.handle_query(&request)
    }
}

impl CustomMockQuerier {
    pub fn handle_query(&self, request: &QueryRequest<Empty>) -> QuerierResult {
        match &request {
            QueryRequest::Wasm(WasmQuery::ContractInfo { contract_addr: _ }) => {
                let response = ContractInfoResponse::new(1, CREATOR);
                SystemResult::Ok(ContractResult::Ok(to_binary(&response).unwrap()))
            }
            _ => self.base.handle_query(request),
        }
    }

    pub fn new(base: MockQuerier<Empty>) -> Self {
        CustomMockQuerier { base }
    }
}

#[test]
fn init() {
    // instantiate sg-names collection
    let mut deps = mock_deps();
    let info = mock_info(CREATOR, &[]);

    let collection_info = CollectionInfo {
        creator: "bobo".to_string(),
        description: "bobo name da best".to_string(),
        image: "ipfs://something".to_string(),
        external_link: None,
        explicit_content: None,
        start_trading_time: None,
        royalty_info: None,
    };
    let init_msg = InstantiateMsg {
        name: "SG Names".to_string(),
        symbol: "NAME".to_string(),
        minter: CREATOR.to_string(),
        collection_info,
    };

    instantiate(deps.as_mut(), mock_env(), info, init_msg).unwrap();
}

#[test]
fn mint_and_update() {
    let contract = Sg721NameContract::default();
    // instantiate sg-names collection
    let mut deps = mock_deps();
    let info = mock_info(CREATOR, &[]);
    let collection_info = CollectionInfo {
        creator: "bobo".to_string(),
        description: "bobo name da best".to_string(),
        image: "ipfs://something".to_string(),
        external_link: None,
        explicit_content: None,
        start_trading_time: None,
        royalty_info: None,
    };
    let init_msg = InstantiateMsg {
        name: "SG Names".to_string(),
        symbol: "NAME".to_string(),
        minter: CREATOR.to_string(),
        collection_info,
    };

    instantiate(deps.as_mut(), mock_env(), info.clone(), init_msg).unwrap();

    // mint token
    let token_id = "Enterprise";

    let mint_msg = MintMsg::<Metadata> {
        token_id: token_id.to_string(),
        owner: info.sender.to_string(),
        token_uri: None,
        extension: Metadata::default(),
    };
    let exec_msg = Sg721ExecuteMsg::Mint(mint_msg.clone());
    contract
        .execute(deps.as_mut(), mock_env(), info.clone(), exec_msg)
        .unwrap();

    // check token contains correct metadata
    let res = contract
        .parent
        .nft_info(deps.as_ref(), token_id.into())
        .unwrap();
    assert_eq!(res.token_uri, mint_msg.token_uri);
    assert_eq!(res.extension, mint_msg.extension);

    // update metadata
    // update bio, profile, records
    let new_metadata = Metadata {
        image_nft: Some(NFT {
            collection: Addr::unchecked("contract"),
            token_id: "token_id".to_string(),
        }),
        profile_nft: None,
        records: vec![TextRecord {
            name: "key".to_string(),
            value: "value".to_string(),
        }],
    };
    let update_metadata_msg = ExecuteMsg::UpdateMetadata {
        name: token_id.to_string(),
        metadata: Some(new_metadata.clone()),
    };
    execute(deps.as_mut(), mock_env(), info.clone(), update_metadata_msg).unwrap();
    let res = contract
        .parent
        .nft_info(deps.as_ref(), token_id.into())
        .unwrap();
    assert_eq!(res.extension, new_metadata);

    // trigger too many records error
    let new_metadata = Metadata {
        image_nft: Some(NFT {
            collection: Addr::unchecked("contract"),
            token_id: "token_id".to_string(),
        }),
        profile_nft: None,
        records: vec![
            TextRecord {
                name: "key1".to_string(),
                value: "value".to_string(),
            },
            TextRecord {
                name: "key2".to_string(),
                value: "value".to_string(),
            },
            TextRecord {
                name: "key3".to_string(),
                value: "value".to_string(),
            },
            TextRecord {
                name: "key4".to_string(),
                value: "value".to_string(),
            },
            TextRecord {
                name: "key5".to_string(),
                value: "value".to_string(),
            },
            TextRecord {
                name: "key6".to_string(),
                value: "value".to_string(),
            },
            TextRecord {
                name: "key7".to_string(),
                value: "value".to_string(),
            },
            TextRecord {
                name: "key8".to_string(),
                value: "value".to_string(),
            },
            TextRecord {
                name: "key9".to_string(),
                value: "value".to_string(),
            },
            TextRecord {
                name: "key10".to_string(),
                value: "value".to_string(),
            },
            TextRecord {
                name: "key11".to_string(),
                value: "value".to_string(),
            },
        ],
    };
    let update_metadata_msg = ExecuteMsg::UpdateMetadata {
        name: token_id.to_string(),
        metadata: Some(new_metadata),
    };
    let res = execute(deps.as_mut(), mock_env(), info.clone(), update_metadata_msg);
    assert_eq!(
        res.unwrap_err().to_string(),
        ContractError::TooManyRecords {
            max: MAX_RECORD_COUNT
        }
        .to_string()
    );

    // reset metadata
    let update_metadata_msg = ExecuteMsg::UpdateMetadata {
        name: token_id.to_string(),
        metadata: None,
    };
    execute(deps.as_mut(), mock_env(), info.clone(), update_metadata_msg).unwrap();
    let res = contract
        .parent
        .nft_info(deps.as_ref(), token_id.into())
        .unwrap();
    assert_eq!(res.extension, Metadata::default());

    // add txt record
    let record = TextRecord {
        name: "test".to_string(),
        value: "test".to_string(),
    };
    let add_record_msg = ExecuteMsg::AddTextRecord {
        name: token_id.to_string(),
        record,
    };
    // unauthorized
    let err = execute(
        deps.as_mut(),
        mock_env(),
        mock_info(IMPOSTER, &[]),
        add_record_msg.clone(),
    )
    .unwrap_err();
    assert_eq!(
        err.to_string(),
        ContractError::Base(Unauthorized {}).to_string()
    );
    // passes
    execute(deps.as_mut(), mock_env(), info.clone(), add_record_msg).unwrap();
    let res = contract
        .parent
        .nft_info(deps.as_ref(), token_id.into())
        .unwrap();
    assert_eq!(res.extension.records.len(), 1);

    // add another txt record
    let record = TextRecord {
        name: "twitter".to_string(),
        value: "jackdorsey".to_string(),
    };
    let add_record_msg = ExecuteMsg::AddTextRecord {
        name: token_id.to_string(),
        record,
    };
    execute(deps.as_mut(), mock_env(), info.clone(), add_record_msg).unwrap();
    let res = contract
        .parent
        .nft_info(deps.as_ref(), token_id.into())
        .unwrap();
    assert_eq!(res.extension.records.len(), 2);

    // add duplicate record RecordNameAlreadyExist
    let record = TextRecord {
        name: "test".to_string(),
        value: "testtesttest".to_string(),
    };
    let add_record_msg = ExecuteMsg::AddTextRecord {
        name: token_id.to_string(),
        record: record.clone(),
    };
    let err = execute(deps.as_mut(), mock_env(), info.clone(), add_record_msg).unwrap_err();
    assert_eq!(
        err.to_string(),
        ContractError::RecordNameAlreadyExists {}.to_string()
    );

    // update txt record
    let update_record_msg = ExecuteMsg::UpdateTextRecord {
        name: token_id.to_string(),
        record: record.clone(),
    };
    execute(deps.as_mut(), mock_env(), info.clone(), update_record_msg).unwrap();
    let res = contract
        .parent
        .nft_info(deps.as_ref(), token_id.into())
        .unwrap();
    assert_eq!(res.extension.records.len(), 2);
    assert_eq!(res.extension.records[1].value, record.value);

    // rm txt record
    let rm_record_msg = ExecuteMsg::RemoveTextRecord {
        name: token_id.to_string(),
        record_name: record.name,
    };
    execute(deps.as_mut(), mock_env(), info, rm_record_msg).unwrap();
    let res = contract
        .parent
        .nft_info(deps.as_ref(), token_id.into())
        .unwrap();
    assert_eq!(res.extension.records.len(), 1);
}

#[test]
fn test_transcode() {
    let res = transcode("cosmos1y54exmx84cqtasvjnskf9f63djuuj68p7hqf47");
    assert_eq!(res.unwrap(), "stars1y54exmx84cqtasvjnskf9f63djuuj68p2th570");
}
