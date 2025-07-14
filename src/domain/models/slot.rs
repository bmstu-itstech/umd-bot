use chrono::Utc;

use crate::domain::Error;
use crate::domain::models::{Booking, Interval, TelegramID};


#[derive(Debug, Clone, PartialEq)]
pub struct Slot<const N: usize> {
    interval:  Interval<Utc>,
    booked_by: Vec<TelegramID>,
}

impl<const N: usize> Slot<N> {
    pub fn empty(interval: Interval<Utc>) -> Self {
        Self { interval, booked_by: Vec::with_capacity(N) }
    }
    
    pub fn restore(interval: Interval<Utc>, reserved_by: &[TelegramID]) -> Result<Self, Error> {
        if reserved_by.len() > N {
            return Err(Error::MaxCapacityExceeded(N))
        }
        
        Ok(Self { 
            interval, 
            booked_by: Vec::from(reserved_by),
        })
    }
    
    pub fn book(&mut self, id: TelegramID) -> Result<Booking, Error> {
        if self.booked_by.len() >= N {
            return Err(Error::MaxCapacityExceeded(N))
        }
        self.booked_by.push(id);
        Ok(Booking::new(self.interval.start().clone(), id))
    }
    
    pub fn interval(&self) -> &Interval<Utc> {
        &self.interval
    }
    
    pub fn is_empty(&self) -> bool {
        self.booked_by.is_empty()
    }
    
    pub fn is_available(&self) -> bool {
        self.booked_by.len() < N
    }
    
    pub fn booked_by(&self) -> &[TelegramID] {
        &self.booked_by
    }
}

#[cfg(test)]
mod slot_tests {
    use super::*;

    use chrono::{TimeZone, Utc};
    use chrono::Duration;

    use crate::domain::models::{Booking, Interval};

    fn interval_with_hours<Tz: TimeZone>(start_h: u32, duration_h: i64, tz: Tz) -> Interval<Tz> {
        let start = tz.with_ymd_and_hms(2025, 1, 1, start_h, 0, 0).unwrap();
        Interval::with_duration(start, Duration::hours(duration_h)).unwrap()
    }

    fn create_booking(time_h: u32, user_id: i64) -> Booking {
        let time = Utc.with_ymd_and_hms(2025, 1, 1, time_h, 0, 0).unwrap();
        Booking::new(time, TelegramID::new(user_id))
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
        // GIVEN ID 2 пользователей
        let interval = interval_with_hours(1, 2, Utc);
        let users = vec![TelegramID::new(1), TelegramID::new(2)];

        // WHEN слот на 3 месте восстанавливается из исходных значений
        let slot = Slot::<3>::restore(interval.clone(), &users).unwrap();

        // THEN слот забронирован указанными ранее пользователями
        assert_eq!(slot.booked_by(), users);

        // THEN слот всё ещё доступен
        assert!(slot.is_available());
    }

    #[test]
    fn test_slot_restore_with_exceeded_capacity() {
        // GIVEN заданный интервал времени
        // GIVEN ID 4 пользователей
        let interval = interval_with_hours(1, 2, Utc);
        let users = vec![TelegramID::new(1), TelegramID::new(2), TelegramID::new(3), TelegramID::new(4)];

        // WHEN попытка восстановить слот на 3 месте из значений
        let result = Slot::<3>::restore(interval, &users);

        // THEN ошибка, что слот переполнен
        assert!(result.is_err());
        assert!(matches!(result, Err(Error::MaxCapacityExceeded(3))));
    }

    #[test]
    fn test_slot_booking() {
        // GIVEN заданный интервал времени
        // GIVEN пустой слот на 3 места
        // GIVEN ID одного пользователя
        let interval = interval_with_hours(1, 2, Utc);
        let mut slot = Slot::<3>::empty(interval);
        let user_id = TelegramID::new(1);

        // WHEN пользователь бронирует слот
        let res = slot.book(user_id);

        // THEN попытка успешна
        assert!(res.is_ok());

        // THEN слот забронирован ровно одним пользователем
        assert_eq!(slot.booked_by().len(), 1);

        // Слот всё ещё доступен
        assert!(slot.is_available());
    }

    #[test]
    fn test_slot_booking_when_full() {
        // GIVEN заданный интервал времени
        // GIVEN слот, занятый 3 из 3 пользователями
        // GIVEN ID четвёртого пользователя
        let interval = interval_with_hours(1, 2, Utc);
        let users = vec![TelegramID::new(1), TelegramID::new(2), TelegramID::new(3)];
        let mut slot = Slot::<3>::restore(interval, &users).unwrap();
        let user_id = TelegramID::new(4);

        // WHEN пользователь бронирует слот
        let result = slot.book(user_id);

        // THEN ошибка переполнения слота
        assert!(matches!(result, Err(Error::MaxCapacityExceeded(3))));

        // THEN слот всё ещё забронирован только 3 пользователями
        assert_eq!(slot.booked_by().len(), 3);
    }
}
