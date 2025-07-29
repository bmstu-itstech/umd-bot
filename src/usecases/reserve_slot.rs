use chrono::{DateTime, Duration, Utc};
use std::sync::Arc;

use crate::domain::Error;
use crate::domain::interfaces::{AvailableSlotsProvider, SlotsRepository, UserProvider};
use crate::domain::models::{Service, UserID};
use crate::domain::services::WorkingHoursPolicy;

#[derive(Clone)]
pub struct ReserveSlotUseCase<const N: usize, WP>
where
    WP: WorkingHoursPolicy,
{
    duration: Duration,
    policy: WP,
    user_provider: Arc<dyn UserProvider>,
    provider: Arc<dyn AvailableSlotsProvider<N>>,
    repos: Arc<dyn SlotsRepository<N>>,
}

impl<const N: usize, WP> ReserveSlotUseCase<N, WP>
where
    WP: WorkingHoursPolicy,
{
    pub fn new(
        duration: Duration,
        policy: WP,
        user_provider: Arc<dyn UserProvider>,
        provider: Arc<dyn AvailableSlotsProvider<N>>,
        repos: Arc<dyn SlotsRepository<N>>,
    ) -> Self {
        Self {
            duration,
            policy,
            user_provider,
            provider,
            repos,
        }
    }

    pub async fn reserve_slot(
        &self,
        user_id: UserID,
        time: DateTime<Utc>,
        service: Service,
    ) -> Result<(), Error> {
        let user = self.user_provider.user(user_id).await?;

        let date = time.date_naive();
        let slots = self.policy.generate_slots(date, self.duration)?;
        let mut slots = self.provider.available_slots(slots).await?;
        let res = slots.iter_mut().find(|slot| slot.interval().start == time);

        let slot = match res {
            Some(slot) => slot,
            None => return Err(Error::SlotNotFoundError),
        };

        slot.reserve(user, service)?;
        self.repos.save_slot(slot).await?;
        Ok(())
    }
}
