use cosmwasm_std::testing::{mock_env, mock_info, MockApi, MockQuerier, MockStorage};
use cosmwasm_std::{
    from_slice, to_binary, ContractInfoResponse, ContractResult, Empty, OwnedDeps, Querier,
    QuerierResult, QueryRequest, SystemError, SystemResult, WasmQuery,
};
use cw721::Cw721Query;
use cw721_base::Extension;
use sg721::{CollectionInfo, ExecuteMsg as Sg721ExecuteMsg, InstantiateMsg, MintMsg};
use sg_name::Metadata;
use std::marker::PhantomData;

use crate::entry::{execute, instantiate};
use crate::{ContractError, ExecuteMsg};
pub type Sg721NameContract<'a> = sg721_base::Sg721Contract<'a, Metadata<Extension>>;
const CREATOR: &str = "creator";

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
        collection_info: collection_info,
    };

    instantiate(deps.as_mut(), mock_env(), info.clone(), init_msg).unwrap();
}

#[test]
fn mint() {
    let contract = Sg721NameContract::default();
    let info = mock_info(CREATOR, &[]);
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
        collection_info: collection_info,
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
    let bio = Some("I am a test".to_string());
    let update_bio_msg = ExecuteMsg::UpdateBio {
        name: token_id.to_string(),
        bio: bio.clone(),
    };
    execute(deps.as_mut(), mock_env(), info, update_bio_msg).unwrap();
    let res = contract
        .parent
        .nft_info(deps.as_ref(), token_id.into())
        .unwrap();
    assert_eq!(res.extension.bio, bio);
}

fn update_profile() {
    // stub mock nft collection to return OwnerOfResponse nft

    // instantiate normal sg721 nft collection
    // let init_msg = InstantiateMsg {
    //     name: "SpaceShips".to_string(),
    //     symbol: "SPACE".to_string(),
    //     minter: CREATOR.to_string(),
    //     collection_info: CollectionInfo {
    //         creator: CREATOR.to_string(),
    //         description: "this is a test".to_string(),
    //         image: "https://larry.engineer".to_string(),
    //         external_link: None,
    //         explicit_content: false,
    //         trading_start_time: None,
    //         royalty_info: None,
    //     },
    // };

    // create names collection
    // add nft profile to names collection
    let mut deps = mock_deps();
    let contract = Sg721NameContract::default();

    // instantiate contract
    let info = mock_info(CREATOR, &[]);
    let init_msg = InstantiateMsg {
        name: "SpaceShips".to_string(),
        symbol: "SPACE".to_string(),
        minter: CREATOR.to_string(),
        collection_info: CollectionInfo {
            creator: CREATOR.to_string(),
            description: "this is a test".to_string(),
            image: "https://larry.engineer".to_string(),
            external_link: None,
            explicit_content: false,
            trading_start_time: None,
            royalty_info: None,
        },
    };
    contract
        .instantiate(deps.as_mut(), mock_env(), info.clone(), init_msg)
        .unwrap();

    // mint token
    let token_id = "Enterprise";
    let mint_msg = MintMsg {
        token_id: token_id.to_string(),
        owner: "john".to_string(),
        token_uri: Some("https://starships.example.com/Starship/Enterprise.json".into()),
        extension: Metadata {
            bio: Some("This is the USS Enterprise NCC-1701".to_string()),
            profile: None,
            records: vec![],
            extension: None,
        },
    };
    let exec_msg = Sg721ExecuteMsg::Mint(mint_msg.clone());
    contract
        .execute(deps.as_mut(), mock_env(), info, exec_msg)
        .unwrap();
}

// add txt record

// rm txt record

// update txt record
