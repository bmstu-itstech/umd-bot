use async_trait::async_trait;
use chrono::{DateTime, NaiveDate, Utc};

use crate::domain::Error;
use crate::domain::models::{Slot, TelegramID, User};


#[async_trait]
pub trait HasAvailableSlotsProvider<const N: usize> {
    async fn has_available_slots(&self, date: NaiveDate, slots: &[Slot<N>]) -> Result<bool, Error>;
}

#[async_trait]
pub trait AvailableSlotsProvider<const N: usize> {
    async fn available_slots(&self, date: NaiveDate, slots: Vec<Slot<N>>) -> Result<Vec<Slot<N>>, Error>;
}

#[async_trait]
pub trait ReservedSlotsProvider<const N: usize> {
    async fn reserved_slots(&self, slots: Vec<Slot<N>>) -> Result<Vec<Slot<N>>, Error>;
}

#[async_trait]
pub trait ReservedSlotProvider<const N: usize> {
    async fn reserved_slot(&self, datetime: DateTime<Utc>) -> Result<Slot<N>, Error>;
}

#[async_trait]
pub trait SlotsRepository<const N: usize> {
    async fn save(&self, slot: &Slot<N>) -> Result<(), Error>;
    async fn delete(&self, slot: &Slot<N>) -> Result<(), Error>;
}

#[async_trait]
pub trait UserProvider {
    async fn user(&self, id: TelegramID) -> Result<User, Error>;
}

#[async_trait]
pub trait UserRepository {
    async fn save(&self, user: &User) -> Result<(), Error>;
}
