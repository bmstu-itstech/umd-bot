use serde::{Deserialize, Serialize};
use teloxide::RequestError;
use teloxide::dispatching::dialogue::InMemStorageError;

use crate::domain::Error;

#[derive(Default, Clone, Serialize, Deserialize)]
pub enum DialogueState {
    #[default]
    Idle,
    RegistrationState,
}

pub type HandlerResult = Result<(), Error>;

impl From<RequestError> for Error {
    fn from(value: RequestError) -> Self {
        Self::Other(value.into())
    }
}

impl From<InMemStorageError> for Error {
    fn from(value: InMemStorageError) -> Self {
        Self::Other(value.into())
    }
}
