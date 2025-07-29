use chrono::{Days, Duration, NaiveDate};
use std::ops::Add;
use std::sync::Arc;

use crate::domain::Error;
use crate::domain::interfaces::{HasAvailableSlotsProvider, UserProvider};
use crate::domain::models::{ClosedRange, Service, UserID};
use crate::domain::services::{DeadlinePolicy, WorkingHoursPolicy};

const MAX_DAYS_BEFORE_RESERVE: Days = Days::new(30);

#[derive(Clone)]
pub struct DaysWithFreeSlotsUseCase<const N: usize, DP, WP>
where
    DP: DeadlinePolicy,
    WP: WorkingHoursPolicy,
{
    duration: Duration,
    deadline_policy: DP,
    working_hours_policy: WP,
    user_provider: Arc<dyn UserProvider>,
    provider: Arc<dyn HasAvailableSlotsProvider<N>>,
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
        user_provider: Arc<dyn UserProvider>,
        provider: Arc<dyn HasAvailableSlotsProvider<N>>,
    ) -> Self {
        Self {
            duration,
            deadline_policy,
            working_hours_policy,
            user_provider,
            provider,
        }
    }

    pub async fn days_with_free_slots(
        &self,
        user_id: UserID,
        date: NaiveDate,
        service: Service,
    ) -> Result<Vec<NaiveDate>, Error> {
        let user = self.user_provider.user(user_id).await?;
        let end = if service.has_deadline() {
            date.add(self.deadline_policy.deadline(user.citizenship()))
        } else {
            date.add(MAX_DAYS_BEFORE_RESERVE)
        };

        let range = ClosedRange { start: date, end };

        let mut result = Vec::new();
        for date in range.into_iter() {
            let slots = self
                .working_hours_policy
                .generate_slots(date, self.duration)?;

            if self.provider.has_available_slots(&slots).await? {
                result.push(date);
            }
        }

        Ok(result)
    }
}
