use std::ops::Add;
use chrono::{Duration, Utc};

use crate::domain::models::{Interval, Slot};

/// SlotsFactory создаёт слоты для заданного промежутка времени.
pub trait SlotsFactory<const N: usize> {
    fn slots(&self, bounds: &Interval<Utc>) -> impl Iterator<Item = Slot<N>>;
}

/// FixedDurationSlotsFactory создаёт слоты с фиксированной продолжительностью времени.
#[derive(Clone)]
pub struct FixedDurationSlotsFactory<const N: usize> {
    duration: Duration
}

impl<const N: usize> SlotsFactory<N> for FixedDurationSlotsFactory<N> {
    fn slots(&self, bounds: &Interval<Utc>) -> impl Iterator<Item = Slot<N>> {
        let start = Interval::with_duration(bounds.start().clone(), self.duration)
            .unwrap(); // Гарантируется, что слоты будут находиться в пределах одного дня.
        
        std::iter::successors(Some(start), move |cur| {
            Some(Interval::with_duration(
                    cur
                        .start()
                        .clone()
                        .add(self.duration), 
                    self.duration
                )
                    .unwrap()   // Так как общий интервал не выходит за границы дня, то и внутренние
                                // интервалы слотов тоже не выходят
            )
        })
            .map(|int| Slot::empty(int))
            .take_while(|slot| bounds.contains(slot.interval()))
    }
}

impl<const N: usize> FixedDurationSlotsFactory<N> {
    pub fn new(duration: Duration) -> Self {
        Self { duration }
    }
}

#[cfg(test)]
mod fixed_duration_slots_factory_tests {
    use chrono::TimeZone;
    use super::*;
    
    const N: usize = 1;

    fn interval_with_times(start: (u32, u32), end: (u32, u32)) -> Interval<Utc> {
        let start = Utc.with_ymd_and_hms(2025, 1, 1, start.0, start.1, 0).unwrap();
        let end = Utc.with_ymd_and_hms(2025, 1, 1, end.0, end.1, 0).unwrap();
        Interval::new(start, end).unwrap()
    }
    
    #[test]
    fn test_no_slots() {
        // GIVEN слоты по 20 минут
        let factory = FixedDurationSlotsFactory::<N>::new(Duration::minutes(20));
        
        // WHEN заданный промежуток времени в 10 минут
        let bounds = interval_with_times((9, 0), (9, 10));
        
        // THEN вектор пустых слотов будет пустым
        let res: Vec<_> = factory.slots(&bounds).collect();
        assert!(res.is_empty());
    }
    
    #[test]
    fn test_one_slot() {
        // GIVEN слоты по 20 минут
        let factory = FixedDurationSlotsFactory::<N>::new(Duration::minutes(20));

        // WHEN заданный промежуток времени в 20 минут
        let bounds = interval_with_times((9, 0), (9, 20));
        
        // THEN вектор будет содержать единственный слот, который совпадает с заданным интервалом
        let res: Vec<_> = factory.slots(&bounds).collect();
        assert_eq!(res, vec![Slot::empty(bounds)]);
    }
}
