use chrono::NaiveDate;
use std::sync::Arc;

use crate::domain::Error;
use crate::domain::interfaces::AvailableSlotsProvider;
use crate::domain::models::Service;
use crate::domain::services::{ServicePolicy, SlotsFactory, WorkingHoursPolicy};
use crate::usecases::FreeSlotDTO;

#[derive(Clone)]
pub struct FreeSlotsUseCase {
    factory: Arc<dyn SlotsFactory>,
    policy: Arc<dyn WorkingHoursPolicy>,
    ap: Arc<dyn AvailableSlotsProvider>,
    sp: Arc<dyn ServicePolicy>,
}

impl FreeSlotsUseCase {
    pub fn new(
        factory: Arc<dyn SlotsFactory>,
        policy: Arc<dyn WorkingHoursPolicy>,
        ap: Arc<dyn AvailableSlotsProvider>,
        sp: Arc<dyn ServicePolicy>,
    ) -> Self {
        Self {
            factory,
            policy,
            ap,
            sp,
        }
    }

    pub async fn free_slots(
        &self,
        date: NaiveDate,
        service: Service,
    ) -> Result<Vec<FreeSlotDTO>, Error> {
        let slots = self
            .factory
            .create_all(date, self.policy.as_ref(), self.sp.as_ref())?;
        Ok(self
            .ap
            .available_slots(slots)
            .await?
            .into_iter()
            .filter(|slot| slot.can_be_served_with(service))
            .map(|slot| FreeSlotDTO::from(&slot))
            .collect())
    }
}
