use std::sync::{Arc, Mutex};
use chrono::Utc;

use crate::domain::Error;
use crate::domain::interfaces::UserProvider;
use crate::domain::models::TelegramID;
use crate::domain::services::DeadlinePolicy;


pub struct CheckDeadlineUseCase<D>
where
    D: DeadlinePolicy,
{
    deadline_policy: D,
    provider: Arc<Mutex<dyn UserProvider>>,
}

impl<D> CheckDeadlineUseCase<D>
where
    D: DeadlinePolicy,
{
    pub fn new(deadline_policy: D, provider: Arc<Mutex<dyn UserProvider>>) -> Self {
        Self { deadline_policy, provider }
    }
    
    pub async fn check_deadline(&self, user_id: i64) -> Result<bool, Error> {
        let provider = self.provider.lock().unwrap();
        let student = provider
            .user(TelegramID::new(user_id))
            .await?;
        
        let today = Utc::now().date_naive();
        let deadline = student
            .arrival_data()
            .checked_add_days(self.deadline_policy.deadline(student.citizenship()))
            .unwrap();
        
        Ok(deadline <= today)
    }
}
