use std::sync::Arc;

use crate::domain::Error;
use crate::domain::interfaces::UserProvider;
use crate::domain::models::UserID;
use crate::usecases::UserDTO;

#[derive(Clone)]
pub struct GetUserUseCase {
    provider: Arc<dyn UserProvider>,
}

impl GetUserUseCase {
    pub fn new(repos: Arc<dyn UserProvider>) -> Self {
        Self { provider: repos }
    }

    pub async fn user(&self, id: UserID) -> Result<UserDTO, Error> {
        match self.provider.user(id).await {
            Ok(user) => Ok((&user).into()),
            Err(err) => Err(err),
        }
    }
}
