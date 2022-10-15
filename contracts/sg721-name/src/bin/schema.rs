use cosmwasm_schema::write_api;

use sg721::InstantiateMsg;
use sg721_name::msg::{ExecuteMsg, QueryMsg};
use sg_name::Metadata;

fn main() {
    write_api! {
        instantiate: InstantiateMsg,
        execute: ExecuteMsg<Metadata>,
        query: QueryMsg,
    }
}
