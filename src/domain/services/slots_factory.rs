use chrono::{DateTime, Duration, NaiveDate, Utc};

use crate::domain::models::{ClosedRange, Slot};
use crate::domain::services::WorkingHoursPolicy;

/// SlotsFactory управляет параметрами создания слота, такими как его размер и продолжительность.
pub trait SlotsFactory: Send + Sync {
    fn create(&self, start: DateTime<Utc>) -> Slot;
    fn create_all(&self, date: NaiveDate, wp: &dyn WorkingHoursPolicy) -> Vec<Slot>;
}

/// Теоретически, выделяя создание слота в абстракцию, можно добиться создания слотов разного
/// размера и продолжительностью согласно рабочей политике.

/// FixedSlotsFactory создаёт слоты фиксированного размера и продолжительности.
pub struct FixedSlotsFactory {
    max_size: usize,
    duration: Duration,
}

impl FixedSlotsFactory {
    pub fn new(max_size: usize, duration: Duration) -> Self {
        Self { max_size, duration }
    }
}

impl SlotsFactory for FixedSlotsFactory {
    fn create(&self, start: DateTime<Utc>) -> Slot {
        Slot::empty(
            ClosedRange {
                start,
                end: start + self.duration,
            },
            self.max_size,
        )
    }

    fn create_all(&self, date: NaiveDate, wp: &dyn WorkingHoursPolicy) -> Vec<Slot> {
        let start = date.and_hms_opt(0, 0, 0).unwrap().and_utc();
        std::iter::successors(Some(start), move |&start| Some(start + self.duration))
            .take_while(move |time| time.date_naive() == date)
            .map(move |time| self.create(time))
            .filter(move |slot| wp.is_working(slot.interval()))
            .collect()
    }
}

#[cfg(test)]
mod fixed_slots_factory_tests {
    use super::*;
    use crate::domain::services::Mon2ThuAndFriWithLunchWorkingHoursPolicy;
    use chrono::NaiveDate;

    #[test]
    fn test_no_slots_in_weekend() {
        // GIVEN слоты размером 3 и длительностью 20 минут.
        let factory = FixedSlotsFactory::new(3, Duration::minutes(20));
        let wp = Mon2ThuAndFriWithLunchWorkingHoursPolicy::default();

        // WHEN выходной день
        let date = NaiveDate::from_ymd_opt(2025, 7, 12).unwrap();

        // THEN слотов для записи не будет
        let slots = factory.create_all(date, &wp);
        assert!(slots.is_empty());
    }
}
