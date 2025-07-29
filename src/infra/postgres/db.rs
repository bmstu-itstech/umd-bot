use chrono::{DateTime, NaiveDate, Utc};
use postgres_types::{FromSql, ToSql};
use tokio_postgres::{GenericClient, Row};

use crate::domain::Error;
use crate::domain::models::{
    Citizenship, OnlyCyrillic, OnlyLatin, Service as DomainService, Slot, User, UserID, Username,
};

pub struct RawUser {
    id: i64,
    username: String,
    full_name_lat: String,
    full_name_cyr: String,
    citizenship: String,
    arrival_date: NaiveDate,
}

impl From<&User> for RawUser {
    fn from(u: &User) -> Self {
        Self {
            id: u.id().as_i64(),
            username: u.username().as_str().to_string(),
            full_name_lat: u.full_name_lat().as_str().to_string(),
            full_name_cyr: u.full_name_cyr().as_str().to_string(),
            citizenship: u.citizenship().as_str().to_string(),
            arrival_date: u.arrival_date().clone(),
        }
    }
}

impl TryInto<User> for RawUser {
    type Error = Error;

    fn try_into(self) -> Result<User, Self::Error> {
        Ok(User::new(
            UserID::new(self.id),
            Username::new(self.username),
            OnlyLatin::new(self.full_name_lat)?,
            OnlyCyrillic::new(self.full_name_cyr)?,
            Citizenship::from(self.citizenship.as_str()),
            self.arrival_date,
        ))
    }
}

#[derive(Debug, ToSql, FromSql)]
#[postgres(name = "service", rename_all = "snake_case")]
enum Service {
    InitialRegistration,
    Visa,
    Insurance,
    VisaAndInsurance,
    RenewalOfRegistration,
    RenewalOfVisa,
    All,
}

impl Into<DomainService> for Service {
    fn into(self) -> DomainService {
        match self {
            Service::InitialRegistration => DomainService::InitialRegistration,
            Service::Visa => DomainService::Visa,
            Service::Insurance => DomainService::Insurance,
            Service::VisaAndInsurance => DomainService::VisaAndInsurance,
            Service::RenewalOfRegistration => DomainService::RenewalOfRegistration,
            Service::RenewalOfVisa => DomainService::RenewalOfVisa,
            Service::All => DomainService::All,
        }
    }
}

impl From<DomainService> for Service {
    fn from(s: DomainService) -> Self {
        match s {
            DomainService::InitialRegistration => Service::InitialRegistration,
            DomainService::Visa => Service::Visa,
            DomainService::Insurance => Service::Insurance,
            DomainService::VisaAndInsurance => Service::VisaAndInsurance,
            DomainService::RenewalOfRegistration => Service::RenewalOfRegistration,
            DomainService::RenewalOfVisa => Service::RenewalOfVisa,
            DomainService::All => Service::All,
        }
    }
}

pub struct RawReservation {
    slot_start: DateTime<Utc>,
    service: String,
    user_id: i64,
}

pub struct RawReservationWithUser {
    slot_start: DateTime<Utc>,
    service: Service,
    user: RawUser,
}

impl RawReservationWithUser {
    pub fn try_unpack(self) -> Result<(DateTime<Utc>, DomainService, User), Error> {
        Ok((self.slot_start, self.service.into(), self.user.try_into()?))
    }
}

pub async fn get_raw_user<C: GenericClient>(client: &C, id: UserID) -> Result<RawUser, Error> {
    let query = r#"
        SELECT
            id,
            username,
            full_name_lat,
            full_name_cyr,
            citizenship,
            arrival_date
        FROM users
        WHERE id = $1
    "#;

    let row_opt = client
        .query_opt(query, &[&id.as_i64()])
        .await
        .map_err(|err| Error::Other(err.into()))?;

    match row_opt {
        Some(row) => {
            let raw_user = fetch_raw_user(&row).map_err(|err| Error::Other(err.into()))?;
            Ok(raw_user)
        }
        None => Err(Error::UserNotFound(id)),
    }
}

pub async fn upsert_raw_user<C: GenericClient>(client: &C, user: RawUser) -> Result<(), Error> {
    client
        .execute(
            r#"
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
            ON CONFLICT (id) 
            DO UPDATE SET
                username      = EXCLUDED.username,
                full_name_lat = EXCLUDED.full_name_lat,
                full_name_cyr = EXCLUDED.full_name_cyr,
                citizenship   = EXCLUDED.citizenship,
                arrival_date  = EXCLUDED.arrival_date"#,
            &[
                &user.id,
                &user.username.as_str(),
                &user.full_name_lat.as_str(),
                &user.full_name_cyr.as_str(),
                &user.citizenship.as_str(),
                &user.arrival_date,
            ],
        )
        .await
        .map_err(|err| Error::Other(err.into()))?;
    Ok(())
}

pub async fn delete_reservations<C: GenericClient>(
    client: &C,
    slot_start: DateTime<Utc>,
) -> Result<(), Error> {
    client
        .execute(
            "DELETE FROM reservations WHERE slot_start = $1",
            &[&slot_start],
        )
        .await
        .map_err(|err| Error::Other(err.into()))?;
    Ok(())
}

pub async fn batch_insert_raw_reservations<C: GenericClient>(
    client: &C,
    reservations: &[RawReservation],
) -> Result<(), Error> {
    let stmt = client
        .prepare("INSERT INTO reservations (slot_start, service, user_id) VALUES ($1, $2, $3)")
        .await
        .map_err(|err| Error::Other(err.into()))?;
    for r in reservations {
        client
            .execute(&stmt, &[&r.slot_start, &r.service.as_str(), &r.user_id])
            .await
            .map_err(|err| Error::Other(err.into()))?;
    }
    Ok(())
}

pub async fn select_slot_raw_reservations_with_user<C: GenericClient>(
    client: &C,
    slot_start: DateTime<Utc>,
) -> Result<Vec<RawReservationWithUser>, Error> {
    let query = r#"
        SELECT
            r.service,
            u.id,
            u.username,
            u.full_name_lat,
            u.full_name_cyr,
            u.citizenship,
            u.arrival_date
        FROM reservations AS r
        LEFT JOIN users AS u
        WHERE
            r.start = $1
        LIMIT $2
    "#;

    let rows = client
        .query(query, &[&slot_start])
        .await
        .map_err(|err| Error::Other(err.into()))?;

    fetch_raw_reservations_with_user(&rows)
}

pub async fn select_raw_reservations_with_user<C: GenericClient>(
    client: &C,
    starts: &[DateTime<Utc>],
) -> Result<Vec<RawReservationWithUser>, Error> {
    let working_slots_start: Vec<String> = starts
        .iter()
        .map(|dt| format!("(TIMESTAMPTZ '{}')", dt.to_string()))
        .collect();

    let query = format!(
        r#"
            WITH working_slots (start) AS (
                SELECT *
                FROM (VALUES {})
                AS t(start)
            )
            SELECT
                r.slot_start,
                r.service,
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

    fetch_raw_reservations_with_user(&rows)
}

pub async fn select_available_raw_reservations_with_user<C: GenericClient>(
    client: &C,
    starts: &[DateTime<Utc>],
) -> Result<Vec<RawReservationWithUser>, Error> {
    let working_slots_start: Vec<String> = starts
        .iter()
        .map(|dt| format!("(TIMESTAMPTZ '{}')", dt.to_string()))
        .collect();

    let query = format!(
        r#"
        WITH working_slots (start) AS (
               SELECT *
               FROM (VALUES {})
               AS t(start)
           )
           SELECT
               r.slot_start,
               r.service,
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

    fetch_raw_reservations_with_user(&rows)
}

pub async fn has_available_slots<C: GenericClient>(
    client: &C,
    starts: &[DateTime<Utc>],
    max_size: i64,
) -> Result<bool, Error> {
    let working_slots_start: Vec<String> = starts
        .iter()
        .map(|dt| format!("(TIMESTAMPTZ '{}')", dt.to_string()))
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

    let row = client
        .query_one(&query, &[&max_size])
        .await
        .map_err(|err| Error::Other(err.into()))?;
    let exists: bool = row.get(0);
    Ok(exists)
}

pub fn fetch_raw_user(row: &Row) -> Result<RawUser, tokio_postgres::Error> {
    Ok(RawUser {
        id: row.try_get("id")?,
        username: row.try_get("username")?,
        full_name_lat: row.try_get("full_name_lat")?,
        full_name_cyr: row.try_get("full_name_cyr")?,
        citizenship: row.try_get("citizenship")?,
        arrival_date: row.try_get("arrival_date")?,
    })
}

pub fn fetch_raw_reservation(row: &Row) -> Result<RawReservation, tokio_postgres::Error> {
    Ok(RawReservation {
        slot_start: row.try_get("slot_start")?,
        service: row.try_get("service")?,
        user_id: row.try_get("user_id")?,
    })
}

pub fn fetch_raw_reservation_with_user(
    row: &Row,
) -> Result<RawReservationWithUser, tokio_postgres::Error> {
    Ok(RawReservationWithUser {
        slot_start: row.try_get("slot_start")?,
        service: row.try_get("service")?,
        user: fetch_raw_user(row)?,
    })
}

pub fn fetch_raw_reservations_with_user(
    rows: &[Row],
) -> Result<Vec<RawReservationWithUser>, Error> {
    rows.iter()
        .map(|row| fetch_raw_reservation_with_user(row))
        .collect::<Result<Vec<RawReservationWithUser>, _>>()
        .map_err(|err| Error::Other(err.into()))
}

pub fn slot_to_raw_reservations(slot: &Slot) -> Vec<RawReservation> {
    slot.reservations()
        .iter()
        .map(|r| RawReservation {
            slot_start: slot.start(),
            service: r.service().clone().into(),
            user_id: r.by().id().as_i64(),
        })
        .collect()
}
