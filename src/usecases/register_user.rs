use chrono::NaiveDate;
use std::sync::Arc;

use crate::domain::Error;
use crate::domain::interfaces::UserRepository;
use crate::domain::models::{Citizenship, OnlyCyrillic, OnlyLatin, User, UserID, Username};

pub struct RegisterUserRequest {
    pub id: UserID,
    pub username: Username,
    pub full_name_lat: OnlyLatin,
    pub full_name_cyr: OnlyCyrillic,
    pub citizenship: Citizenship,
    pub arrival_date: NaiveDate,
}

#[derive(Clone)]
pub struct RegisterUserUseCase {
    repos: Arc<dyn UserRepository>,
}

impl RegisterUserUseCase {
    pub fn new(repos: Arc<dyn UserRepository>) -> Self {
        Self { repos }
    }

    pub async fn register(&self, req: RegisterUserRequest) -> Result<(), Error> {
        let user = User::new(
            req.id,
            req.username,
            req.full_name_lat,
            req.full_name_cyr,
            req.citizenship,
            req.arrival_date,
        );
        self.repos.save_user(user).await?;
        Ok(())
    }
}
