use chrono::{Duration, NaiveTime};
use dotenv::dotenv;
use std::env;
use std::sync::Arc;
use teloxide::Bot;
use tokio::sync::Mutex;

use crate::dispatcher::UmdDispatcher;
use crate::domain::models::{ClosedRange, UserID};
use crate::domain::services::{
    FixedSlotsFactory, Mon2ThuAndFriWithLunchWorkingHoursPolicy, StandardDeadlinePolicy,
};
use crate::infra::{MockAdminProvider, PostgresRepository};
use crate::usecases::{App, CancelReservationUseCase, CheckDeadlineUseCase, CheckRegisteredUseCase, DaysWithFreeSlotsUseCase, FreeSlotsUseCase, GetUserUseCase, RegisterUserUseCase, ReserveSlotUseCase, ReservationsUseCase, UpdateUserUseCase, CheckAdminUseCase};
use crate::utils::postgres::pool;

mod bot;
mod dispatcher;
mod domain;
mod infra;
mod usecases;
mod utils;

#[tokio::main]
async fn main() {
    dotenv().ok();
    pretty_env_logger::init();

    let uri = env::var("DATABASE_URI").expect("DATABASE_URI must be set");
    let pool =
        pool::connect(&uri).expect(format!("unable to connect to database: {}", uri).as_str());
    log::info!("Connected to PostgreSQL database: {}", uri);

    let admin_ids_str = env::var("ADMIN_IDS").unwrap_or_default();
    let admin_ids: Vec<UserID> = admin_ids_str
        .split(",")
        .into_iter()
        .map(|s| UserID::new(s.parse::<i64>().expect("unable to parse user ids")))
        .collect();
        
    let admin_provider = Arc::new(MockAdminProvider::new(admin_ids));
    let slots_factory = Arc::new(FixedSlotsFactory::new(3, Duration::minutes(20)));
    let deadline_policy = Arc::new(StandardDeadlinePolicy::default());
    let working_hours_policy = Arc::new(Mon2ThuAndFriWithLunchWorkingHoursPolicy::new(
        ClosedRange {
            start: NaiveTime::from_hms_opt(10, 0, 0).unwrap(),
            end: NaiveTime::from_hms_opt(17, 0, 0).unwrap(),
        },
        ClosedRange {
            start: NaiveTime::from_hms_opt(12, 0, 0).unwrap(),
            end: NaiveTime::from_hms_opt(16, 0, 0).unwrap(),
        },
        ClosedRange {
            start: NaiveTime::from_hms_opt(12, 30, 0).unwrap(),
            end: NaiveTime::from_hms_opt(13, 30, 0).unwrap(),
        },
    ));
    let repos = Arc::new(PostgresRepository::new(pool));

    let app = App {
        cancel_reservation: CancelReservationUseCase::new(
            slots_factory.clone(),
            repos.clone(),
            repos.clone(),
        ),
        check_admin: CheckAdminUseCase::new(admin_provider.clone()),
        check_deadline: CheckDeadlineUseCase::new(deadline_policy.clone(), repos.clone()),
        check_registered: CheckRegisteredUseCase::new(repos.clone()),
        days_with_free_slots: DaysWithFreeSlotsUseCase::new(
            slots_factory.clone(),
            deadline_policy.clone(),
            working_hours_policy.clone(),
            repos.clone(),
            repos.clone(),
        ),
        free_slots: FreeSlotsUseCase::new(
            slots_factory.clone(),
            working_hours_policy.clone(),
            repos.clone(),
        ),
        get_user: GetUserUseCase::new(repos.clone()),
        register_user: RegisterUserUseCase::new(repos.clone()),
        reserve_slot: ReserveSlotUseCase::new(
            slots_factory.clone(),
            working_hours_policy.clone(),
            repos.clone(),
            repos.clone(),
            repos.clone(),
        ),
        slots: ReservationsUseCase::new(
            slots_factory.clone(),
            working_hours_policy.clone(),
            repos.clone(),
        ),
        update_user: UpdateUserUseCase::new(repos.clone(), repos.clone()),
    };

    let bot = Bot::from_env();
    let mut dispatcher = UmdDispatcher::create(bot, app).await;

    dispatcher.dispatch().await;
}
