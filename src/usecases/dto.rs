use crate::domain::models::{
    Citizenship, OnlyCyrillic, OnlyLatin, Service, Slot, User, UserID, Username,
};
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug)]
pub struct UserDTO {
    pub id: UserID,
    pub username: Username,
    pub full_name_lat: OnlyLatin,
    pub full_name_cyr: OnlyCyrillic,
    pub citizenship: Citizenship,
    pub arrival_date: NaiveDate,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FreeSlotDTO {
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
}

pub struct ReservationDTO {
    pub slot_start: DateTime<Utc>,
    pub slot_end: DateTime<Utc>,
    pub service: Service,
    pub user_id: UserID,
    pub user_name_lat: String,
    pub user_name_cyr: String,
    pub citizenship: Citizenship,
    pub arrival_date: NaiveDate,
}

impl From<&Slot> for FreeSlotDTO {
    fn from(s: &Slot) -> Self {
        Self {
            start: s.start(),
            end: s.interval().end,
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
