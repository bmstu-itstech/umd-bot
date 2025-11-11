use crate::domain::models::{Service, UserID};

pub type StdError = Box<dyn std::error::Error + Send + Sync>;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("invalid value: {0}")]
    InvalidValue(String),

    #[error("max capacity={0} exceeded")]
    MaxCapacityExceeded(usize),

    #[error("student not found: {0}")]
    UserNotFound(UserID),

    #[error("user has not reserved slot: {0}")]
    UserNotReserved(UserID),

    #[error("slot not found")]
    SlotNotFound,

    #[error("slot already reserved by user")]
    SlotAlreadyReserved(UserID),

    #[error("can not reserve slot for service {0:?}")]
    CanNotReserveSlot(Service),

    #[error(transparent)]
    Other(#[from] StdError),
}
