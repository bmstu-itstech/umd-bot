use chrono::{Datelike, NaiveDate, NaiveTime, Utc, Weekday};

use crate::domain::models::{Interval, NaiveTimeInterval};


/// WorkingHoursPolicy описывает рабочие часы сотрудника УМД.
pub trait WorkingHoursPolicy {
    fn bounds(&self, date: NaiveDate) -> Option<Interval<Utc>>;
    fn is_working(&self, interval: &Interval<Utc>) -> bool;
}

/// Mon2FriWorkingHoursPolicy описывает рабочую неделю с
/// понедельника по пятницу без учёта времени работы.
#[derive(Default)]
pub struct Mon2FriWorkingHoursPolicy;

impl WorkingHoursPolicy for Mon2FriWorkingHoursPolicy {
    fn bounds(&self, date: NaiveDate) -> Option<Interval<Utc>> {
        match date.weekday() {
            Weekday::Sat | Weekday::Sun => None,
            _ => Some(Interval::new(
                date.and_time(NaiveTime::from_hms_opt(0, 0, 0)?).and_utc(),
                date.and_time(NaiveTime::from_hms_opt(23, 59, 59)?).and_utc(),
            ).unwrap())
        }
    }

    fn is_working(&self, interval: &Interval<Utc>) -> bool {
        // Считаем, что интервалы в рамках одного дня
        let date = interval.start().date_naive();
        match date.weekday() {
            Weekday::Sat | Weekday::Sun => false,
            _ => true,
        }
    }
}


/// Mon2ThuAndFriWithLunchWorkingHoursPolicy описывает стандартную пятидневную рабочую неделю с сокращёнными
/// часами в пятницу с фиксированным обеденным перерывом. Не учитывает праздничные дни.
pub struct Mon2ThuAndFriWithLunchWorkingHoursPolicy {
    weekday_hours: NaiveTimeInterval,
    friday_hours:  NaiveTimeInterval,
    lunch:         NaiveTimeInterval,
}

impl WorkingHoursPolicy for Mon2ThuAndFriWithLunchWorkingHoursPolicy {
    fn bounds(&self, date: NaiveDate) -> Option<Interval<Utc>> {
        match date.weekday() {
            Weekday::Mon | Weekday::Tue | Weekday::Wed | Weekday::Thu =>
                Some(self.weekday_hours.clone().and_date(date)),

            Weekday::Fri =>
                Some(self.friday_hours.clone().and_date(date)),

            Weekday::Sat | Weekday::Sun => None,
        }
    }

    fn is_working(&self, interval: &Interval<Utc>) -> bool {
        // Считаем, что интервалы в рамках одного дня
        let date = interval.start().date_naive();

        let working_hours = match self.bounds(date) {
            Some(working_hours) => working_hours,
            None => return false,
        };
        let lunch_time = self.lunch.clone().and_date(date);
        working_hours.contains(interval) && !lunch_time.overlaps(interval)
    }
}

impl Mon2ThuAndFriWithLunchWorkingHoursPolicy {
    pub fn new(
        weekday_hours: NaiveTimeInterval,
        friday_hours:  NaiveTimeInterval,
        lunch:         NaiveTimeInterval,
    ) -> Self {
        Self { weekday_hours, friday_hours, lunch }
    }
}

#[cfg(test)]
mod mon2thu_and_fri_with_lunch_working_hours_policy_tests {
    use chrono::Duration;
    use super::*;

    fn setup_default() -> Mon2ThuAndFriWithLunchWorkingHoursPolicy {
        Mon2ThuAndFriWithLunchWorkingHoursPolicy::new(
            NaiveTimeInterval::new(
                    NaiveTime::from_hms_opt(10, 0, 0).unwrap(),
                    NaiveTime::from_hms_opt(17, 0, 0).unwrap(),
            ).unwrap(),
            NaiveTimeInterval::new(
                NaiveTime::from_hms_opt(12, 0, 0).unwrap(),
                NaiveTime::from_hms_opt(16, 0, 0).unwrap(),
            ).unwrap(),
            NaiveTimeInterval::new(
                NaiveTime::from_hms_opt(12, 30, 0).unwrap(),
                NaiveTime::from_hms_opt(13, 30, 0).unwrap(),
            ).unwrap(),
        )
    }

    mod test_weekdays_bounds {
        use chrono::TimeZone;
        use super::*;
        
        #[test]
        fn test_weekday_bounds() {
            // GIVEN Mon2ThuAndFriWithLunchWorkingHoursPolicy
            let policy = setup_default();

            // WHEN день - понедельник (рабочий и не пятница)
            let date = NaiveDate::from_ymd_opt(2025, 7, 7).unwrap();
            
            // THEN границы будут с 10 до 17
            let result = policy.bounds(date);
            assert!(result.is_some());
            let bounds = result.unwrap();
            let expected = Interval::new(
                Utc.with_ymd_and_hms(2025, 7, 7, 10, 00, 0).unwrap(),
                Utc.with_ymd_and_hms(2025, 7, 7, 17, 00, 0).unwrap(),
            ).unwrap();
            assert_eq!(bounds, expected);
        }

        #[test]
        fn test_friday_bounds() {
            // GIVEN Mon2ThuAndFriWithLunchWorkingHoursPolicy
            let policy = setup_default();

            // WHEN день - пятница
            let date = NaiveDate::from_ymd_opt(2025, 7, 11).unwrap();

            // THEN границы будут с 12 до 16
            let result = policy.bounds(date);
            assert!(result.is_some());
            let bounds = result.unwrap();
            let expected = Interval::new(
                Utc.with_ymd_and_hms(2025, 7, 11, 12, 00, 0).unwrap(),
                Utc.with_ymd_and_hms(2025, 7, 11, 16, 00, 0).unwrap(),
            ).unwrap();
            assert_eq!(bounds, expected);
        }

        #[test]
        fn test_weekend_bounds() {
            // GIVEN Mon2ThuAndFriWithLunchWorkingHoursPolicy
            let policy = setup_default();

            // WHEN день - выходной
            let date = NaiveDate::from_ymd_opt(2025, 7, 12).unwrap();

            // THEN границы будут отсутствовать, так как день не рабочий
            let result = policy.bounds(date);
            assert!(result.is_none());
        }
    }
    
    mod test_is_working {
        use super::*;
        
        #[test]
        fn test_weekend_is_not_working() {
            // GIVEN Mon2ThuAndFriWithLunchWorkingHoursPolicy
            let policy = setup_default();

            // WHEN выходной день
            let date = NaiveDate::from_ymd_opt(2025, 7, 12).unwrap();
            let datetime = date.and_hms_opt(14, 0, 0).unwrap().and_utc();
            let interval = Interval::with_duration(datetime, Duration::hours(1)).unwrap();

            // THEN в середине дня точно не будет никаких слотов
            assert!(!policy.is_working(&interval));
        }

        #[test]
        fn test_weekday_is_working_until_17_hours() {
            // GIVEN Mon2ThuAndFriWithLunchWorkingHoursPolicy
            let policy = setup_default();

            // WHEN будний день кроме пятницы и время с 16:00 до 17:00
            let date = NaiveDate::from_ymd_opt(2025, 7, 10).unwrap();
            let datetime = date.and_hms_opt(16, 0, 0).unwrap().and_utc();
            let interval = Interval::with_duration(datetime, Duration::hours(1)).unwrap();

            // THEN это время будет рабочим
            assert!(policy.is_working(&interval));
        }

        #[test]
        fn test_friday_is_not_working_from_16() {
            // GIVEN Mon2ThuAndFriWithLunchWorkingHoursPolicy
            let policy = setup_default();

            // WHEN пятница и время с 16 до 17
            let date = NaiveDate::from_ymd_opt(2025, 7, 11).unwrap();
            let datetime = date.and_hms_opt(16, 0, 0).unwrap().and_utc();
            let interval = Interval::with_duration(datetime, Duration::hours(1)).unwrap();

            // THEN это время будет нерабочим
            assert!(!policy.is_working(&interval));
        }

        #[test]
        fn test_friday_is_working_until_16() {
            // GIVEN Mon2ThuAndFriWithLunchWorkingHoursPolicy
            let policy = setup_default();

            // WHEN пятница и время с 16 до 17
            let date = NaiveDate::from_ymd_opt(2025, 7, 11).unwrap();
            let datetime = date.and_hms_opt(15, 0, 0).unwrap().and_utc();
            let interval = Interval::with_duration(datetime, Duration::hours(1)).unwrap();

            // THEN это время будет рабочим
            assert!(policy.is_working(&interval));
        }

        #[test]
        fn test_lunch_is_not_working() {
            // GIVEN Mon2ThuAndFriWithLunchWorkingHoursPolicy
            let policy = setup_default();

            // WHEN будний день и время, пересекающее обед
            let date = NaiveDate::from_ymd_opt(2025, 7, 10).unwrap();
            let datetime = date.and_hms_opt(12, 0, 0).unwrap().and_utc();
            let interval = Interval::with_duration(datetime, Duration::hours(1)).unwrap();

            // THEN это время будет нерабочим
            assert!(!policy.is_working(&interval));
        }
    }
}
