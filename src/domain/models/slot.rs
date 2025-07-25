use chrono::{DateTime, Utc};

use crate::domain::Error;
use crate::domain::models::{ClosedRange, User};


#[derive(Debug, Clone)]
pub struct Slot<const N: usize> {
    interval:  ClosedRange<DateTime<Utc>>,
    reserved_by: Vec<User>,
}

impl<const N: usize> Slot<N> {
    pub fn empty(interval: ClosedRange<DateTime<Utc>>) -> Self {
        Self { interval, reserved_by: Vec::with_capacity(N) }
    }
    
    pub fn restore(interval: ClosedRange<DateTime<Utc>>, reserved_by: &[User]) -> Result<Self, Error> {
        if reserved_by.len() > N {
            return Err(Error::MaxCapacityExceeded(N))
        }
        
        Ok(Self { 
            interval, 
            reserved_by: Vec::from(reserved_by),
        })
    }
    
    pub fn reserve(&mut self, user: &User) -> Result<(), Error> {
        if self.reserved_by.len() >= N {
            return Err(Error::MaxCapacityExceeded(N))
        }
        self.reserved_by.push(user.clone());
        Ok(())
    }

    pub fn cancel(&mut self, user: &User) -> Result<(), Error> {
        let idx = self.reserved_by
            .iter()
            .enumerate()
            .find(|(_, u)| u.id() == user.id())
            .map(|(i, _)| i);
        
        if let Some(idx) = idx {
            self.reserved_by.remove(idx);
            Ok(())
        } else {
            Err(Error::UserNotReserved(user.id().clone()))
        }
    }
    
    pub fn interval(&self) -> &ClosedRange<DateTime<Utc>> {
        &self.interval
    }
    
    pub fn start(&self) -> DateTime<Utc> {
        self.interval.start
    }
    
    pub fn is_empty(&self) -> bool {
        self.reserved_by.is_empty()
    }
    
    pub fn is_available(&self) -> bool {
        self.reserved_by.len() < N
    }
    
    pub fn reserved_by(&self) -> &[User] {
        &self.reserved_by
    }
}

#[cfg(test)]
mod slot_tests {
    use super::*;

    use chrono::{Duration, NaiveDate, TimeZone, Utc};

    use crate::domain::models::{Citizenship, OnlyCyrillic, OnlyLatin, TelegramID, TelegramUsername};

    fn interval_with_hours<Tz: TimeZone>(start_h: u32, duration_h: i64, tz: Tz) -> ClosedRange<DateTime<Tz>> {
        let start = tz.with_ymd_and_hms(2025, 1, 1, start_h, 0, 0).unwrap();
        ClosedRange { start: start.clone(), end: start + Duration::hours(duration_h) }
    }

    fn create_user(id: i64) -> User {
        User::new(
            TelegramID::new(id),
            TelegramUsername::new("username"),
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
        let slot = Slot::<3>::empty(interval.clone());

        // THEN слот должен быть доступным
        assert!(slot.is_available());

        // THEN слот имеет заданный интервал
        assert_eq!(slot.interval(), &interval);
    }

    #[test]
    fn test_slot_restore_with_valid_data() {
        // GIVEN заданный интервал времени
        // GIVEN 2 пользователя
        let interval = interval_with_hours(1, 2, Utc);
        let users = vec![
            create_user(1),
            create_user(1),
        ];

        // WHEN слот на 3 месте восстанавливается из исходных значений
        let slot = Slot::<3>::restore(interval.clone(), &users).unwrap();

        // THEN слот забронирован указанными ранее пользователями
        assert_eq!(slot.reserved_by(), users);

        // THEN слот всё ещё доступен
        assert!(slot.is_available());
    }

    #[test]
    fn test_slot_restore_with_exceeded_capacity() {
        // GIVEN заданный интервал времени
        // GIVEN 4 пользователя
        let interval = interval_with_hours(1, 2, Utc);
        let users = vec![
            create_user(1),
            create_user(2),
            create_user(3),
            create_user(4),
        ];

        // WHEN попытка восстановить слот на 3 месте из значений
        let result = Slot::<3>::restore(interval, &users);

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
        let mut slot = Slot::<3>::empty(interval);
        let user = create_user(1);

        // WHEN пользователь бронирует слот
        let res = slot.reserve(&user);

        // THEN попытка успешна
        assert!(res.is_ok());

        // THEN слот забронирован ровно одним пользователем
        assert_eq!(slot.reserved_by().len(), 1);

        // Слот всё ещё доступен
        assert!(slot.is_available());
    }

    #[test]
    fn test_slot_reserving_when_full() {
        // GIVEN заданный интервал времени
        // GIVEN слот, занятый 3 из 3 пользователями
        // GIVEN четвёртый пользователь
        let interval = interval_with_hours(1, 2, Utc);
        let users = vec![
            create_user(1),
            create_user(2),
            create_user(3),
        ];
        let mut slot = Slot::<3>::restore(interval, &users).unwrap();
        let user = create_user(4);

        // WHEN пользователь бронирует слот
        let result = slot.reserve(&user);

        // THEN ошибка переполнения слота
        assert!(matches!(result, Err(Error::MaxCapacityExceeded(3))));

        // THEN слот всё ещё забронирован только 3 пользователями
        assert_eq!(slot.reserved_by().len(), 3);
    }
}
