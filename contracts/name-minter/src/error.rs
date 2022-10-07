use cosmwasm_std::StdError;
use cw_utils::PaymentError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    PaymentError(#[from] PaymentError),

    #[error("Name Minter: Unauthorized")]
    Unauthorized {},

    #[error("Invalid reply ID")]
    InvalidReplyID {},

    #[error("Invalid name")]
    InvalidName {},

    #[error("Name too short")]
    NameTooShort {},

    #[error("Name too long")]
    NameTooLong {},

    #[error("Incorrect payment amount")]
    IncorrectPayment {},

    #[error("Reply error")]
    ReplyOnSuccess {},
}
