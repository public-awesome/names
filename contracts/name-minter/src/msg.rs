use cosmwasm_schema::cw_serde;

#[cw_serde]
pub struct InstantiateMsg {
    pub collection_code_id: u64,
    pub marketplace_addr: String,
}

#[cw_serde]
pub enum ExecuteMsg {
    MintAndList { name: String },
}

#[cw_serde]
pub enum QueryMsg {
    Config {},
}

// We define a custom struct for each query response
#[cw_serde]
pub struct ConfigResponse {
    pub collection_addr: String,
}
