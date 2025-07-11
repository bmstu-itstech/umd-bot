use async_trait::async_trait;
use chrono::NaiveDate;

use crate::domain::Error;
use crate::domain::models::{Booking, Student};


#[async_trait]
pub trait StudentsProvider {
    async fn student(&self, tg_username: &str) -> Result<Student, Error>;
}

#[async_trait]
pub trait StudentsRepository {
    async fn save(&self, student: Student) -> Result<(), Error>;
    async fn update(&self, student: Student) -> Result<(), Error>;
}

#[async_trait]
pub trait BookingsProvider {
    async fn bookings(&self, date: NaiveDate) -> Result<Vec<Booking>, Error>;
}
