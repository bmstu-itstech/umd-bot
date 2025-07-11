use std::ops::Add;
use chrono::{DateTime, Duration, TimeZone, Utc};

use crate::domain::Error;


/// Interval описывает временной интервал от start до end в пределах одного дня.
#[derive(Debug, Clone)]
pub struct Interval<Tz: TimeZone> {
    start: DateTime<Tz>,
    end:   DateTime<Tz>,
}


impl<Tz: TimeZone> Interval<Tz> {
    /// new возвращает ошибку Error::InvalidInterval если:
    /// - не выполняется start < end;
    /// - start и end в разных днях.
    pub fn new(start: DateTime<Tz>, end: DateTime<Tz>) -> Result<Self, Error> {
        if start > end {
            return Err(Error::InvalidInterval(
                format!("expected start < end, got {:?} -- {:?}", start, end)
            ))
        }
        
        if start == end {
            return Err(Error::InvalidInterval(
                format!("expected not zero interval, got {:?} -- {:?}", start, end)
            ))
        }
        
        if start.date_naive() != end.date_naive() {
            return Err(Error::InvalidInterval(
                format!("expected interval in one day, got {:?} -- {:?}", start, end)
            ))
        }
        
        Ok(Self { start, end })
    }
    
    pub fn with_duration(start: DateTime<Tz>, duration: Duration) -> Result<Self, Error> {
        let end = start.clone().add(duration);
        Self::new(start, end)
    }

    pub fn start(&self) -> &DateTime<Tz> {
        &self.start
    }

    pub fn end(&self) -> &DateTime<Tz> {
        &self.end
    }

    pub fn contains(&self, other: &Interval<Tz>) -> bool {
        self.start <= other.start && self.end >= other.end
    }

    pub fn overlaps(&self, other: &Interval<Tz>) -> bool {
        self.start < other.end && self.end > other.start
    }

    pub fn is_disjoint(&self, other: &Interval<Tz>) -> bool {
        !self.overlaps(other)
    }

    pub fn separate_with(&self, other: &Interval<Tz>) -> bool {
        self.is_disjoint(other) && !self.contains(other) && !other.contains(self)
    }
}

impl PartialEq for Interval<Utc> {
    fn eq(&self, other: &Self) -> bool {
        self.start == other.start && self.end == other.end
    }

    fn ne(&self, other: &Self) -> bool {
        self.start != other.start || self.end != other.end
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Utc, TimeZone, Duration};

    fn interval_with_hours<Tz: TimeZone>(start_h: i64, end_h: i64, tz: Tz) -> Interval<Tz> {
        let start = tz.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).unwrap() + Duration::hours(start_h);
        let end = tz.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).unwrap() + Duration::hours(end_h);
        Interval::new(start, end).unwrap()
    }

    #[test]
    fn test_creation_and_accessors() {
        let start = Utc::now();
        let end = start + Duration::hours(1);
        let interval = Interval::new(start, end).unwrap();

        assert_eq!(interval.start(), &start);
        assert_eq!(interval.end(), &end);
    }
    
    #[test]
    fn test_error_if_interval_is_invalid() {
        let start = Utc::now();
        let end = start + Duration::hours(1);
        let res = Interval::new(end, start);
        assert!(res.is_err());
        assert!(matches!(res, Err(Error::InvalidInterval(_))));
    }

    #[test]
    fn test_error_if_interval_greater_then_one_day() {
        let start = Utc::now();
        let end = start + Duration::hours(25);
        let res = Interval::new(start, end);
        assert!(res.is_err());
        assert!(matches!(res, Err(Error::InvalidInterval(_))));
    }

    #[test]
    fn test_contains() {
        let base = interval_with_hours(1, 5, Utc);

        // Полное совпадение
        assert!(base.contains(&interval_with_hours(1, 5, Utc)));
        // Полное включение
        assert!(base.contains(&interval_with_hours(2, 4, Utc)));
        // Не включен (выходит за границы)
        assert!(!base.contains(&interval_with_hours(2, 6, Utc)));
        // Не включен (до)
        assert!(!base.contains(&interval_with_hours(0, 2, Utc)));
    }

    #[test]
    fn test_overlaps() {
        let base = interval_with_hours(2, 5, Utc);

        // Пересечение справа
        assert!(base.overlaps(&interval_with_hours(4, 6, Utc)));
        // Пересечение слева
        assert!(base.overlaps(&interval_with_hours(1, 3, Utc)));
        // Соприкосновение без пересечения
        assert!(!base.overlaps(&interval_with_hours(5, 7, Utc)));
        // Полное отсутствие пересечения
        assert!(!base.overlaps(&interval_with_hours(6, 8, Utc)));
    }

    #[test]
    fn test_is_disjoint() {
        let base = interval_with_hours(2, 5, Utc);

        // Непересекающиеся интервалы
        assert!(base.is_disjoint(&interval_with_hours(0, 1, Utc)));
        assert!(base.is_disjoint(&interval_with_hours(6, 8, Utc)));
        // Соприкосновение границ
        assert!(base.is_disjoint(&interval_with_hours(5, 7, Utc)));
        // Пересекающиеся интервалы
        assert!(!base.is_disjoint(&interval_with_hours(4, 6, Utc)));
    }

    #[test]
    fn test_separate_with() {
        let base = interval_with_hours(2, 5, Utc);

        // Полностью отдельные интервалы
        assert!(base.separate_with(&interval_with_hours(0, 1, Utc)));
        assert!(base.separate_with(&interval_with_hours(6, 8, Utc)));
        // Пересекающиеся
        assert!(!base.separate_with(&interval_with_hours(4, 6, Utc)));
        // Содержащиеся внутри
        assert!(!base.separate_with(&interval_with_hours(3, 4, Utc)));
        // Содержащие base
        assert!(!base.separate_with(&interval_with_hours(1, 6, Utc)));
    }
}
