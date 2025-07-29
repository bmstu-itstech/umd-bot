use chrono::NaiveDate;
use std::sync::Arc;

use crate::domain::Error;
use crate::domain::interfaces::{UserProvider, UserRepository};
use crate::domain::models::{Citizenship, OnlyCyrillic, OnlyLatin, UserID};

#[derive(Clone)]
pub struct UpdateUserUseCase {
    repos: Arc<dyn UserRepository>,
    provider: Arc<dyn UserProvider>,
}

impl UpdateUserUseCase {
    pub fn new(repos: Arc<dyn UserRepository>, provider: Arc<dyn UserProvider>) -> Self {
        Self { repos, provider }
    }

    pub async fn update_name_lat(&self, id: i64, name: OnlyLatin) -> Result<(), Error> {
        let mut user = self.provider.user(UserID::new(id)).await?;
        user.set_full_name_lat(name);
        self.repos.save_user(user).await?;
        Ok(())
    }

    pub async fn update_name_cyr(&self, id: i64, name: OnlyCyrillic) -> Result<(), Error> {
        let mut user = self.provider.user(UserID::new(id)).await?;
        user.set_full_name_cyr(name);
        self.repos.save_user(user).await?;
        Ok(())
    }

    pub async fn update_citizenship(&self, id: i64, citizenship: Citizenship) -> Result<(), Error> {
        let mut user = self.provider.user(UserID::new(id)).await?;
        user.set_citizenship(citizenship);
        self.repos.save_user(user).await?;
        Ok(())
    }

    pub async fn update_arrival_date(&self, id: i64, arrival_date: NaiveDate) -> Result<(), Error> {
        let mut user = self.provider.user(UserID::new(id)).await?;
        user.set_arrival_date(arrival_date);
        self.repos.save_user(user).await?;
        Ok(())
    }
}
