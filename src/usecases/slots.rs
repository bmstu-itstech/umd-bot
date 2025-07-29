use chrono::{Duration, NaiveDate};
use std::sync::Arc;

use crate::domain::Error;
use crate::domain::interfaces::ReservedSlotsProvider;
use crate::domain::services::WorkingHoursPolicy;
use crate::usecases::SlotDTO;

#[derive(Clone)]
pub struct SlotsUseCase<const N: usize, WP>
where
    WP: WorkingHoursPolicy,
{
    duration: Duration,
    policy: WP,
    provider: Arc<dyn ReservedSlotsProvider<N>>,
}

impl<const N: usize, WP> SlotsUseCase<N, WP>
where
    WP: WorkingHoursPolicy,
{
    pub fn new(
        duration: Duration,
        policy: WP,
        provider: Arc<dyn ReservedSlotsProvider<N>>,
    ) -> Self {
        Self {
            duration,
            policy,
            provider,
        }
    }

    pub async fn slots(&self, date: NaiveDate) -> Result<Vec<SlotDTO>, Error> {
        let slots = self.policy.generate_slots(date, self.duration)?;
        Ok(self
            .provider
            .reserved_slots(slots)
            .await?
            .iter()
            .map(|slot| slot.into())
            .collect())
    }
}
