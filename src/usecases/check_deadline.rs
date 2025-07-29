use chrono::Utc;
use std::sync::Arc;

use crate::domain::Error;
use crate::domain::interfaces::UserProvider;
use crate::domain::models::{Service, UserID};
use crate::domain::services::DeadlinePolicy;

#[derive(Clone)]
pub struct CheckDeadlineUseCase<DP>
where
    DP: DeadlinePolicy + Send + Sync,
{
    deadline_policy: DP,
    provider: Arc<dyn UserProvider>,
}

impl<D> CheckDeadlineUseCase<D>
where
    D: DeadlinePolicy + Send + Sync,
{
    pub fn new(deadline_policy: D, provider: Arc<dyn UserProvider>) -> Self {
        Self {
            deadline_policy,
            provider,
        }
    }

    pub async fn check_deadline(&self, user_id: UserID, service: Service) -> Result<bool, Error> {
        if !service.has_deadline() {
            return Ok(true);
        }
        let user = self.provider.user(user_id).await?;
        let today = Utc::now().date_naive();
        let deadline = user
            .arrival_date()
            .checked_add_days(self.deadline_policy.deadline(user.citizenship()))
            .unwrap();
        Ok(deadline <= today)
    }
}
