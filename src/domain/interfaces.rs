use async_trait::async_trait;

use crate::domain::Error;
use crate::domain::models::{Slot, User, UserID};

#[async_trait]
pub trait HasAvailableSlotsProvider: Send + Sync {
    async fn has_available_slots(&self, slots: &[Slot]) -> Result<bool, Error>;
}

#[async_trait]
pub trait AvailableSlotsProvider: Send + Sync {
    async fn available_slots(&self, slots: Vec<Slot>) -> Result<Vec<Slot>, Error>;
}

#[async_trait]
pub trait ReservedSlotsProvider: Send + Sync {
    async fn reserved_slots(&self, slots: Vec<Slot>) -> Result<Vec<Slot>, Error>;
}

#[async_trait]
pub trait ReservedSlotProvider: Send + Sync {
    async fn reserved_slot(&self, slot: Slot) -> Result<Slot, Error>;
}

#[async_trait]
pub trait SlotsRepository: Send + Sync {
    async fn save_slot(&self, slot: &Slot) -> Result<(), Error>;
}

#[async_trait]
pub trait UserProvider: Send + Sync {
    async fn user(&self, id: UserID) -> Result<User, Error>;
}

#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn save_user(&self, user: User) -> Result<(), Error>;
}

#[async_trait]
pub trait AdminProvider: Send + Sync {
    async fn is_admin(&self, id: UserID) -> Result<bool, Error>;
}
