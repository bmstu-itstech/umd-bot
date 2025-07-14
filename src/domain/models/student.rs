use std::fmt::Display;
use chrono::NaiveDate;

use crate::domain::Error;
use crate::domain::models::Citizenship;


#[derive(Debug, Clone)]
pub struct OnlyLatin(String);

impl OnlyLatin {
    fn check(s: &str) -> Result<(), Error> {
        if s.chars().into_iter().all(|c| c.is_ascii_alphabetic() || c == ' ' || c == '-') {
            Ok(())
        } else {
            Err(Error::InvalidValue(format!("OnlyLatin: got {}", s)))
        }
    }
    
    pub fn new(s: impl Into<String>) -> Result<Self, Error> {
        let s = s.into();
        Self::check(&s)?;
        Ok(Self(s))
    }
    
    pub fn as_str(&self) -> &str {
        &self.0
    }
}


trait CyrillicCheck {
    fn is_cyrillic(&self) -> bool;
}

impl CyrillicCheck for char {
    fn is_cyrillic(&self) -> bool {
        matches!(*self,
            'а'..='я' |
            'А'..='Я' |
            'ё' | 'Ё'
        )
    }
}

#[derive(Debug, Clone)]
pub struct OnlyCyrillic(String);

impl OnlyCyrillic {
    fn check(s: &str) -> Result<(), Error> {
        if s.chars().into_iter().all(|c| c.is_cyrillic() || c == ' ' || c == '-') {
            Ok(())
        } else {
            Err(Error::InvalidValue(format!("OnlyCyrillic: got {}", s)))
        }
    }
    
    pub fn new(s: impl Into<String>) -> Result<Self, Error> {
        let s = s.into();
        Self::check(&s)?;
        Ok(Self(s))
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TelegramID(i64);

impl TelegramID {
    pub fn new(id: impl Into<i64>) -> Self {
        Self(id.into())
    }
    
    pub fn as_i64(&self) -> i64 {
        self.0
    }
}

impl Display for TelegramID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone)]
pub struct TelegramUsername(String);

impl TelegramUsername {
    pub fn new(username: impl Into<String>) -> Self {
        Self(username.into())
    }
    
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

#[derive(Debug, Clone)]
pub struct Student {
    id:            TelegramID,   
    username:      TelegramUsername,
    full_name_lat: OnlyLatin,
    full_name_cyr: OnlyCyrillic,
    citizenship:   Citizenship,
    arrival_date:  NaiveDate,
}

impl Student {
    pub fn new(
        id: TelegramID,
        username: TelegramUsername,
        full_name_lat: OnlyLatin,
        full_name_cyr: OnlyCyrillic,
        citizenship: Citizenship,
        arrival_date: NaiveDate,
    ) -> Self {
        Self { id, username, full_name_lat, full_name_cyr, citizenship, arrival_date }
    }
    
    pub fn id(&self) -> &TelegramID {
        &self.id
    }
    
    pub fn username(&self) -> &TelegramUsername {
        &self.username
    }
    
    pub fn full_name_lat(&self) -> &OnlyLatin {
        &self.full_name_lat
    }
    
    pub fn full_name_cyr(&self) -> &OnlyCyrillic {
        &self.full_name_cyr
    }
    
    pub fn citizenship(&self) -> &Citizenship {
        &self.citizenship
    }
    
    pub fn arrival_data(&self) -> &NaiveDate {
        &self.arrival_date
    }
    
    pub fn set_full_name_lat(&mut self, full_name_lat: OnlyLatin) {
        self.full_name_lat = full_name_lat;
    }
    
    pub fn set_full_name_cyr(&mut self, full_name_cyr: OnlyCyrillic) {
        self.full_name_cyr = full_name_cyr;
    }
    
    pub fn set_citizenship(&mut self, citizenship: Citizenship) {
        self.citizenship = citizenship;
    }
    
    pub fn set_arrival_data(&mut self, arrival_data: NaiveDate) {
        self.arrival_date = arrival_data;
    }
}



#[cfg(test)]
mod tests {
    use super::*;
    
    mod only_latin {
        use super::*;

        #[test]
        fn should_allow_valid_chars() {
            assert!(OnlyLatin::new("Ivanov Ivan Ivanovich".to_string()).is_ok());
        }
        
        #[test]
        fn should_not_allow_cyrillic() {
            assert!(OnlyLatin::new("Иван".to_string()).is_err());
        }
        
        #[test]
        fn should_not_allow_special_chars() {
            assert!(OnlyLatin::new("Ivan 2".to_string()).is_err());
        }
    }

    mod only_cyrillic {
        use super::*;

        #[test]
        fn should_allow_valid_chars() {
            assert!(OnlyCyrillic::new("Иванов Иван Иванович".to_string()).is_ok());
        }

        #[test]
        fn should_not_allow_not_cyrillic() {
            assert!(OnlyCyrillic::new("Ivan".to_string()).is_err());
        }

        #[test]
        fn should_not_allow_special_chars() {
            assert!(OnlyCyrillic::new("Иван 2".to_string()).is_err());
        }
    }
}
