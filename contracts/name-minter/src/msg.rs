use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use sg_name::{TextRecord, NFT};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub collection_code_id: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    MintAndList { name: String },
    // TODO: update
    // only owner can update, make sure sender == owner
    UpdateBio { name: String, bio: Option<String> },
    UpdateProfile { name: String, profile: Option<NFT> },
    AddTextRecord { name: String, record: TextRecord },
    RemoveTextRecord { name: String, record_name: String },
    UpdateTextRecord { name: String, record: TextRecord },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Config {},
}

// We define a custom struct for each query response
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ConfigResponse {
    pub collection_addr: String,
}
