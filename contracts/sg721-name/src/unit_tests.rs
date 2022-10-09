use cosmwasm_std::testing::{mock_env, mock_info, MockApi, MockQuerier, MockStorage};
use cosmwasm_std::{
    from_slice, to_binary, ContractInfoResponse, ContractResult, Empty, OwnedDeps, Querier,
    QuerierResult, QueryRequest, SystemError, SystemResult, Timestamp, WasmQuery,
};
use cw721::Cw721Query;
use cw721_base::{Extension, MintMsg};
use sg721::{CollectionInfo, ExecuteMsg as Sg721ExecuteMsg, InstantiateMsg};
use sg721_base::ContractError::Unauthorized;
use sg_name::{Metadata, TextRecord};
use std::marker::PhantomData;

use crate::entry::{execute, instantiate};
use crate::{ContractError, ExecuteMsg};
pub type Sg721NameContract<'a> = sg721_base::Sg721Contract<'a, Metadata<Extension>>;
const CREATOR: &str = "creator";
const IMPOSTER: &str = "imposter";
const FRIEND: &str = "friend";

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
        explicit_content: false,
        trading_start_time: None,
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
        explicit_content: false,
        trading_start_time: None,
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

    let mint_msg = MintMsg::<Metadata<Extension>> {
        token_id: token_id.to_string(),
        owner: info.sender.to_string(),
        token_uri: None,
        extension: Metadata {
            bio: None,
            profile: None,
            records: vec![],
            extension: None,
        },
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

    // update bio
    // too long
    let long_bio = Some("a".repeat(600));
    let update_bio_msg = ExecuteMsg::UpdateBio {
        name: token_id.to_string(),
        bio: long_bio,
    };
    let err = execute(deps.as_mut(), mock_env(), info.clone(), update_bio_msg).unwrap_err();
    assert_eq!(err.to_string(), ContractError::BioTooLong {}.to_string());
    // passes
    let bio = Some("I am a test".to_string());
    let update_bio_msg = ExecuteMsg::UpdateBio {
        name: token_id.to_string(),
        bio: bio.clone(),
    };
    execute(deps.as_mut(), mock_env(), info.clone(), update_bio_msg).unwrap();
    let res = contract
        .parent
        .nft_info(deps.as_ref(), token_id.into())
        .unwrap();
    assert_eq!(res.extension.bio, bio);

    // add txt record
    let record = TextRecord {
        name: "test".to_string(),
        value: "test".to_string(),
        verified_at: Some(Timestamp::from_seconds(100)),
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
    assert_eq!(res.extension.records[0].verified_at, None);

    // add another txt record
    let record = TextRecord {
        name: "twitter".to_string(),
        value: "jackdorsey".to_string(),
        verified_at: None,
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
    assert_eq!(res.extension.records[0].verified_at, None);

    // add duplicate record RecordNameAlreadyExist
    let record = TextRecord {
        name: "test".to_string(),
        value: "testtesttest".to_string(),
        verified_at: Some(Timestamp::from_seconds(100)),
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
    execute(deps.as_mut(), mock_env(), info.clone(), rm_record_msg).unwrap();
    let res = contract
        .parent
        .nft_info(deps.as_ref(), token_id.into())
        .unwrap();
    assert_eq!(res.extension.records.len(), 1);

    // transfer to friend
    let transfer_msg = ExecuteMsg::TransferNft {
        recipient: FRIEND.to_string(),
        token_id: token_id.to_string(),
    };
    execute(deps.as_mut(), mock_env(), info, transfer_msg).unwrap();
    // confirm transfer resets all records and bio
    let res = contract
        .parent
        .nft_info(deps.as_ref(), token_id.into())
        .unwrap();
    assert_eq!(res.extension.records.len(), 0);
    assert_eq!(res.extension.bio, None);
    assert_eq!(res.extension.profile, None);
    // confirm friend is new owner
    let res = contract
        .parent
        .owner_of(deps.as_ref(), mock_env(), token_id.into(), false)
        .unwrap();
    assert_eq!(res.owner, FRIEND.to_string());
}

#[test]
fn update_profile() {
    unimplemented!("TODO");
    // stub mock nft collection to return OwnerOfResponse nft
}
