use chrono::{DateTime, NaiveDate, Utc};

use crate::domain::models::{
    Citizenship, OnlyCyrillic, OnlyLatin, Reservation, Service, Slot, User, UserID, Username,
};

#[derive(Clone, Debug)]
pub struct UserDTO {
    pub id: UserID,
    pub username: Username,
    pub full_name_lat: OnlyLatin,
    pub full_name_cyr: OnlyCyrillic,
    pub citizenship: Citizenship,
    pub arrival_date: NaiveDate,
}

#[derive(Clone, Debug)]
pub struct FreeSlotDTO {
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
}

#[derive(Clone, Debug)]
pub struct ReservationDTO {
    pub by: UserDTO,
    pub service: Service,
}

#[derive(Clone, Debug)]
pub struct SlotDTO {
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
    pub reservations: Vec<ReservationDTO>,
}

impl<const N: usize> From<&Slot<N>> for FreeSlotDTO {
    fn from(s: &Slot<N>) -> Self {
        Self {
            start: s.start(),
            end: s.interval().end,
        }
    }
}

impl From<&Reservation> for ReservationDTO {
    fn from(r: &Reservation) -> Self {
        Self {
            by: r.by().into(),
            service: r.service().clone(),
        }
    }
}

impl From<&User> for UserDTO {
    fn from(user: &User) -> Self {
        Self {
            id: user.id(),
            username: user.username().clone(),
            full_name_lat: user.full_name_lat().clone(),
            full_name_cyr: user.full_name_cyr().clone(),
            citizenship: user.citizenship().clone(),
            arrival_date: user.arrival_date().clone(),
        }
    }
}

impl<const N: usize> From<&Slot<N>> for SlotDTO {
    fn from(slot: &Slot<N>) -> Self {
        Self {
            start: slot.interval().start,
            end: slot.interval().end,
            reservations: slot.reservations().iter().map(|r| r.into()).collect(),
        }
    }
}
