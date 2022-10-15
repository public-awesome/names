use cosmwasm_schema::write_api;

use cw721_base::Extension;
use sg721::InstantiateMsg;
use sg721_name::msg::{ExecuteMsg, QueryMsg};
use sg_name::Metadata;

fn main() {
    write_api! {
        instantiate: InstantiateMsg,
        execute: ExecuteMsg<Metadata<Extension>>,
        query: QueryMsg,
    }
}
