use std::sync::{Arc, Mutex};
use chrono::Duration;
use chrono::NaiveDate;

use crate::domain::Error;
use crate::domain::interfaces::AvailableSlotsProvider;
use crate::domain::services::WorkingHoursPolicy;
use crate::usecases::FreeSlot;


pub struct FreeSlotsUseCase<const N: usize, WP>
where
    WP: WorkingHoursPolicy,
{
    duration: Duration,
    policy: WP,
    provider: Arc<Mutex<dyn AvailableSlotsProvider<N>>>
}

impl<const N: usize, WP> FreeSlotsUseCase<N, WP>
where
    WP: WorkingHoursPolicy,
{
    pub async fn free_slots(&self, date: NaiveDate) -> Result<Vec<FreeSlot>, Error> {
        let provider = self.provider.lock().unwrap();
        let slots = self.policy.generate_slots(date, self.duration)?;
        let slots = provider.available_slots(date, slots).await?;
        Ok(
            slots
                .into_iter()
                .map(|slot| FreeSlot{
                    start: slot.interval().start,
                    end:   slot.interval().end,
                })
                .collect()
        )
    }
}
