use async_trait::async_trait;
use chrono::{DateTime, Utc};

use crate::domain::Error;
use crate::domain::models::{Slot, TelegramID, User};

#[async_trait]
pub trait HasAvailableSlotsProvider<const N: usize>: Send + Sync {
    async fn has_available_slots(&self, slots: &[Slot<N>]) -> Result<bool, Error>;
}

#[async_trait]
pub trait AvailableSlotsProvider<const N: usize>: Send + Sync {
    async fn available_slots(&self, slots: Vec<Slot<N>>) -> Result<Vec<Slot<N>>, Error>;
}

#[async_trait]
pub trait ReservedSlotsProvider<const N: usize>: Send + Sync {
    async fn reserved_slots(&self, slots: Vec<Slot<N>>) -> Result<Vec<Slot<N>>, Error>;
}

#[async_trait]
pub trait ReservedSlotProvider<const N: usize>: Send + Sync {
    async fn reserved_slot(&self, datetime: DateTime<Utc>) -> Result<Slot<N>, Error>;
}

#[async_trait]
pub trait SlotsRepository<const N: usize>: Send + Sync {
    async fn save_slot(&self, slot: &Slot<N>) -> Result<(), Error>;
}

#[async_trait]
pub trait UserProvider: Send + Sync {
    async fn user(&self, id: TelegramID) -> Result<User, Error>;
}

#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn save_user(&self, user: &User) -> Result<(), Error>;
}
