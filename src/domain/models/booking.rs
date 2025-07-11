use chrono::{DateTime, Utc};

use crate::domain::Error;
use crate::domain::models::{Slot, TelegramID};


#[derive(Debug, Clone)]
pub struct Booking {
    time: DateTime<Utc>,
    by:   TelegramID,
}

impl Booking {
    pub fn new(time: DateTime<Utc>, by: TelegramID) -> Self {
        Self { time, by }
    }
    
    pub fn time(&self) -> &DateTime<Utc> {
        &self.time
    }   
    
    pub fn by(&self) -> TelegramID { 
        self.by
    }
}

pub fn book_all<const N: usize>(slots: &mut [Slot<N>], bookings: &[Booking]) -> Result<(), Error>
{
    slots
        .iter_mut()
        .try_for_each(|slot| {
            let booking = bookings
                .iter()
                .find(|&book| book.time() == slot.interval().start());
            if let Some(booking) = booking {
                slot.book(booking.by())?;
            }
            Ok(())
        })
}

#[cfg(test)]
mod booking_tests {
    use chrono::{Duration, TimeZone, Utc};
    
    use crate::domain::Error;
    use crate::domain::models::{book_all, Booking, Interval, Slot, TelegramID};
    

    fn interval_with_hours<Tz: TimeZone>(start_h: u32, duration_h: i64, tz: Tz) -> Interval<Tz> {
        let start = tz.with_ymd_and_hms(2025, 1, 1, start_h, 0, 0).unwrap();
        Interval::with_duration(start, Duration::hours(duration_h)).unwrap()
    }

    fn create_booking(time_h: u32, user_id: i64) -> Booking {
        let time = Utc.with_ymd_and_hms(2025, 1, 1, time_h, 0, 0).unwrap();
        Booking::new(time, TelegramID::new(user_id))
    }
    
    #[test]
    fn test_book_all_with_matching_times() {
        // GIVEN два слота с непересекающимися интервалами времени
        let interval1 = interval_with_hours(1, 2, Utc);
        let interval2 = interval_with_hours(3, 4, Utc);
        let mut slots = vec![
            Slot::<2>::empty(interval1),
            Slot::<2>::empty(interval2),
        ];

        // GIVEN две брони на начала слотов
        let bookings = vec![
            create_booking(1, 1),
            create_booking(3, 2),
        ];

        // WHEN бронирование слотов
        let result = book_all(&mut slots, &bookings);

        // THEN операция успешна
        assert!(result.is_ok());
        
        // THEN первая бронь принадлежит первому слоту
        assert_eq!(slots[0].booked_by().len(), 1);
        assert_eq!(slots[0].booked_by()[0], TelegramID::new(1));
        // THEN вторая бронь принадлежит второму слоту
        assert_eq!(slots[1].booked_by().len(), 1);
        assert_eq!(slots[1].booked_by()[0], TelegramID::new(2));
    }

    #[test]
    fn test_book_all_with_no_matching_times() {
        // GIVEN слот с интервалом времени (1, 3)
        let interval = interval_with_hours(1, 2, Utc);
        let mut slots = vec![Slot::<2>::empty(interval)];

        // GIVEN бронь на время (3)
        let bookings = vec![create_booking(3, 1)]; // Время не совпадает

        // WHEN бронирование слотов
        let result = book_all(&mut slots, &bookings);
        
        // THEN операция успешна
        assert!(result.is_ok());
        
        // THEN слот не будет забронирован
        assert!(slots[0].booked_by().is_empty());
    }

    #[test]
    fn test_book_all_with_full_slot() {
        // GIVEN одноместный занятый слот с интервалом времени (1, 3)
        let interval = interval_with_hours(1, 2, Utc);
        let slot = Slot::<1>::restore(interval, &[TelegramID::new(1)]).unwrap();
        let mut slots = vec![slot];
        
        // GIVEN второй пользователь
        let bookings = vec![create_booking(1, 2)]; // Попытка добавить второго пользователя

        // WHEN попытка забронировать
        let result = book_all(&mut slots, &bookings);

        // THEN ошибка переполнения слота (1, 3)
        assert!(result.is_err());
        assert!(matches!(result, Err(Error::MaxCapacityExceeded(1))));
    }
}
