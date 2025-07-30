use crate::usecases::{CancelReservationUseCase, CheckDeadlineUseCase, CheckRegisteredUseCase, DaysWithFreeSlotsUseCase, FreeSlotsUseCase, GetUserUseCase, RegisterUserUseCase, ReserveSlotUseCase, ReservationsUseCase, UpdateUserUseCase, CheckAdminUseCase};

pub struct App {
    pub cancel_reservation: CancelReservationUseCase,
    pub check_admin: CheckAdminUseCase,
    pub check_deadline: CheckDeadlineUseCase,
    pub check_registered: CheckRegisteredUseCase,
    pub days_with_free_slots: DaysWithFreeSlotsUseCase,
    pub free_slots: FreeSlotsUseCase,
    pub get_user: GetUserUseCase,
    pub register_user: RegisterUserUseCase,
    pub reserve_slot: ReserveSlotUseCase,
    pub slots: ReservationsUseCase,
    pub update_user: UpdateUserUseCase,
}
