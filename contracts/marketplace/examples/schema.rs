use cosmwasm_schema::{export_schema, export_schema_with_title, remove_schemas, schema_for};
use name_marketplace::msg::{
    AskCountResponse, AskOffset, AskResponse, AsksResponse, BidOffset, BidResponse, BidsResponse,
    ConfigResponse, ExecuteMsg, InstantiateMsg, ParamsResponse, QueryMsg, SudoMsg,
};
use name_marketplace::MarketplaceContract;
use sg_controllers::HooksResponse;
use std::env::current_dir;
use std::fs::create_dir_all;

fn main() {
    let mut out_dir = current_dir().unwrap();
    out_dir.push("schema");
    create_dir_all(&out_dir).unwrap();
    remove_schemas(&out_dir).unwrap();

    export_schema(&schema_for!(MarketplaceContract), &out_dir);
    export_schema(&schema_for!(InstantiateMsg), &out_dir);
    export_schema(&schema_for!(ExecuteMsg), &out_dir);
    export_schema(&schema_for!(QueryMsg), &out_dir);
    export_schema(&schema_for!(SudoMsg), &out_dir);

    export_schema(&schema_for!(AskCountResponse), &out_dir);
    export_schema(&schema_for!(AskOffset), &out_dir);
    export_schema(&schema_for!(AskResponse), &out_dir);
    export_schema(&schema_for!(AsksResponse), &out_dir);
    export_schema(&schema_for!(AskCountResponse), &out_dir);
    export_schema(&schema_for!(BidOffset), &out_dir);
    export_schema(&schema_for!(BidResponse), &out_dir);
    export_schema(&schema_for!(BidsResponse), &out_dir);
    export_schema(&schema_for!(ParamsResponse), &out_dir);
    export_schema(&schema_for!(ConfigResponse), &out_dir);

    // cosmwasm-typescript-gen expects the query return type as QueryNameResponse
    // Here we map query resonses to the correct name
    export_schema_with_title(&schema_for!(AsksResponse), &out_dir, "ReverseAsksResponse");
    export_schema_with_title(&schema_for!(AsksResponse), &out_dir, "AsksBySellerResponse");
    export_schema_with_title(&schema_for!(AsksResponse), &out_dir, "RenewalQueueResponse");
    export_schema_with_title(&schema_for!(HooksResponse), &out_dir, "AskHooksResponse");

    export_schema_with_title(
        &schema_for!(BidsResponse),
        &out_dir,
        "BidsSortedByPriceResponse",
    );
    export_schema_with_title(
        &schema_for!(BidsResponse),
        &out_dir,
        "ReverseBidsSortedByPriceResponse",
    );
    export_schema_with_title(&schema_for!(BidsResponse), &out_dir, "BidsByBidderResponse");
    export_schema_with_title(&schema_for!(HooksResponse), &out_dir, "BidHooksResponse");

    export_schema_with_title(&schema_for!(HooksResponse), &out_dir, "SaleHooksResponse");
}
