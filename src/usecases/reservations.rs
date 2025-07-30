use chrono::NaiveDate;
use std::sync::Arc;

use crate::domain::Error;
use crate::domain::interfaces::ReservedSlotsProvider;
use crate::domain::services::{SlotsFactory, WorkingHoursPolicy};
use crate::usecases::ReservationDTO;

#[derive(Clone)]
pub struct ReservationsUseCase {
    factory: Arc<dyn SlotsFactory>,
    policy: Arc<dyn WorkingHoursPolicy>,
    provider: Arc<dyn ReservedSlotsProvider>,
}

impl ReservationsUseCase {
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

    pub async fn reservations(&self, date: NaiveDate) -> Result<Vec<ReservationDTO>, Error> {
        let slots = self.factory.create_all(date, self.policy.as_ref());
        let slots = self.provider.reserved_slots(slots).await?;
        let mut res = Vec::new();
        for slot in slots.into_iter() {
            for r in slot.reservations() {
                res.push(ReservationDTO {
                    slot_start: slot.start(),
                    slot_end: slot.interval().end,
                    service: r.service().clone(),
                    username: r.by().username().as_str().to_string(),
                    user_name_lat: r.by().full_name_lat().as_str().to_string(),
                    user_name_cyr: r.by().full_name_cyr().as_str().to_string(),
                    citizenship: r.by().citizenship().clone(),
                    arrival_date: r.by().arrival_date().clone(),
                })
            }
        }
        Ok(res)
    }
}
