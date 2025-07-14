use std::sync::{Arc, Mutex};
use chrono::NaiveDate;

use crate::domain::interfaces::BookingsProvider;
use crate::domain::models::Slot;
use crate::domain::services::{AvailableSlotsFactory, WorkingHoursPolicy};


#[derive(thiserror::Error, Debug)]
pub enum AvailableSlotsError {
    #[error(transparent)]
    Internal(#[from] Box<dyn std::error::Error + Send + Sync>),
}

#[derive(Clone)]
pub struct AvailableSlotsUseCase<const N: usize, WH, AF>
where
    WH: WorkingHoursPolicy,
    AF: AvailableSlotsFactory<N>,
{
    working_hours_policy: WH,
    available_slots_factory: AF,
    bookings_provider: Arc<Mutex<dyn BookingsProvider>>,
}

impl<const N: usize, WH, AF> AvailableSlotsUseCase<N, WH, AF>
where
    WH: WorkingHoursPolicy,
    AF: AvailableSlotsFactory<N> + Clone,
{
    pub fn new(
        working_hours_policy: WH,
        available_slots_factory: AF,
        bookings_provider: Arc<Mutex<dyn BookingsProvider>>,
    ) -> Self {
        Self { working_hours_policy, available_slots_factory, bookings_provider }
    }

    pub async fn available_slots(&self, date: NaiveDate) -> Result<Vec<Slot<N>>, AvailableSlotsError>{
        let booking_provider = self.bookings_provider.lock().unwrap();
        let bookings = booking_provider
            .bookings(date)
            .await
            .map_err(|err| AvailableSlotsError::Internal(Box::new(err)))?;  // Ошибка провайдера
        
        self.available_slots_factory
            .available_slots(date, &bookings)
            .map_err(|err| AvailableSlotsError::Internal(Box::new(err)))    // Ошибка переполнения слота
    }
}
