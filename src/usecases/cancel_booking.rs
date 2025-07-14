use std::sync::{Arc, Mutex};
use chrono::{DateTime, Utc};

use crate::domain;
use crate::domain::interfaces::BookingsRepository;
use crate::domain::models::{Booking, TelegramID};


#[derive(thiserror::Error, Debug)]
pub enum CancelBookingError {
    #[error("booked slot not found")]
    SlotNotFound,
    
    #[error(transparent)]
    Internal(#[from] Box<dyn std::error::Error + Send + Sync>),
}

pub struct CancelBookingUseCase {
    booking_repo: Arc<Mutex<dyn BookingsRepository>>
}

impl CancelBookingUseCase {
    pub fn new(booking_repo: Arc<Mutex<dyn BookingsRepository>>) -> Self {
        Self { booking_repo }
    }
    
    pub async fn cancel_booking(&self, student_id: i64, time: DateTime<Utc>) -> Result<(), CancelBookingError> {
        let booking_repo = self.booking_repo.lock().unwrap();
        
        let booking = Booking::new(time, TelegramID::new(student_id));
        
        booking_repo
            .remove(booking)
            .await
            .map_err(|err| match err {
                domain::Error::SlotNotFoundError => CancelBookingError::SlotNotFound,
                _ => CancelBookingError::Internal(Box::new(err)),
            })
    }
}
