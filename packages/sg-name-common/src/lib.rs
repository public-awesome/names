use cosmwasm_std::{coins, Decimal, Uint128};
use sg1::fair_burn;
use sg_std::{create_fund_community_pool_msg, Response, SubMsg, NATIVE_DENOM};

pub fn charge_fees(res: &mut Response, fair_burn_percent: Decimal, fee: Uint128) {
    let fair_burn_amount = fee * fair_burn_percent;
    let community_pool_amount = fee - fair_burn_amount;

    fair_burn(fair_burn_amount.u128(), None, res);

    res.messages
        .push(SubMsg::new(create_fund_community_pool_msg(coins(
            community_pool_amount.u128(),
            NATIVE_DENOM,
        ))));
}

pub const SECONDS_PER_YEAR: u64 = 31536000;
