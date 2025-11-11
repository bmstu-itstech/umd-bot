use chrono::{DateTime, Utc};
use std::sync::Arc;

use crate::domain::Error;
use crate::domain::interfaces::{AvailableSlotsProvider, SlotsRepository, UserProvider};
use crate::domain::models::{Service, UserID};
use crate::domain::services::{ServicePolicy, SlotsFactory, WorkingHoursPolicy};

#[derive(Clone)]
pub struct ReserveSlotUseCase {
    factory: Arc<dyn SlotsFactory>,
    user_provider: Arc<dyn UserProvider>,
    as_provider: Arc<dyn AvailableSlotsProvider>,
    repos: Arc<dyn SlotsRepository>,
    wp: Arc<dyn WorkingHoursPolicy>,
    sp: Arc<dyn ServicePolicy>,
}

impl ReserveSlotUseCase {
    pub fn new(
        factory: Arc<dyn SlotsFactory>,
        user_provider: Arc<dyn UserProvider>,
        as_provider: Arc<dyn AvailableSlotsProvider>,
        repos: Arc<dyn SlotsRepository>,
        wp: Arc<dyn WorkingHoursPolicy>,
        sp: Arc<dyn ServicePolicy>,
    ) -> Self {
        Self {
            factory,
            wp,
            user_provider,
            as_provider,
            repos,
            sp,
        }
    }

    pub async fn reserve_slot(
        &self,
        user_id: UserID,
        time: DateTime<Utc>,
        service: Service,
    ) -> Result<(), Error> {
        let user = self.user_provider.user(user_id).await?;

        let date = time.date_naive();
        let slots = self
            .factory
            .create_all(date, self.wp.as_ref(), self.sp.as_ref())?;
        let mut slots = self.as_provider.available_slots(slots).await?;
        let res = slots.iter_mut().find(|slot| slot.interval().start == time);

        let slot = match res {
            Some(slot) => slot,
            None => return Err(Error::SlotNotFound),
        };

        slot.reserve(user, service)?;
        self.repos.save_slot(slot).await?;
        Ok(())
    }
}
