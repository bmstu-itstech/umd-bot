use std::sync::Arc;

use crate::domain::Error;
use crate::domain::interfaces::UserProvider;
use crate::domain::models::UserID;

#[derive(Clone)]
pub struct CheckRegisteredUseCase {
    provider: Arc<dyn UserProvider>,
}

impl CheckRegisteredUseCase {
    pub fn new(repos: Arc<dyn UserProvider>) -> Self {
        Self { provider: repos }
    }

    pub async fn is_registered(&self, id: UserID) -> Result<bool, Error> {
        match self.provider.user(id).await {
            Ok(_) => Ok(true),
            Err(Error::UserNotFound(_)) => Ok(false),
            Err(err) => Err(err),
        }
    }
}
