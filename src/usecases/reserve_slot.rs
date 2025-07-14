use std::sync::{Arc, Mutex};
use chrono::{DateTime, Duration, Utc};

use crate::domain::Error;
use crate::domain::interfaces::{AvailableSlotsProvider, SlotsRepository, UserProvider};
use crate::domain::models::TelegramID;
use crate::domain::services::WorkingHoursPolicy;


pub struct ReserveSlotUseCase<const N: usize, WP>
where 
    WP: WorkingHoursPolicy,
{
    duration: Duration,
    policy: WP,
    user_provider: Arc<Mutex<dyn UserProvider>>,
    provider: Arc<Mutex<dyn AvailableSlotsProvider<N>>>,
    repos: Arc<Mutex<dyn SlotsRepository<N>>>,
}

impl<const N: usize, WP> ReserveSlotUseCase<N, WP>
where
    WP: WorkingHoursPolicy,
{
    pub fn new(
        duration: Duration, 
        policy: WP, 
        user_provider: Arc<Mutex<dyn UserProvider>>,
        provider: Arc<Mutex<dyn AvailableSlotsProvider<N>>>,
        repos: Arc<Mutex<dyn SlotsRepository<N>>>,
    ) -> Self {
        Self { duration, policy, user_provider, provider, repos }
    }
    
    pub async fn reserve_slot(&self, user_id: i64, time: DateTime<Utc>) -> Result<(), Error> {
        let user_provider = self.user_provider.lock().unwrap();
        let provider = self.provider.lock().unwrap();
        let repos = self.repos.lock().unwrap();
        
        let user = user_provider
            .user(TelegramID::new(user_id))
            .await?;
        
        let date = time.date_naive();
        let slots = self.policy.generate_slots(date, self.duration)?;
        let mut slots = provider.available_slots(date, slots).await?;
        let res = slots
            .iter_mut()
            .find(|slot| slot.interval().start == time);
        
        let slot = match res {
            Some(slot) => slot,
            None => return Err(Error::SlotNotFoundError),
        };
        
        slot.reserve(&user)?;
        repos.save(slot).await?;
        Ok(())
    }
}
