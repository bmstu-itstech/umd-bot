use chrono::{DateTime, NaiveDate, Utc};
use crate::domain::models;

#[derive(Clone, Debug)]
pub enum Citizenship {
    Tajikistan,
    Uzbekistan,
    Kazakhstan,
    Kyrgyzstan,
    Armenia,
    Belarus,
    Ukraine,
    Other(String),
}

#[derive(Clone, Debug)]
pub struct Student {
    pub id:            i64,
    pub username:      String,
    pub full_name_lat: String,
    pub full_name_cyr: String,
    pub citizenship:   Citizenship,
    pub arrival_date:  NaiveDate,
}

#[derive(Clone, Debug)]
pub struct EmptySlot {
    pub start: DateTime<Utc>,
    pub end:   DateTime<Utc>,
}

#[derive(Clone, Debug)]
pub struct BookedSlot {
    pub start:     DateTime<Utc>,
    pub end:       DateTime<Utc>,
    pub booked_by: Vec<Student>,
}

impl Into<String> for Citizenship {
    fn into(self) -> String {
        match self {
            Citizenship::Tajikistan => "Таджикистан".to_string(),
            Citizenship::Uzbekistan => "Узбекистан".to_string(),
            Citizenship::Kazakhstan => "Казахстан".to_string(),
            Citizenship::Kyrgyzstan => "Кыргызстан".to_string(),
            Citizenship::Armenia    => "Армения".to_string(),
            Citizenship::Belarus    => "Беларусь".to_string(),
            Citizenship::Ukraine    => "Украина".to_string(),
            Citizenship::Other(s) => s,
        }
    }
}

impl From<String> for Citizenship {
    fn from(s: String) -> Self {
        match s.as_str() {
            "Таджикистан" => Citizenship::Tajikistan,
            "Узбекистан"  => Citizenship::Uzbekistan,
            "Казахстан"   => Citizenship::Kazakhstan,
            "Кыргызстан"  => Citizenship::Kyrgyzstan,
            "Армения"     => Citizenship::Armenia,
            "Беларусь"    => Citizenship::Belarus,
            "Украина"     => Citizenship::Ukraine,
            s => Citizenship::Other(s.into()),
        }
    }
}

impl Into<models::Citizenship> for Citizenship {
    fn into(self) -> models::Citizenship {
        match self {
            Citizenship::Tajikistan => models::Citizenship::Tajikistan,
            Citizenship::Uzbekistan => models::Citizenship::Uzbekistan,
            Citizenship::Kazakhstan => models::Citizenship::Kazakhstan,
            Citizenship::Kyrgyzstan => models::Citizenship::Kyrgyzstan,
            Citizenship::Armenia    => models::Citizenship::Armenia,
            Citizenship::Belarus    => models::Citizenship::Belarus,
            Citizenship::Ukraine    => models::Citizenship::Ukraine,
            Citizenship::Other(s) => models::Citizenship::Other(s.into()),
        }
    }
}

impl From<models::Citizenship> for Citizenship {
    fn from(s: models::Citizenship) -> Self {
        match s {
            models::Citizenship::Tajikistan => Citizenship::Tajikistan,
            models::Citizenship::Uzbekistan => Citizenship::Uzbekistan,
            models::Citizenship::Kazakhstan => Citizenship::Kazakhstan,
            models::Citizenship::Kyrgyzstan => Citizenship::Kyrgyzstan,
            models::Citizenship::Armenia    => Citizenship::Armenia,
            models::Citizenship::Belarus    => Citizenship::Belarus,
            models::Citizenship::Ukraine    => Citizenship::Ukraine,
            models::Citizenship::Other(s) => Citizenship::Other(s.into()),
        }
    }
}
