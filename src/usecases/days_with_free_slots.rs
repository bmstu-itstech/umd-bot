use std::ops::Add;
use std::sync::{Arc, Mutex};
use chrono::{Duration, NaiveDate};

use crate::domain::Error;
use crate::domain::interfaces::{HasAvailableSlotsProvider, UserProvider};
use crate::domain::models::{ClosedRange, TelegramID};
use crate::domain::services::{DeadlinePolicy, WorkingHoursPolicy};


pub struct DaysWithFreeSlotsUseCase<const N: usize, DP, WP>
where
    DP: DeadlinePolicy,
    WP: WorkingHoursPolicy,
{
    duration: Duration,
    deadline_policy: DP,
    working_hours_policy: WP,
    user_provider: Arc<Mutex<dyn UserProvider>>,
    provider: Arc<Mutex<dyn HasAvailableSlotsProvider<N>>>,
}

impl<const N: usize, DP, WP> DaysWithFreeSlotsUseCase<N, DP, WP>
where
    DP: DeadlinePolicy,
    WP: WorkingHoursPolicy,
{
    pub fn new(
        duration: Duration,
        deadline_policy: DP,
        working_hours_policy: WP,
        user_provider: Arc<Mutex<dyn UserProvider>>,
        provider: Arc<Mutex<dyn HasAvailableSlotsProvider<N>>>,
    ) -> Self {
        Self { duration, deadline_policy, working_hours_policy, user_provider, provider }
    }
    
    pub async fn days_with_free_slots(
        &self, user_id: TelegramID, date: NaiveDate
    ) -> Result<Vec<NaiveDate>, Error> {
        let user_provider = self.user_provider.lock().unwrap();
        let provider = self.provider.lock().unwrap();
        
        let user = user_provider.user(user_id).await?;
        let deadline = date.add(self.deadline_policy.deadline(user.citizenship()));
        let range = ClosedRange{ start: date, end: deadline };
        
        let mut result = Vec::new();
        for date in range.into_iter() {
            let slots = self.working_hours_policy
                .generate_slots(date, self.duration)?;
            
            if provider.has_available_slots(&slots).await? {
                result.push(date);
            }
        }
        
        Ok(result)
    }
}
