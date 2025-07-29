use teloxide::dispatching::dialogue::InMemStorage;
use teloxide::dispatching::{DefaultKey, UpdateHandler};
use teloxide::dptree::entry;
use teloxide::prelude::Dispatcher;
use teloxide::{Bot, dptree};

use crate::bot::handlers::user::{
    RegistrationState, UpdateState, registration_schema, update_schema, view_schema,
};
use crate::domain::Error;
use crate::domain::services::{DeadlinePolicy, WorkingHoursPolicy};
use crate::usecases::App;

pub struct UmdDispatcher;

impl UmdDispatcher {
    pub async fn create<const N: usize, DP, WP>(
        bot: Bot,
        app: App<N, DP, WP>,
    ) -> Dispatcher<Bot, Error, DefaultKey>
    where
        DP: DeadlinePolicy + Send + Sync + 'static,
        WP: WorkingHoursPolicy + Send + Sync + 'static,
    {
        Dispatcher::builder(bot, Self::scheme())
            .dependencies(dptree::deps![
                app.cancel_reservation,
                app.check_deadline,
                app.check_registered,
                app.days_with_free_slots,
                app.free_slots,
                app.get_user,
                app.register_user,
                app.reserve_slot,
                app.slots,
                app.update_user,
                InMemStorage::<RegistrationState>::new(),
                InMemStorage::<UpdateState>::new()
            ])
            .default_handler(|upd| async move {
                log::warn!("Unhandled update: {:?}", upd);
            })
            .enable_ctrlc_handler()
            .build()
    }

    fn scheme() -> UpdateHandler<Error> {
        entry()
            .branch(registration_schema())
            .branch(update_schema())
            .branch(view_schema())
    }
}
