use chrono::{Days, NaiveDate};
use std::ops::Add;
use std::sync::Arc;

use crate::domain::Error;
use crate::domain::interfaces::{HasAvailableSlotsProvider, UserProvider};
use crate::domain::models::{ClosedRange, Service, UserID};
use crate::domain::services::{DeadlinePolicy, SlotsFactory, WorkingHoursPolicy};

const MAX_DAYS_BEFORE_RESERVE: Days = Days::new(30);

#[derive(Clone)]
pub struct DaysWithFreeSlotsUseCase {
    factory: Arc<dyn SlotsFactory>,
    deadline_policy: Arc<dyn DeadlinePolicy>,
    working_hours_policy: Arc<dyn WorkingHoursPolicy>,
    user_provider: Arc<dyn UserProvider>,
    provider: Arc<dyn HasAvailableSlotsProvider>,
}

impl DaysWithFreeSlotsUseCase {
    pub fn new(
        factory: Arc<dyn SlotsFactory>,
        deadline_policy: Arc<dyn DeadlinePolicy>,
        working_hours_policy: Arc<dyn WorkingHoursPolicy>,
        user_provider: Arc<dyn UserProvider>,
        provider: Arc<dyn HasAvailableSlotsProvider>,
    ) -> Self {
        Self {
            factory,
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
                .factory
                .create_all(date, self.working_hours_policy.as_ref());

            if self.provider.has_available_slots(&slots).await? {
                result.push(date);
            }
        }

        Ok(result)
    }
}
