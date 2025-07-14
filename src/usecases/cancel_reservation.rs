use std::sync::{Arc, Mutex};
use chrono::{DateTime, Duration, Utc};

use crate::domain::Error;
use crate::domain::interfaces::{ReservedSlotProvider, SlotsRepository, UserProvider};
use crate::domain::models::TelegramID;

pub struct CancelReservationUseCase<const N: usize> {
    duration:      Duration,
    user_provider: Arc<Mutex<dyn UserProvider>>,
    provider:      Arc<Mutex<dyn ReservedSlotProvider<N>>>,
    repos:         Arc<Mutex<dyn SlotsRepository<N>>>,
}

impl<const N: usize> CancelReservationUseCase<N> {
    pub async fn cancel_reservation(&self, user_id: i64, time: DateTime<Utc>) -> Result<(), Error> {
        let user_provider = self.user_provider.lock().unwrap();
        let provider = self.provider.lock().unwrap();
        let repos = self.repos.lock().unwrap();
        
        let mut slot = provider
            .reserved_slot(time)
            .await?;
        
        let user = user_provider
            .user(TelegramID::new(user_id))
            .await?;
        
        slot.cancel(&user)?;
        
        if slot.is_empty() {
            repos.delete(&slot).await?;
        } else {
            repos.save(&slot).await?;
        }
        
        Ok(())
    }
}
