use std::sync::{Arc, Mutex};
use chrono::{NaiveDate, Duration};

use crate::domain::Error;
use crate::domain::interfaces::{ReservedSlotsProvider};
use crate::domain::services::WorkingHoursPolicy;
use crate::usecases::Slot;


pub struct SlotsUseCase<const N: usize, WP>
where
    WP: WorkingHoursPolicy,
{
    duration: Duration,
    policy: WP,
    provider: Arc<Mutex<dyn ReservedSlotsProvider<N>>>,
}

impl<const N: usize, WP> SlotsUseCase<N, WP>
where
    WP: WorkingHoursPolicy,
{
    pub fn new(duration: Duration, policy: WP, provider: Arc<Mutex<dyn ReservedSlotsProvider<N>>>) -> Self {
        Self { duration, policy, provider }
    }
    
    pub async fn slots(&self, date: NaiveDate) -> Result<Vec<Slot>, Error> {
        let provider = self.provider.lock().unwrap();
        
        let slots = self.policy.generate_slots(date, self.duration)?;
        Ok(provider
            .reserved_slots(slots)
            .await?
            .into_iter()
            .map(|slot| slot.into())
            .collect()
        )
    }
}
