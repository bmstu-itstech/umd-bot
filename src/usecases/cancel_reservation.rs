use chrono::{DateTime, Duration, Utc};
use std::ops::Add;
use std::sync::Arc;

use crate::domain::Error;
use crate::domain::interfaces::{ReservedSlotProvider, SlotsRepository};
use crate::domain::models::{ClosedRange, Slot, UserID};

#[derive(Clone)]
pub struct CancelReservationUseCase<const N: usize> {
    duration: Duration,
    provider: Arc<dyn ReservedSlotProvider<N>>,
    repos: Arc<dyn SlotsRepository<N>>,
}

impl<const N: usize> CancelReservationUseCase<N> {
    pub fn new(
        duration: Duration,
        provider: Arc<dyn ReservedSlotProvider<N>>,
        repos: Arc<dyn SlotsRepository<N>>,
    ) -> Self {
        Self {
            duration,
            provider,
            repos,
        }
    }

    pub async fn cancel_reservation(
        &self,
        user_id: UserID,
        time: DateTime<Utc>,
    ) -> Result<(), Error> {
        let slot = Slot::empty(ClosedRange {
            start: time,
            end: time.add(self.duration),
        });
        let mut slot = self.provider.reserved_slot(slot).await?;
        slot.cancel(user_id)?;
        self.repos.save_slot(&slot).await?;
        Ok(())
    }
}
