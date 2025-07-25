use std::sync::{Arc, Mutex};

use crate::usecases::User;
use crate::domain::{models, Error};
use crate::domain::interfaces::UserRepository;


pub struct RegisterUserUseCase {
    repos: Arc<Mutex<dyn UserRepository>>,
}

impl RegisterUserUseCase {
    pub fn new(repos: Arc<Mutex<dyn UserRepository>>) -> Self {
        Self { repos }
    }
    
    pub async fn register(&self, u: &User) -> Result<(), Error> {
        let id = models::TelegramID::new(u.id);
        let username = models::TelegramUsername::new(u.username.clone());
        let full_name_lat = models::OnlyLatin::new(u.full_name_lat.clone())?;
        let full_name_cyr = models::OnlyCyrillic::new(u.full_name_cyr.clone())?;
        let citizenship: models::Citizenship = u.citizenship.clone().into();
        let arrival_date = u.arrival_date.clone();
        
        let user = models::User::new(
            id, username, full_name_lat, full_name_cyr, citizenship, arrival_date
        );

        let repos = self.repos.lock().unwrap();
        repos.save_user(&user)
            .await?;
        
        Ok(())
    }
}
