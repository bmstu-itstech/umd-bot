use crate::domain::models::{Citizenship, CyrillicText, LatinText, Service, Slot, User};
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug)]
pub struct UserDTO {
    pub full_name_lat: LatinText,
    pub full_name_cyr: CyrillicText,
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
    pub username: String,
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
            full_name_lat: user.full_name_lat().clone(),
            full_name_cyr: user.full_name_cyr().clone(),
            citizenship: user.citizenship().clone(),
            arrival_date: user.arrival_date().clone(),
        }
    }
}
