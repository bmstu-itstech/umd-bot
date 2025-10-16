use crate::domain::models::Service;
use chrono::{Datelike, NaiveDate, Weekday};

/// ServicePolicy описывает доступные сервисы для слотов в зависимости от дня недели.
pub trait ServicePolicy: Send + Sync {
    fn available_services(&self, date: NaiveDate) -> Vec<Service>;
}

/// FriForConsultationsServicePolicy описывает политику, при которой в пятницу доступны только
/// консультации.
pub struct FriForConsultationsServicePolicy;

impl ServicePolicy for FriForConsultationsServicePolicy {
    fn available_services(&self, date: NaiveDate) -> Vec<Service> {
        match date.weekday() {
            Weekday::Sat | Weekday::Sun => Vec::new(),
            Weekday::Mon | Weekday::Tue | Weekday::Wed | Weekday::Thu => Service::all().to_vec(),
            Weekday::Fri => vec![Service::Consultation],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    #[test]
    fn test_friday_only_consultations() {
        // GIVEN FriForConsultationServicePolicy
        let policy = FriForConsultationsServicePolicy;

        // WHEN пятница
        let friday = NaiveDate::from_ymd_opt(2024, 1, 5).unwrap(); // 5 января 2024 - пятница

        // THEN из услуг доступна только консультация
        let services = policy.available_services(friday);
        assert_eq!(services, vec![Service::Consultation]);
        assert_eq!(services.len(), 1);
    }

    #[test]
    fn test_weekdays_all_services() {
        // GIVEN FriForConsultationServicePolicy
        let policy = FriForConsultationsServicePolicy;

        // WHEN дни с понедельника по четверг
        let weekdays = vec![
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(), // понедельник
            NaiveDate::from_ymd_opt(2024, 1, 2).unwrap(), // вторник
            NaiveDate::from_ymd_opt(2024, 1, 3).unwrap(), // среда
            NaiveDate::from_ymd_opt(2024, 1, 4).unwrap(), // четверг
        ];

        // THEN доступны все услуги
        for day in weekdays {
            let services = policy.available_services(day);
            assert_eq!(services, Service::all().to_vec());
        }
    }

    #[test]
    fn test_weekend_no_services() {
        // GIVEN FriForConsultationServicePolicy
        let policy = FriForConsultationsServicePolicy;

        // WHEN суббота и воскресенье
        let weekend_days = vec![
            NaiveDate::from_ymd_opt(2024, 1, 6).unwrap(), // суббота
            NaiveDate::from_ymd_opt(2024, 1, 7).unwrap(), // воскресенье
        ];

        // THEN услуги не доступны
        for day in weekend_days {
            let services = policy.available_services(day);
            assert!(services.is_empty());
        }
    }
}
