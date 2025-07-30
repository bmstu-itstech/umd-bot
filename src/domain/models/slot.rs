use chrono::{DateTime, Utc};

use crate::domain::Error;
use crate::domain::models::reservation::Reservation;
use crate::domain::models::{ClosedRange, Service, User, UserID};

#[derive(Debug, Clone)]
pub struct Slot {
    interval: ClosedRange<DateTime<Utc>>,
    reservations: Vec<Reservation>,
    max_size: usize,
}

impl Slot {
    pub fn empty(interval: ClosedRange<DateTime<Utc>>, size: usize) -> Self {
        Self {
            interval,
            reservations: Vec::with_capacity(size),
            max_size: size,
        }
    }

    pub fn restore(
        interval: ClosedRange<DateTime<Utc>>,
        reservations: &[Reservation],
        max_size: usize,
    ) -> Result<Self, Error> {
        if reservations.len() > max_size {
            return Err(Error::MaxCapacityExceeded(max_size));
        }

        Ok(Self {
            interval,
            reservations: Vec::from(reservations),
            max_size,
        })
    }

    pub fn reserve(&mut self, user: User, service: Service) -> Result<(), Error> {
        if self.reservations.len() >= self.max_size {
            return Err(Error::MaxCapacityExceeded(self.max_size));
        }
        if self.reservations.iter().find(|r| r.by().id() == user.id()).is_some() {
            return Err(Error::SlotAlreadyReserved(user.id()));
        }
        self.reservations.push(Reservation::new(user, service));
        Ok(())
    }

    pub fn cancel(&mut self, id: UserID) -> Result<(), Error> {
        let idx = self
            .reservations
            .iter()
            .enumerate()
            .find(|(_, u)| u.by().id() == id)
            .map(|(i, _)| i);

        if let Some(idx) = idx {
            self.reservations.remove(idx);
            Ok(())
        } else {
            Err(Error::UserNotReserved(id))
        }
    }

    pub fn interval(&self) -> &ClosedRange<DateTime<Utc>> {
        &self.interval
    }

    pub fn start(&self) -> DateTime<Utc> {
        self.interval.start
    }

    pub fn max_size(&self) -> usize {
        self.max_size
    }

    pub fn is_empty(&self) -> bool {
        self.reservations.is_empty()
    }

    pub fn reservations(&self) -> &[Reservation] {
        &self.reservations
    }

    pub fn is_available(&self) -> bool {
        self.reservations.len() < self.max_size
    }

    pub fn reserved(&self) -> usize {
        self.reservations.len()
    }
}

#[cfg(test)]
mod slot_tests {
    use super::*;

    use chrono::{Duration, NaiveDate, TimeZone, Utc};

    use crate::domain::models::{Citizenship, OnlyCyrillic, OnlyLatin, UserID, Username};

    fn interval_with_hours<Tz: TimeZone>(
        start_h: u32,
        duration_h: i64,
        tz: Tz,
    ) -> ClosedRange<DateTime<Tz>> {
        let start = tz.with_ymd_and_hms(2025, 1, 1, start_h, 0, 0).unwrap();
        ClosedRange {
            start: start.clone(),
            end: start + Duration::hours(duration_h),
        }
    }

    fn create_user(id: i64) -> User {
        User::new(
            UserID::new(id),
            Username::new("username"),
            OnlyLatin::new("Ivan").unwrap(),
            OnlyCyrillic::new("Иван").unwrap(),
            Citizenship::Armenia,
            NaiveDate::from_ymd_opt(2025, 7, 7).unwrap(),
        )
    }

    #[test]
    fn test_slot_empty_creation() {
        // GIVEN пустой слот
        // GIVEN заданный интервал времени
        let interval = interval_with_hours(1, 2, Utc);
        let slot = Slot::empty(interval.clone(), 1);

        // THEN слот должен быть доступным
        assert!(slot.is_available());

        // THEN слот имеет заданный интервал
        assert_eq!(slot.interval(), &interval);
    }

    #[test]
    fn test_slot_restore_with_valid_data() {
        // GIVEN заданный интервал времени
        // GIVEN 2 пользователя, записанные на получение услуги
        let interval = interval_with_hours(1, 2, Utc);
        let reservations = vec![
            Reservation::new(create_user(1), Service::All),
            Reservation::new(create_user(2), Service::RenewalOfRegistration),
        ];

        // WHEN слот на 3 места восстанавливается из исходных значений
        let slot = Slot::restore(interval.clone(), &reservations, 3).unwrap();

        // THEN слот забронирован указанными ранее пользователями
        assert_eq!(slot.reservations(), reservations);

        // THEN слот всё ещё доступен
        assert!(slot.is_available());
    }

    #[test]
    fn test_slot_restore_with_exceeded_capacity() {
        // GIVEN заданный интервал времени
        // GIVEN 4 пользователя, записанные на получение услуги
        let interval = interval_with_hours(1, 2, Utc);
        let reservations = vec![
            Reservation::new(create_user(1), Service::All),
            Reservation::new(create_user(2), Service::RenewalOfRegistration),
            Reservation::new(create_user(3), Service::Visa),
            Reservation::new(create_user(4), Service::InitialRegistration),
        ];

        // WHEN попытка восстановить слот на 3 места из значений
        let result = Slot::restore(interval, &reservations, 3);

        // THEN ошибка, что слот переполнен
        assert!(result.is_err());
        assert!(matches!(result, Err(Error::MaxCapacityExceeded(3))));
    }

    #[test]
    fn test_slot_reserving() {
        // GIVEN заданный интервал времени
        // GIVEN пустой слот на 3 места
        // GIVEN один пользователь
        let interval = interval_with_hours(1, 2, Utc);
        let mut slot = Slot::empty(interval, 3);
        let user = create_user(1);

        // WHEN пользователь бронирует слот
        let res = slot.reserve(user, Service::RenewalOfVisa);

        // THEN попытка успешна
        assert!(res.is_ok());

        // THEN слот забронирован ровно одним пользователем
        assert_eq!(slot.reserved(), 1);

        // Слот всё ещё доступен
        assert!(slot.is_available());
    }

    #[test]
    fn test_slot_reserving_when_full() {
        // GIVEN заданный интервал времени
        // GIVEN слот, занятый 3 из 3 пользователями
        // GIVEN четвёртый пользователь
        let interval = interval_with_hours(1, 2, Utc);
        let reservations = vec![
            Reservation::new(create_user(1), Service::All),
            Reservation::new(create_user(2), Service::RenewalOfRegistration),
            Reservation::new(create_user(3), Service::Visa),
        ];
        let mut slot = Slot::restore(interval, &reservations, 3).unwrap();
        let user = create_user(4);

        // WHEN пользователь бронирует слот
        let result = slot.reserve(user, Service::InitialRegistration);

        // THEN ошибка переполнения слота
        assert!(matches!(result, Err(Error::MaxCapacityExceeded(3))));

        // THEN слот всё ещё забронирован только 3 пользователями
        assert_eq!(slot.reserved(), 3);
    }
    
    #[test]
    fn test_slot_reserving_twice() {
        // GIVEN заданный интервал времени
        // GIVEN слот на 2 места, занятый одним пользователем
        // GIVEN четвёртый пользователь
        let interval = interval_with_hours(1, 2, Utc);
        let user = create_user(1);
        let reservations = vec![
            Reservation::new(user.clone(), Service::All),
        ];
        let mut slot = Slot::restore(interval, &reservations, 2).unwrap();

        // WHEN тот же пользователь бронирует слот
        let result = slot.reserve(user, Service::InitialRegistration);

        // THEN ошибка повторного бронирования
        assert!(matches!(result, Err(Error::SlotAlreadyReserved(_))));

        // THEN слот всё ещё забронирован на одно место
        assert_eq!(slot.reserved(), 1);
    }
}
