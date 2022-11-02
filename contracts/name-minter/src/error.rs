use cosmwasm_std::{StdError, Timestamp};
use cw_controllers::AdminError;
use cw_utils::PaymentError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    PaymentError(#[from] PaymentError),

    #[error("{0}")]
    Admin(#[from] AdminError),

    #[error("Name Minter: Unauthorized")]
    Unauthorized {},

    #[error("MintingPaused")]
    MintingPaused {},

    #[error("NotWhitelisted")]
    NotWhitelisted {},

    #[error("Invalid reply ID")]
    InvalidReplyID {},

    #[error("Invalid name")]
    InvalidName {},

    #[error("Name too short")]
    NameTooShort {},

    #[error("Name too long")]
    NameTooLong {},

    #[error("Incorrect payment, got: {got}, expected {expected}")]
    IncorrectPayment { got: u128, expected: u128 },

    #[error("InvalidTradingStartTime {0} < {1}")]
    InvalidTradingStartTime(Timestamp, Timestamp),

    #[error("MintingNotStarted")]
    MintingNotStarted {},

    #[error("Reply error")]
    ReplyOnSuccess {},
}
