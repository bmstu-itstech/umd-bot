use crate::domain::models::UserID;

pub type StdError = Box<dyn std::error::Error + Send + Sync>;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("invalid value: {0}")]
    InvalidValue(String),

    #[error("invalid interval: {0}")]
    InvalidInterval(String),

    #[error("max capacity={0} exceeded")]
    MaxCapacityExceeded(usize),

    #[error("student already exists: {0}")]
    UserAlreadyExists(UserID),

    #[error("student not found: {0}")]
    UserNotFound(UserID),

    #[error("user has not reserved slot: {0}")]
    UserNotReserved(UserID),

    #[error("slot not found")]
    SlotNotFoundError,

    #[error(transparent)]
    Other(#[from] StdError),
}
