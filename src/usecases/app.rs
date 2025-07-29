use crate::domain::services::{DeadlinePolicy, WorkingHoursPolicy};
use crate::usecases::{
    CancelReservationUseCase, CheckDeadlineUseCase, CheckRegisteredUseCase,
    DaysWithFreeSlotsUseCase, FreeSlotsUseCase, GetUserUseCase, RegisterUserUseCase,
    ReserveSlotUseCase, SlotsUseCase, UpdateUserUseCase,
};

pub struct App<const N: usize, DP, WP>
where
    DP: DeadlinePolicy + Send + Sync + 'static,
    WP: WorkingHoursPolicy + Send + Sync + 'static,
{
    pub cancel_reservation: CancelReservationUseCase<N>,
    pub check_deadline: CheckDeadlineUseCase<DP>,
    pub check_registered: CheckRegisteredUseCase,
    pub days_with_free_slots: DaysWithFreeSlotsUseCase<N, DP, WP>,
    pub free_slots: FreeSlotsUseCase<N, WP>,
    pub get_user: GetUserUseCase,
    pub register_user: RegisterUserUseCase,
    pub reserve_slot: ReserveSlotUseCase<N, WP>,
    pub slots: SlotsUseCase<N, WP>,
    pub update_user: UpdateUserUseCase,
}
