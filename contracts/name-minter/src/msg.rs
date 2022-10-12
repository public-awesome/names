use cosmwasm_schema::cw_serde;
use cosmwasm_std::Uint128;

#[cw_serde]
pub struct InstantiateMsg {
    pub collection_code_id: u64,
    pub marketplace_addr: String,
    pub min_name_length: u64,
    pub max_name_length: u64,
    pub base_price: Uint128,
}

#[cw_serde]
pub enum ExecuteMsg {
    MintAndList { name: String },
}

#[cw_serde]
pub enum SudoMsg {
    UpdateParams {
        min_name_length: u64,
        max_name_length: u64,
        base_price: Uint128,
    },
    UpdateNameCollection {
        collection: String,
    },
    UpdateNameMarketplace {
        marketplace: String,
    },
}

#[cw_serde]
pub enum QueryMsg {
    Config {},
    Params {},
}

// We define a custom struct for each query response
#[cw_serde]
pub struct ConfigResponse {
    pub collection_addr: String,
}
