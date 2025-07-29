use chrono::Duration;
use chrono::NaiveDate;
use std::sync::Arc;

use crate::domain::Error;
use crate::domain::interfaces::AvailableSlotsProvider;
use crate::domain::services::WorkingHoursPolicy;
use crate::usecases::FreeSlotDTO;

#[derive(Clone)]
pub struct FreeSlotsUseCase<const N: usize, WP>
where
    WP: WorkingHoursPolicy,
{
    duration: Duration,
    policy: WP,
    provider: Arc<dyn AvailableSlotsProvider<N>>,
}

impl<const N: usize, WP> FreeSlotsUseCase<N, WP>
where
    WP: WorkingHoursPolicy,
{
    pub fn new(
        duration: Duration,
        policy: WP,
        provider: Arc<dyn AvailableSlotsProvider<N>>,
    ) -> Self {
        Self {
            duration,
            policy,
            provider,
        }
    }

    pub async fn free_slots(&self, date: NaiveDate) -> Result<Vec<FreeSlotDTO>, Error> {
        let slots = self.policy.generate_slots(date, self.duration)?;
        let slots = self.provider.available_slots(slots).await?;
        Ok(slots.iter().map(|slot| slot.into()).collect())
    }
}
