use async_trait::async_trait;
use chrono::{DateTime, Utc};
use deadpool_postgres::Pool;
use std::collections::HashMap;
use std::pin::Pin;
use tokio_postgres::{Client, GenericClient, Transaction};

use crate::domain::Error;
use crate::domain::interfaces::{
    AvailableSlotsProvider, HasAvailableSlotsProvider, ReservedSlotProvider, ReservedSlotsProvider,
    SlotsRepository, UserProvider, UserRepository,
};
use crate::domain::models::{Slot, User, UserID};
use crate::infra::postgres::db::{
    batch_insert_raw_reservations, delete_reservations, get_raw_user, has_available_slots,
    select_raw_reservations_with_user, select_slot_raw_reservations_with_user,
    slot_to_raw_reservations, upsert_raw_user,
};
use crate::{with_client, with_transaction};

pub struct PostgresRepository {
    pool: Pool,
}

impl PostgresRepository {
    pub fn new(pool: Pool) -> PostgresRepository {
        PostgresRepository { pool }
    }

    pub async fn with_client<F, R>(&self, f: F) -> Result<R, Error>
    where
        F: FnOnce(&Client) -> Pin<Box<dyn Future<Output = Result<R, Error>> + '_>>,
    {
        let obj = self
            .pool
            .get()
            .await
            .map_err(|err| Error::Other(err.into()))?;
        let client = obj.client();
        f(client).await
    }
}

#[async_trait]
impl<const N: usize> HasAvailableSlotsProvider<N> for PostgresRepository {
    async fn has_available_slots(&self, slots: &[Slot<N>]) -> Result<bool, Error> {
        let starts: Vec<_> = slots.iter().map(|slot| slot.start()).collect();
        let n = N as i64;
        with_client!(self.pool, async |client: &Client| {
            has_available_slots(client, &starts, n).await
        })
    }
}

#[async_trait]
impl<const N: usize> AvailableSlotsProvider<N> for PostgresRepository {
    async fn available_slots(&self, slots: Vec<Slot<N>>) -> Result<Vec<Slot<N>>, Error> {
        let starts: Vec<DateTime<Utc>> = slots.iter().map(|slot| slot.start()).collect();
        let mut slots: HashMap<_, _> =
            HashMap::from_iter(slots.into_iter().map(|slot| (slot.start(), slot)));

        with_client!(self.pool, async |client: &Client| {
            let rs = select_raw_reservations_with_user(client, &starts).await?;

            for r in rs {
                let (start, service, user) = r.try_unpack()?;
                let slot = slots.get_mut(&start).unwrap();
                slot.reserve(user, service)?;
            }

            let reserved_slots = slots
                .into_values()
                .filter(|slot| slot.is_available())
                .collect();

            Ok(reserved_slots)
        })
    }
}

#[async_trait]
impl<const N: usize> ReservedSlotsProvider<N> for PostgresRepository {
    async fn reserved_slots(&self, slots: Vec<Slot<N>>) -> Result<Vec<Slot<N>>, Error> {
        let starts: Vec<DateTime<Utc>> = slots.iter().map(|slot| slot.start()).collect();
        let mut slots: HashMap<_, _> =
            HashMap::from_iter(slots.into_iter().map(|slot| (slot.start(), slot)));

        with_client!(self.pool, async |client: &Client| {
            let rs = select_raw_reservations_with_user(client, &starts).await?;

            for r in rs {
                let (start, service, user) = r.try_unpack()?;
                let slot = slots.get_mut(&start).unwrap();
                slot.reserve(user, service)?;
            }

            let reserved_slots = slots
                .into_values()
                .filter(|slot| !slot.is_empty())
                .collect();

            Ok(reserved_slots)
        })
    }
}

#[async_trait]
impl<const N: usize> ReservedSlotProvider<N> for PostgresRepository {
    async fn reserved_slot(&self, mut slot: Slot<N>) -> Result<Slot<N>, Error> {
        with_client!(self.pool, async |client| {
            let raw = select_slot_raw_reservations_with_user(client, slot.start()).await?;
            for r in raw {
                let (_, service, user) = r.try_unpack()?;
                slot.reserve(user, service)?;
            }
            Ok(slot)
        })
    }
}

#[async_trait]
impl<const N: usize> SlotsRepository<N> for PostgresRepository {
    async fn save_slot(&self, slot: &Slot<N>) -> Result<(), Error> {
        with_transaction!(self.pool, async |tx: &Transaction| {
            delete_reservations(tx, slot.start()).await?;
            let raw_reservations = slot_to_raw_reservations(&slot);
            batch_insert_raw_reservations(tx, &raw_reservations).await?;
            Ok::<_, Error>(())
        })
    }
}

#[async_trait]
impl UserRepository for PostgresRepository {
    async fn save_user(&self, user: User) -> Result<(), Error> {
        with_client!(self.pool, async |client| {
            let raw_user = (&user).into();
            upsert_raw_user(client, raw_user).await?;
            Ok(())
        })
    }
}

#[async_trait]
impl UserProvider for PostgresRepository {
    async fn user(&self, id: UserID) -> Result<User, Error> {
        with_client!(self.pool, async |client| {
            let raw_user = get_raw_user(client, id).await?;
            raw_user.try_into()
        })
    }
}

#[cfg(test)]
mod test_utils {
    use crate::domain::models::{ClosedRange, Slot};
    use chrono::{Duration, NaiveDate, NaiveTime};
    use deadpool_postgres::Pool;

    pub async fn create_slot_hm<const N: usize>(
        date: NaiveDate,
        start_h: u32,
        start_m: u32,
        dur_m: u32,
    ) -> Slot<N> {
        let start = date
            .and_time(NaiveTime::from_hms_opt(start_h, start_m, 0).unwrap())
            .and_utc();
        Slot::empty(ClosedRange {
            start,
            end: start + Duration::minutes(dur_m as i64),
        })
    }

    pub async fn setup_db(pool: &Pool) -> Result<(), tokio_postgres::Error> {
        let client = pool
            .get()
            .await
            .expect("error creating Postgres client from pool");

        let fixtures_sql = include_str!("./fixtures/test_slots.sql");
        client.batch_execute(fixtures_sql).await?;

        Ok(())
    }
}

#[cfg(test)]
mod has_available_slots_tests {
    use super::test_utils::*;
    use super::*;
    use crate::utils::postgres::testing::test_db_setup;
    use chrono::NaiveDate;

    #[tokio::test]
    async fn test_has_available_slots() {
        let pool = test_db_setup().await;
        setup_db(&pool).await.unwrap();
        let repo = PostgresRepository { pool };

        let date = NaiveDate::from_ymd_opt(2025, 7, 14).unwrap();
        let slots: Vec<Slot<3>> = vec![
            create_slot_hm(date, 9, 0, 20).await,
            create_slot_hm(date, 9, 20, 20).await,
            create_slot_hm(date, 9, 40, 20).await,
        ];

        let res = repo.has_available_slots(&slots).await;

        assert!(res.is_ok(), "{}", res.err().unwrap());
        let available_slots = res.unwrap();
        assert!(available_slots);
    }

    #[tokio::test]
    async fn test_has_no_available_slots() {
        let pool = test_db_setup().await;
        setup_db(&pool).await.unwrap();
        let repo = PostgresRepository { pool };

        let date = NaiveDate::from_ymd_opt(2025, 7, 14).unwrap();
        let slots: Vec<Slot<2>> = vec![
            create_slot_hm(date, 9, 0, 20).await,
            create_slot_hm(date, 9, 20, 20).await,
        ];

        let res = repo.has_available_slots(&slots).await;

        assert!(res.is_ok(), "{}", res.err().unwrap());
        let available_slots = res.unwrap();
        assert!(!available_slots);
    }
}

#[cfg(test)]
mod available_slots_tests {
    use super::test_utils::*;
    use super::*;
    use crate::utils::postgres::testing::test_db_setup;
    use chrono::NaiveDate;

    #[tokio::test]
    async fn test_one_available_slot() {
        let pool = test_db_setup().await;
        setup_db(&pool).await.unwrap();
        let repo = PostgresRepository { pool };

        let date = NaiveDate::from_ymd_opt(2025, 7, 14).unwrap();
        let slots: Vec<Slot<3>> = vec![
            create_slot_hm(date, 9, 0, 0).await,
            create_slot_hm(date, 9, 20, 0).await,
        ];

        let res = repo.available_slots(slots).await;

        assert!(res.is_ok(), "{}", res.err().unwrap());
        let slots = res.unwrap();
        assert_eq!(slots.len(), 1);
        let slot = &slots[0];
        assert_eq!(slot.reserved(), 2);
    }

    #[tokio::test]
    async fn test_empty_available_slots() {
        let pool = test_db_setup().await;
        setup_db(&pool).await.unwrap();
        let repo = PostgresRepository { pool };

        let date = NaiveDate::from_ymd_opt(2025, 7, 14).unwrap();
        let slots: Vec<Slot<3>> = vec![create_slot_hm(date, 9, 0, 0).await];

        let res = repo.available_slots(slots).await;

        assert!(res.is_ok(), "{}", res.err().unwrap());
        let slots = res.unwrap();
        assert!(slots.is_empty());
    }

    #[tokio::test]
    async fn test_available_slots() {
        let pool = test_db_setup().await;
        setup_db(&pool).await.unwrap();
        let repo = PostgresRepository { pool };

        let date = NaiveDate::from_ymd_opt(2025, 7, 14).unwrap();
        let slots: Vec<Slot<3>> = vec![
            create_slot_hm(date, 9, 0, 0).await,
            create_slot_hm(date, 9, 20, 0).await,
            create_slot_hm(date, 9, 40, 0).await,
        ];

        let res = repo.available_slots(slots).await;

        assert!(res.is_ok(), "{}", res.err().unwrap());
        let slots = res.unwrap();
        assert_eq!(slots.len(), 2);

        let free_places = slots.iter().fold(0, |acc, slot| acc + 3 - slot.reserved());
        assert_eq!(free_places, 4);
    }
}

#[cfg(test)]
mod reserved_slots_tests {
    use super::test_utils::*;
    use super::*;
    use crate::utils::postgres::testing::test_db_setup;
    use chrono::NaiveDate;

    #[tokio::test]
    async fn test_no_reserved_slots() {
        let pool = test_db_setup().await;
        setup_db(&pool).await.unwrap();
        let repo = PostgresRepository { pool };

        let date = NaiveDate::from_ymd_opt(2025, 7, 14).unwrap();
        let slots: Vec<Slot<3>> = vec![
            create_slot_hm(date, 9, 40, 0).await,
            create_slot_hm(date, 10, 00, 0).await,
        ];

        let res = repo.reserved_slots(slots).await;

        assert!(res.is_ok(), "{}", res.err().unwrap());
        let slots = res.unwrap();
        assert!(slots.is_empty());
    }

    #[tokio::test]
    async fn test_one_reserved_slot() {
        let pool = test_db_setup().await;
        setup_db(&pool).await.unwrap();
        let repo = PostgresRepository { pool };

        let date = NaiveDate::from_ymd_opt(2025, 7, 14).unwrap();
        let slots: Vec<Slot<3>> = vec![create_slot_hm(date, 9, 20, 0).await];

        let res = repo.reserved_slots(slots).await;

        assert!(res.is_ok(), "{}", res.err().unwrap());
        let slots = res.unwrap();
        assert_eq!(slots.len(), 1);
        let slot = &slots[0];
        assert_eq!(slot.reserved(), 2);
    }

    #[tokio::test]
    async fn test_reserved_slots() {
        let pool = test_db_setup().await;
        setup_db(&pool).await.unwrap();
        let repo = PostgresRepository { pool };

        let date = NaiveDate::from_ymd_opt(2025, 7, 14).unwrap();
        let slots: Vec<Slot<3>> = vec![
            create_slot_hm(date, 9, 00, 0).await,
            create_slot_hm(date, 9, 20, 0).await,
        ];

        let res = repo.reserved_slots(slots).await;

        assert!(res.is_ok(), "{}", res.err().unwrap());
        let slots = res.unwrap();
        assert_eq!(slots.len(), 2);
    }
}
