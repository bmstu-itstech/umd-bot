use std::sync::{Arc, Mutex};
use chrono::{DateTime, Utc};

use crate::domain::interfaces::BookingsRepository;
use crate::domain::models::TelegramID;
use crate::domain::services::{SlotsFactory, WorkingHoursPolicy};


#[derive(thiserror::Error, Debug)]
pub enum BookError {
    #[error("slot not found")]
    SlotNotFound,
    
    #[error("slot is not available")]
    SlotIsNotAvailable,
    
    #[error(transparent)]
    Internal(#[from] Box<dyn std::error::Error + Send + Sync>),
}

pub struct BookUseCase<const N: usize, SF, WP>
where
    SF: SlotsFactory<N>,
    WP: WorkingHoursPolicy,
{
    slots_factory: SF,
    working_time_policy: WP,
    bookings_repo: Arc<Mutex<dyn BookingsRepository>>,
}

impl<const N: usize, SF, WP> BookUseCase<N, SF, WP>
where
    SF: SlotsFactory<N>,
    WP: WorkingHoursPolicy,
{
    pub fn new(slots_factory: SF, working_time_policy: WP, bookings_repo: Arc<Mutex<dyn BookingsRepository>>) -> Self {
        BookUseCase { slots_factory, working_time_policy, bookings_repo }
    }
    
    pub async fn book(&self, datetime: DateTime<Utc>, by: i64) -> Result<(), BookError> {
        let bounds = self.working_time_policy
            .bounds(datetime.date_naive())
            .ok_or(BookError::SlotNotFound)?;
        
        let mut slot = self.slots_factory
            .slots(&bounds)
            .find(|slot| slot.interval().start() == &datetime)
            .ok_or(BookError::SlotIsNotAvailable)?;
        
        let booking = slot.book(TelegramID::new(by))
            .map_err(|_| BookError::SlotIsNotAvailable)?;
        
        let bookings_repository = self.bookings_repo
            .lock()
            .unwrap();
        
        bookings_repository
            .save(booking)
            .await
            .map_err(|err| BookError::Internal(Box::new(err)))
    }
}
