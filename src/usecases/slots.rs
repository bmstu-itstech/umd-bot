use chrono::NaiveDate;
use std::sync::Arc;

use crate::domain::Error;
use crate::domain::interfaces::ReservedSlotsProvider;
use crate::domain::services::{SlotsFactory, WorkingHoursPolicy};
use crate::usecases::SlotDTO;

#[derive(Clone)]
pub struct SlotsUseCase {
    factory: Arc<dyn SlotsFactory>,
    policy: Arc<dyn WorkingHoursPolicy>,
    provider: Arc<dyn ReservedSlotsProvider>,
}

impl SlotsUseCase
{
    pub fn new(
        factory: Arc<dyn SlotsFactory>,
        policy: Arc<dyn WorkingHoursPolicy>,
        provider: Arc<dyn ReservedSlotsProvider>,
    ) -> Self {
        Self {
            factory,
            policy,
            provider,
        }
    }

    pub async fn slots(&self, date: NaiveDate) -> Result<Vec<SlotDTO>, Error> {
        let slots = self.factory.create_all(date, self.policy.as_ref());
        Ok(self
            .provider
            .reserved_slots(slots)
            .await?
            .iter()
            .map(|slot| slot.into())
            .collect())
    }
}
