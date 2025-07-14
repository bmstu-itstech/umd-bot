use std::sync::Mutex;
use async_trait::async_trait;
use chrono::NaiveDate;

use crate::domain::Error;
use crate::domain::interfaces::{BookingsProvider, BookingsRepository};
use crate::domain::models::Booking;


pub struct MockBookingsRepository {
    bs: Mutex<Vec<Booking>>,
}

#[async_trait]
impl BookingsProvider for MockBookingsRepository {
    async fn bookings(&self, date: NaiveDate) -> Result<Vec<Booking>, Error> {
        let bs = self.bs.lock().unwrap();
        Ok(bs
            .iter()
            .filter(|&b| b.time().date_naive() == date)
            .map(|b| b.clone())
            .collect()
        )
    }
}

#[async_trait]
impl BookingsRepository for MockBookingsRepository {
    async fn save(&self, booking: Booking) -> Result<(), Error> {
        let mut bs = self.bs.lock().unwrap();
        bs.push(booking);
        Ok(())
    }

    async fn remove(&self, booking: Booking) -> Result<(), Error> {
        let mut bs = self.bs.lock().unwrap();
        let before = bs.len();
        bs.retain(|b| b != &booking);
        let after = bs.len();
        if before == after {
            Err(Error::SlotNotFoundError)
        } else {
            Ok(())
        }
    }
}
