use async_trait::async_trait;
use chrono::NaiveDate;

use crate::domain::Error;
use crate::domain::models::{Booking, Student, TelegramID};


#[async_trait]
pub trait StudentsProvider {
    async fn student(&self, id: TelegramID) -> Result<Student, Error>;
}

#[async_trait]
pub trait StudentsRepository: StudentsProvider {
    async fn save(&self, student: Student) -> Result<(), Error>;
    async fn update(&self, student: Student) -> Result<(), Error>;
}

#[async_trait]
pub trait BookingsProvider {
    async fn bookings(&self, date: NaiveDate) -> Result<Vec<Booking>, Error>;
}

#[async_trait]
pub trait BookingsRepository: BookingsProvider {
    async fn save(&self, booking: Booking) -> Result<(), Error>;
    async fn remove(&self, booking: Booking) -> Result<(), Error>;
}
