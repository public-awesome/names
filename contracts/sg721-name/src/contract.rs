#[cfg(not(feature = "library"))]
use crate::state::NAME_MAP;
use cosmwasm_std::{entry_point, DepsMut, Env, MessageInfo};
use cw2::set_contract_version;

use cw_utils::nonpayable;
use sg721_base::ContractError;
use sg_std::Response;
