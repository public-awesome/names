use cosmwasm_std::StdError;
use cw_utils::PaymentError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    Payment(#[from] PaymentError),

    #[error("{0}")]
    Base(#[from] sg721_base::ContractError),

    #[error("NameNotFound")]
    NameNotFound {},

    #[error("AddressAlreadyMapped")]
    AddressAlreadyMapped {},

    #[error("BioTooLong")]
    BioTooLong {},

    #[error("RecordNameAlreadyExists")]
    RecordNameAlreadyExists {},

    #[error("RecordNameEmpty")]
    RecordNameEmpty {},

    #[error("RecordNameTooLong")]
    RecordNameTooLong {},

    #[error("RecordValueTooLong")]
    RecordValueTooLong {},

    #[error("Unauthorized: Not contract creator or admin")]
    UnauthorizedCreatorOrAdmin {},
}
