pub type StdError = Box<dyn std::error::Error + Send + Sync>;


#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("invalid value: {0}")]
    InvalidValue(String),

    #[error("invalid interval: {0}")]
    InvalidInterval(String),
    
    #[error("max capacity={0} exceeded")]
    MaxCapacityExceeded(usize),
    
    #[error(transparent)]
    Other(#[from] StdError),
}
