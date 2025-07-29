use chrono::NaiveDate;
use std::sync::Arc;

use crate::domain::Error;
use crate::domain::interfaces::AvailableSlotsProvider;
use crate::domain::services::{SlotsFactory, WorkingHoursPolicy};
use crate::usecases::FreeSlotDTO;

#[derive(Clone)]
pub struct FreeSlotsUseCase {
    factory: Arc<dyn SlotsFactory>,
    policy: Arc<dyn WorkingHoursPolicy>,
    provider: Arc<dyn AvailableSlotsProvider>,
}

impl FreeSlotsUseCase {
    pub fn new(
        factory: Arc<dyn SlotsFactory>,
        policy: Arc<dyn WorkingHoursPolicy>,
        provider: Arc<dyn AvailableSlotsProvider>,
    ) -> Self {
        Self {
            factory,
            policy,
            provider,
        }
    }

    pub async fn free_slots(&self, date: NaiveDate) -> Result<Vec<FreeSlotDTO>, Error> {
        let slots = self.factory.create_all(date, self.policy.as_ref());
        let slots = self.provider.available_slots(slots).await?;
        Ok(slots.iter().map(|slot| slot.into()).collect())
    }
}
