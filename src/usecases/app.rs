use crate::usecases::{
    CancelReservationUseCase, CheckDeadlineUseCase, CheckRegisteredUseCase,
    DaysWithFreeSlotsUseCase, FreeSlotsUseCase, GetUserUseCase, RegisterUserUseCase,
    ReserveSlotUseCase, SlotsUseCase, UpdateUserUseCase,
};

pub struct App {
    pub cancel_reservation: CancelReservationUseCase,
    pub check_deadline: CheckDeadlineUseCase,
    pub check_registered: CheckRegisteredUseCase,
    pub days_with_free_slots: DaysWithFreeSlotsUseCase,
    pub free_slots: FreeSlotsUseCase,
    pub get_user: GetUserUseCase,
    pub register_user: RegisterUserUseCase,
    pub reserve_slot: ReserveSlotUseCase,
    pub slots: SlotsUseCase,
    pub update_user: UpdateUserUseCase,
}
