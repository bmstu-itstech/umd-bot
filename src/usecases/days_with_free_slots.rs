use crate::Duration;
use chrono::{Days, NaiveDate, Utc};
use std::ops::Add;
use std::sync::Arc;

use crate::domain::Error;
use crate::domain::interfaces::{HasAvailableSlotsProvider, UserProvider};
use crate::domain::models::{ClosedRange, Service, UserID};
use crate::domain::services::{DeadlinePolicy, ServicePolicy, SlotsFactory, WorkingHoursPolicy};

const MAX_DAYS_BEFORE_RESERVE: Days = Days::new(14);

#[derive(Clone)]
pub struct DaysWithFreeSlotsUseCase {
    factory: Arc<dyn SlotsFactory>,
    deadline_policy: Arc<dyn DeadlinePolicy>,
    working_hours_policy: Arc<dyn WorkingHoursPolicy>,
    user_provider: Arc<dyn UserProvider>,
    provider: Arc<dyn HasAvailableSlotsProvider>,
    service_policy: Arc<dyn ServicePolicy>,
}

impl DaysWithFreeSlotsUseCase {
    pub fn new(
        factory: Arc<dyn SlotsFactory>,
        deadline_policy: Arc<dyn DeadlinePolicy>,
        working_hours_policy: Arc<dyn WorkingHoursPolicy>,
        user_provider: Arc<dyn UserProvider>,
        provider: Arc<dyn HasAvailableSlotsProvider>,
        service_policy: Arc<dyn ServicePolicy>,
    ) -> Self {
        Self {
            factory,
            deadline_policy,
            working_hours_policy,
            user_provider,
            provider,
            service_policy,
        }
    }

    pub async fn days_with_free_slots(
        &self,
        user_id: UserID,
        service: Service,
    ) -> Result<Vec<NaiveDate>, Error> {
        let user = self.user_provider.user(user_id).await?;
        let start = Utc::now().naive_utc().date().add(Duration::days(1));
        let end = if service.has_deadline() {
            start.add(self.deadline_policy.deadline(user.citizenship()))
        } else {
            start.add(MAX_DAYS_BEFORE_RESERVE)
        };

        let range = ClosedRange { start, end };

        let mut result = Vec::new();
        for date in range.into_iter() {
            let slots = self
                .factory
                .create_all(
                    date,
                    self.working_hours_policy.as_ref(),
                    self.service_policy.as_ref(),
                )?
                .into_iter()
                .filter(|slot| slot.can_be_served_with(service))
                .collect::<Vec<_>>();

            if !slots.is_empty() && self.provider.has_available_slots(&slots).await? {
                result.push(date);
            }
        }

        Ok(result)
    }
}
