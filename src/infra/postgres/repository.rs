use std::collections::HashMap;
use async_trait::async_trait;
use chrono::{DateTime, NaiveDate, Utc};
use deadpool_postgres::{GenericClient, Pool};
use tokio_postgres::Row;

use crate::domain::Error;
use crate::domain::interfaces::{AvailableSlotsProvider, HasAvailableSlotsProvider, ReservedSlotsProvider, SlotsRepository, UserRepository};
use crate::domain::models::{Citizenship, OnlyCyrillic, OnlyLatin, Slot, TelegramID, TelegramUsername, User};

const TIMESTAMP_FORMAT: &'static str = "%Y-%m-%d %H:%M:%S";


pub struct PostgresRepository {
    pool: Pool,
}


#[derive(Debug)]
pub struct RawUser {
    id:            i64,
    username:      String,
    full_name_lat: String,
    full_name_cyr: String,
    citizenship:   String,
    arrival_date:  NaiveDate,
}


#[derive(Debug)]
pub struct RawReservationWithUser {
    start: DateTime<Utc>,
    user:  RawUser,
}


#[async_trait]
impl<const N: usize> HasAvailableSlotsProvider<N> for PostgresRepository {
    async fn has_available_slots(&self, slots: &[Slot<N>]) -> Result<bool, Error> {
        let client = self.pool
            .get()
            .await
            .map_err(|err| Error::Other(err.into()))?;

        let working_slots_start: Vec<String> = slots
            .iter()
            .map(|slot| 
                format!("(TIMESTAMPTZ '{}')", slot.start().format(TIMESTAMP_FORMAT))
            )
            .collect();

        let query = format!(
            r#"
            SELECT EXISTS (
                WITH working_slots (start) AS (
                    SELECT *
                    FROM (VALUES {})
                    AS t(start)
                ),
                occupied_slots AS (
                    SELECT
                        ws.start,
                        COUNT(r.user_id) AS user_count
                    FROM working_slots AS ws
                    LEFT JOIN
                        reservations AS r
                        ON r.slot_start = ws.start
                    GROUP BY ws.start
                )
                SELECT 1
                FROM occupied_slots AS os
                WHERE
                    os.user_count < $1
                LIMIT 1
            );
            "#, 
            working_slots_start.join(", ")
        );
        
        let n = N as i64;
        let row = client
            .query_one(&query, &[&n])
            .await
            .map_err(|err| Error::Other(err.into()))?;
        let exists: bool = row.get(0);
        
        Ok(exists)
    }
}


#[async_trait]
impl<const N: usize> AvailableSlotsProvider<N> for PostgresRepository {
    async fn available_slots(&self, slots: Vec<Slot<N>>) -> Result<Vec<Slot<N>>, Error> {
        let client = self.pool
            .get()
            .await
            .map_err(|err| Error::Other(err.into()))?;

        let working_slots_start: Vec<String> = slots
            .iter()
            .map(|slot|
                format!("(TIMESTAMPTZ '{}')", slot.start().format(TIMESTAMP_FORMAT))
            )
            .collect();

        let query = format!(
            r#"
            WITH working_slots (start) AS (
                SELECT *
                FROM (VALUES {})
                AS t(start)
            )
            SELECT
                ws.start,
                u.*
            FROM working_slots AS ws
            LEFT JOIN
                reservations AS r
                ON ws.start = r.slot_start
            INNER JOIN
                users AS u
                ON u.id = r.user_id
            "#,
            working_slots_start.join(", ")
        );

        let rows = client
            .query(&query, &[])
            .await
            .map_err(|err| Error::Other(err.into()))?;

        let reservation = rows
            .into_iter()
            .map(|row| RawReservationWithUser::try_from(row))
            .collect::<Result<Vec<RawReservationWithUser>, _>>()
            .map_err(|err| Error::Other(err.into()))?;

        let mut slots: HashMap<_, _> = HashMap::from_iter(slots
            .into_iter()
            .map(|slot| (slot.start(), slot))
        );

        for reservation in reservation.into_iter() {
            let user = reservation.user.try_into()?;
            let slot = slots.get_mut(&reservation.start).unwrap();
            slot.reserve(&user)?;
        }

        let available_slots = slots
            .into_values()
            .filter(|slot| slot.is_available())
            .collect();

        Ok(available_slots)
    }
}


#[async_trait]
impl<const N: usize> ReservedSlotsProvider<N> for PostgresRepository {
    async fn reserved_slots(&self, slots: Vec<Slot<N>>) -> Result<Vec<Slot<N>>, Error> {
        let client = self.pool
            .get()
            .await
            .map_err(|err| Error::Other(err.into()))?;

        let working_slots_start: Vec<String> = slots
            .iter()
            .map(|slot|
                format!("(TIMESTAMPTZ '{}')", slot.start().format(TIMESTAMP_FORMAT))
            )
            .collect();

        let query = format!(
            r#"
            WITH working_slots (start) AS (
                SELECT *
                FROM (VALUES {})
                AS t(start)
            )
            SELECT
                ws.start,
                u.*
            FROM working_slots AS ws
            LEFT JOIN
                reservations AS r
                ON ws.start = r.slot_start
            INNER JOIN
                users AS u
                ON u.id = r.user_id
            "#,
            working_slots_start.join(", ")
        );

        let rows = client
            .query(&query, &[])
            .await
            .map_err(|err| Error::Other(err.into()))?;

        let reservation = rows
            .into_iter()
            .map(|row| RawReservationWithUser::try_from(row))
            .collect::<Result<Vec<RawReservationWithUser>, _>>()
            .map_err(|err| Error::Other(err.into()))?;

        let mut slots: HashMap<_, _> = HashMap::from_iter(slots
            .into_iter()
            .map(|slot| (slot.start(), slot))
        );

        for reservation in reservation.into_iter() {
            let user = reservation.user.try_into()?;
            let slot = slots.get_mut(&reservation.start).unwrap();
            slot.reserve(&user)?;
        }

        let reserved_slots = slots
            .into_values()
            .filter(|slot| !slot.is_empty())
            .collect();

        Ok(reserved_slots)
    }
}


#[async_trait]
impl<const N: usize> SlotsRepository<N> for PostgresRepository {
    async fn save_slot(&self, slot: &Slot<N>) -> Result<(), Error> {
        let mut client = self.pool
            .get()
            .await
            .map_err(|err| Error::Other(err.into()))?;

        let transaction = client
            .transaction()
            .await
            .map_err(|err| Error::Other(err.into()))?;

        let slot_start = slot.start();

        transaction.execute(r#"
            DELETE FROM 
                reservations 
            WHERE slot_start = $1"#,
            &[&slot_start]
        )
            .await
            .map_err(|err| Error::Other(err.into()))?;

        for user in slot.reserved_by() {
            transaction.execute(r#"
                INSERT INTO reservations (
                    slot_start,
                    user_id
                ) VALUES 
                    ($1, $2)"#,
                &[&slot_start, &user.id().as_i64()]
            )
                .await
                .map_err(|err| Error::Other(err.into()))?;
        }

        transaction
            .commit()
            .await
            .map_err(|err| Error::Other(err.into()))?;

        Ok(())
    }
}


#[async_trait]
impl UserRepository for PostgresRepository {
    async fn save_user(&self, user: &User) -> Result<(), Error> {
        let client = self.pool
            .get()
            .await
            .map_err(|err| Error::Other(err.into()))?;

        let citizenship: String = user.citizenship().clone().into();
        client.execute(r#"
            INSERT INTO users (
                id,
                username,
                full_name_lat,
                full_name_cyr,
                citizenship,
                arrival_date
            )
            VALUES
                ($1, $2, $3, $4, $5, $6)
            ON CONFLICT (id) DO UPDATE SET
                username = EXCLUDED.username,
                full_name_lat = EXCLUDED.full_name_lat,
                full_name_cyr = EXCLUDED.full_name_cyr,
                citizenship = EXCLUDED.citizenship,
                arrival_date = EXCLUDED.arrival_date"#,
            &[
                &user.id().as_i64(),
                &user.username().as_str(),
                &user.full_name_lat().as_str(),
                &user.full_name_cyr().as_str(),
                &citizenship.as_str(),
                &user.arrival_date(),
            ],
        )
            .await
            .map_err(|err| Error::Other(err.into()))?;

        Ok(())
    }
}


impl TryFrom<Row> for RawReservationWithUser {
    type Error = tokio_postgres::Error;

    fn try_from(row: Row) -> Result<Self, Self::Error> {
        Ok(RawReservationWithUser {
            start: row.try_get("start")?,
            user:  RawUser::try_from(row)?,
        })
    }
}


impl TryFrom<Row> for RawUser {
    type Error = tokio_postgres::Error;

    fn try_from(row: Row) -> Result<Self, Self::Error> {
        Ok(RawUser{
            id:            row.try_get("id")?,
            username:      row.try_get("username")?,
            full_name_lat: row.try_get("full_name_lat")?,
            full_name_cyr: row.try_get("full_name_cyr")?,
            citizenship:   row.try_get("citizenship")?,
            arrival_date:  row.try_get("arrival_date")?,
        })
    }
}


impl TryInto<User> for RawUser {
    type Error = Error;

    fn try_into(self) -> Result<User, Self::Error> {
        Ok(User::new(
            TelegramID::new(self.id),
            TelegramUsername::new(self.username),
            OnlyLatin::new(self.full_name_lat)?,
            OnlyCyrillic::new(self.full_name_cyr)?,
            self.citizenship.into(),
            self.arrival_date,
        ))
    }
}


impl From<String> for Citizenship {
    fn from(s: String) -> Self {
        match s.as_str() {
            "Tajikistan" => Citizenship::Tajikistan,
            "Uzbekistan" => Citizenship::Uzbekistan,
            "Kazakhstan" => Citizenship::Kazakhstan,
            "Kyrgyzstan" => Citizenship::Kyrgyzstan,
            "Armenia"    => Citizenship::Armenia,
            "Belarus"    => Citizenship::Belarus,
            "Ukraine"    => Citizenship::Ukraine,
            _            => Citizenship::Other(s.into()),
        }
    }
}


impl Into<String> for Citizenship {
    fn into(self) -> String {
        match self {
            Citizenship::Tajikistan => "Tajikistan".into(),
            Citizenship::Uzbekistan => "Uzbekistan".into(),
            Citizenship::Kazakhstan => "Kazakhstan".into(),
            Citizenship::Kyrgyzstan => "Kyrgyzstan".into(),
            Citizenship::Armenia => "Armenia".into(),
            Citizenship::Belarus => "Belarus".into(),
            Citizenship::Ukraine => "Ukraine".into(),
            Citizenship::Other(s) => s.into(),
        }
    }
}


#[cfg(test)]
mod test_utils {
    use chrono::{Duration, NaiveDate, NaiveTime};
    use deadpool_postgres::Pool;
    use crate::domain::models::{ClosedRange, Slot};

    pub async fn create_slot_hm<const N: usize>(date: NaiveDate, start_h: u32, start_m: u32, dur_m: u32) -> Slot<N> {
        let start = date.and_time(
            NaiveTime::from_hms_opt(start_h, start_m, 0).unwrap()
        ).and_utc();
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
    use crate::utils::postgres::testing::test_db_setup;
    use super::test_utils::*;
    use super::*;


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

        let res = repo
            .has_available_slots(&slots)
            .await;

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

        let res = repo
            .has_available_slots(&slots)
            .await;

        assert!(res.is_ok(), "{}", res.err().unwrap());
        let available_slots = res.unwrap();
        assert!(!available_slots);
    }
}


#[cfg(test)]
mod available_slots_tests {
    use crate::utils::postgres::testing::test_db_setup;
    use super::test_utils::*;
    use super::*;

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

        let res = repo
            .available_slots(slots)
            .await;

        assert!(res.is_ok(), "{}", res.err().unwrap());
        let slots = res.unwrap();
        assert_eq!(slots.len(), 1);
        let slot = &slots[0];
        assert_eq!(slot.reserved_by().len(), 2);
    }

    #[tokio::test]
    async fn test_empty_available_slots() {
        let pool = test_db_setup().await;
        setup_db(&pool).await.unwrap();
        let repo = PostgresRepository { pool };

        let date = NaiveDate::from_ymd_opt(2025, 7, 14).unwrap();
        let slots: Vec<Slot<3>> = vec![
            create_slot_hm(date, 9, 0, 0).await,
        ];

        let res = repo
            .available_slots(slots)
            .await;

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

        let res = repo
            .available_slots(slots)
            .await;

        assert!(res.is_ok(), "{}", res.err().unwrap());
        let slots = res.unwrap();
        assert_eq!(slots.len(), 2);

        let free_places = slots
            .iter()
            .fold(0, |acc, slot| acc + 3 - slot.reserved_by().len());
        assert_eq!(free_places, 4);
    }
}


#[cfg(test)]
mod reserved_slots_tests {
    use crate::utils::postgres::testing::test_db_setup;
    use super::test_utils::*;
    use super::*;

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

        let res = repo
            .reserved_slots(slots)
            .await;

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
        let slots: Vec<Slot<3>> = vec![
            create_slot_hm(date, 9, 20, 0).await,
        ];

        let res = repo
            .reserved_slots(slots)
            .await;

        assert!(res.is_ok(), "{}", res.err().unwrap());
        let slots = res.unwrap();
        assert_eq!(slots.len(), 1);
        let slot = &slots[0];
        assert_eq!(slot.reserved_by().len(), 2);
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

        let res = repo
            .reserved_slots(slots)
            .await;

        assert!(res.is_ok(), "{}", res.err().unwrap());
        let slots = res.unwrap();
        assert_eq!(slots.len(), 2);
    }
}
