use chrono::{DateTime, Datelike, NaiveDate, NaiveTime, Utc, Weekday};

use crate::domain::models::ClosedRange;

/// WorkingHoursPolicy описывает рабочие часы сотрудника УМД.
pub trait WorkingHoursPolicy: Send + Sync {
    fn is_working(&self, interval: &ClosedRange<DateTime<Utc>>) -> bool;
}

/// Mon2FriWorkingHoursPolicy описывает рабочую неделю с
/// понедельника по пятницу без учёта времени работы.
#[derive(Default)]
pub struct Mon2FriWorkingHoursPolicy;

impl WorkingHoursPolicy for Mon2FriWorkingHoursPolicy {
    fn is_working(&self, interval: &ClosedRange<DateTime<Utc>>) -> bool {
        let date = interval.start.date_naive();
        match date.weekday() {
            Weekday::Sat | Weekday::Sun => false,
            _ => true,
        }
    }
}

/// Mon2ThuAndFriWithLunchWorkingHoursPolicy описывает стандартную пятидневную рабочую неделю с сокращёнными
/// часами в пятницу с фиксированным обеденным перерывом. Не учитывает праздничные дни.
#[derive(Clone)]
pub struct Mon2ThuAndFriWithLunchWorkingHoursPolicy {
    weekday_hours: ClosedRange<NaiveTime>,
    friday_hours: ClosedRange<NaiveTime>,
    lunch: ClosedRange<NaiveTime>,
}

impl WorkingHoursPolicy for Mon2ThuAndFriWithLunchWorkingHoursPolicy {
    fn is_working(&self, interval: &ClosedRange<DateTime<Utc>>) -> bool {
        let date = interval.start.date_naive();

        let working_hours = match self.bounds(date) {
            Some(working_hours) => working_hours,
            None => return false,
        };

        let lunch_time = ClosedRange {
            start: date.and_time(self.lunch.start).and_utc(),
            end: date.and_time(self.lunch.end).and_utc(),
        };
        working_hours.contains(interval) && !lunch_time.overlaps(interval)
    }
}

#[cfg(test)]
impl Default for Mon2ThuAndFriWithLunchWorkingHoursPolicy {
    fn default() -> Self {
        Self::new(
            ClosedRange {
                start: NaiveTime::from_hms_opt(10, 0, 0).unwrap(),
                end: NaiveTime::from_hms_opt(17, 0, 0).unwrap(),
            },
            ClosedRange {
                start: NaiveTime::from_hms_opt(12, 0, 0).unwrap(),
                end: NaiveTime::from_hms_opt(16, 0, 0).unwrap(),
            },
            ClosedRange {
                start: NaiveTime::from_hms_opt(12, 30, 0).unwrap(),
                end: NaiveTime::from_hms_opt(13, 30, 0).unwrap(),
            },
        )
    }
}

impl Mon2ThuAndFriWithLunchWorkingHoursPolicy {
    pub fn new(
        weekday_hours: ClosedRange<NaiveTime>,
        friday_hours: ClosedRange<NaiveTime>,
        lunch: ClosedRange<NaiveTime>,
    ) -> Self {
        Self {
            weekday_hours,
            friday_hours,
            lunch,
        }
    }

    fn bounds(&self, date: NaiveDate) -> Option<ClosedRange<DateTime<Utc>>> {
        match date.weekday() {
            Weekday::Mon | Weekday::Tue | Weekday::Wed | Weekday::Thu => Some(ClosedRange {
                start: date.and_time(self.weekday_hours.start).and_utc(),
                end: date.and_time(self.weekday_hours.end).and_utc(),
            }),

            Weekday::Fri => Some(ClosedRange {
                start: date.and_time(self.friday_hours.start).and_utc(),
                end: date.and_time(self.friday_hours.end).and_utc(),
            }),

            Weekday::Sat | Weekday::Sun => None,
        }
    }
}

#[cfg(test)]
mod mon2thu_and_fri_with_lunch_working_hours_policy_tests {
    use super::*;
    use chrono::Duration;

    mod test_weekdays_bounds {
        use super::*;
        use chrono::TimeZone;

        #[test]
        fn test_weekday_bounds() {
            // GIVEN Mon2ThuAndFriWithLunchWorkingHoursPolicy
            let policy = Mon2ThuAndFriWithLunchWorkingHoursPolicy::default();

            // WHEN день - понедельник (рабочий и не пятница)
            let date = NaiveDate::from_ymd_opt(2025, 7, 7).unwrap();

            // THEN границы будут с 10 до 17
            let result = policy.bounds(date);
            assert!(result.is_some());
            let bounds = result.unwrap();
            let expected = ClosedRange {
                start: Utc.with_ymd_and_hms(2025, 7, 7, 10, 00, 0).unwrap(),
                end: Utc.with_ymd_and_hms(2025, 7, 7, 17, 00, 0).unwrap(),
            };
            assert_eq!(bounds, expected);
        }

        #[test]
        fn test_friday_bounds() {
            // GIVEN Mon2ThuAndFriWithLunchWorkingHoursPolicy
            let policy = Mon2ThuAndFriWithLunchWorkingHoursPolicy::default();

            // WHEN день - пятница
            let date = NaiveDate::from_ymd_opt(2025, 7, 11).unwrap();

            // THEN границы будут с 12 до 16
            let result = policy.bounds(date);
            assert!(result.is_some());
            let bounds = result.unwrap();
            let expected = ClosedRange {
                start: Utc.with_ymd_and_hms(2025, 7, 11, 12, 00, 0).unwrap(),
                end: Utc.with_ymd_and_hms(2025, 7, 11, 16, 00, 0).unwrap(),
            };
            assert_eq!(bounds, expected);
        }

        #[test]
        fn test_weekend_bounds() {
            // GIVEN Mon2ThuAndFriWithLunchWorkingHoursPolicy
            let policy = Mon2ThuAndFriWithLunchWorkingHoursPolicy::default();

            // WHEN день - выходной
            let date = NaiveDate::from_ymd_opt(2025, 7, 12).unwrap();

            // THEN границы будут отсутствовать, так как день не рабочий
            let result = policy.bounds(date);
            assert!(result.is_none());
        }
    }

    mod test_is_working {
        use super::*;
        use std::ops::Add;

        #[test]
        fn test_weekend_is_not_working() {
            // GIVEN Mon2ThuAndFriWithLunchWorkingHoursPolicy
            let policy = Mon2ThuAndFriWithLunchWorkingHoursPolicy::default();

            // WHEN выходной день
            let date = NaiveDate::from_ymd_opt(2025, 7, 12).unwrap();
            let datetime = date.and_hms_opt(14, 0, 0).unwrap().and_utc();
            let interval = ClosedRange {
                start: datetime,
                end: datetime.add(Duration::hours(1)),
            };

            // THEN в середине дня точно не будет никаких слотов
            assert!(!policy.is_working(&interval));
        }

        #[test]
        fn test_weekday_is_working_until_17_hours() {
            // GIVEN Mon2ThuAndFriWithLunchWorkingHoursPolicy
            let policy = Mon2ThuAndFriWithLunchWorkingHoursPolicy::default();

            // WHEN будний день кроме пятницы и время с 16:00 до 17:00
            let date = NaiveDate::from_ymd_opt(2025, 7, 10).unwrap();
            let datetime = date.and_hms_opt(16, 0, 0).unwrap().and_utc();
            let interval = ClosedRange {
                start: datetime,
                end: datetime.add(Duration::hours(1)),
            };

            // THEN это время будет рабочим
            assert!(policy.is_working(&interval));
        }

        #[test]
        fn test_friday_is_not_working_from_16() {
            // GIVEN Mon2ThuAndFriWithLunchWorkingHoursPolicy
            let policy = Mon2ThuAndFriWithLunchWorkingHoursPolicy::default();

            // WHEN пятница и время с 16 до 17
            let date = NaiveDate::from_ymd_opt(2025, 7, 11).unwrap();
            let datetime = date.and_hms_opt(16, 0, 0).unwrap().and_utc();
            let interval = ClosedRange {
                start: datetime,
                end: datetime.add(Duration::hours(1)),
            };

            // THEN это время будет нерабочим
            assert!(!policy.is_working(&interval));
        }

        #[test]
        fn test_friday_is_working_until_16() {
            // GIVEN Mon2ThuAndFriWithLunchWorkingHoursPolicy
            let policy = Mon2ThuAndFriWithLunchWorkingHoursPolicy::default();

            // WHEN пятница и время с 16 до 17
            let date = NaiveDate::from_ymd_opt(2025, 7, 11).unwrap();
            let datetime = date.and_hms_opt(15, 0, 0).unwrap().and_utc();
            let interval = ClosedRange {
                start: datetime,
                end: datetime.add(Duration::hours(1)),
            };

            // THEN это время будет рабочим
            assert!(policy.is_working(&interval));
        }

        #[test]
        fn test_lunch_is_not_working() {
            // GIVEN Mon2ThuAndFriWithLunchWorkingHoursPolicy
            let policy = Mon2ThuAndFriWithLunchWorkingHoursPolicy::default();

            // WHEN будний день и время, пересекающее обед
            let date = NaiveDate::from_ymd_opt(2025, 7, 10).unwrap();
            let datetime = date.and_hms_opt(12, 0, 0).unwrap().and_utc();
            let interval = ClosedRange {
                start: datetime,
                end: datetime.add(Duration::hours(1)),
            };

            // THEN это время будет нерабочим
            assert!(!policy.is_working(&interval));
        }
    }
}
