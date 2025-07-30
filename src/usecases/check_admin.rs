use std::sync::Arc;
use crate::domain::Error;
use crate::domain::interfaces::AdminProvider;
use crate::domain::models::UserID;

#[derive(Clone)]
pub struct CheckAdminUseCase {
    provider: Arc<dyn AdminProvider>,
}

impl CheckAdminUseCase {
    pub fn new(
        provider: Arc<dyn AdminProvider>,
    ) -> Self {
        Self { provider }
    }
    
    pub async fn is_admin(&self, id: UserID) -> Result<bool, Error> {
        self.provider.is_admin(id).await
    }
}
