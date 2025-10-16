use chrono::{DateTime, Utc};
use std::sync::Arc;

use crate::domain::Error;
use crate::domain::interfaces::{ReservedSlotProvider, SlotsRepository};
use crate::domain::models::UserID;
use crate::domain::services::{ServicePolicy, SlotsFactory};

#[derive(Clone)]
pub struct CancelReservationUseCase {
    factory: Arc<dyn SlotsFactory>,
    provider: Arc<dyn ReservedSlotProvider>,
    repos: Arc<dyn SlotsRepository>,
    sp: Arc<dyn ServicePolicy>,
}

impl CancelReservationUseCase {
    pub fn new(
        factory: Arc<dyn SlotsFactory>,
        provider: Arc<dyn ReservedSlotProvider>,
        repos: Arc<dyn SlotsRepository>,
        sp: Arc<dyn ServicePolicy>,
    ) -> Self {
        Self {
            factory,
            provider,
            repos,
            sp,
        }
    }

    pub async fn cancel_reservation(
        &self,
        user_id: UserID,
        time: DateTime<Utc>,
    ) -> Result<(), Error> {
        let services = self.sp.available_services(time.date_naive());
        let slot = self.factory.create(time, services)?;
        let mut slot = self.provider.reserved_slot(slot).await?;
        slot.cancel(user_id)?;
        self.repos.save_slot(&slot).await?;
        Ok(())
    }
}
