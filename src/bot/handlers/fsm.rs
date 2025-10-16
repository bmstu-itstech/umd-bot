use teloxide::RequestError;
use teloxide::dispatching::dialogue::InMemStorageError;

use crate::domain::Error;

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
