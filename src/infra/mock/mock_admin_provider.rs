use async_trait::async_trait;
use crate::domain::Error;
use crate::domain::interfaces::AdminProvider;
use crate::domain::models::UserID;

pub struct MockAdminProvider {
    admin_ids: Vec<UserID>,
}

impl MockAdminProvider {
    pub fn new(admin_ids: Vec<UserID>) -> Self {
        MockAdminProvider { admin_ids }
    }
}

#[async_trait]
impl AdminProvider for MockAdminProvider {
    async fn is_admin(&self, id: UserID) -> Result<bool, Error> {
        Ok(self.admin_ids.contains(&id))
    }
}
