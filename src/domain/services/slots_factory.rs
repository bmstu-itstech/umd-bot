use crate::domain::Error;
use crate::domain::models::{ClosedRange, Service, Slot};
use crate::domain::services::{ServicePolicy, WorkingHoursPolicy};
use chrono::{DateTime, Duration, NaiveDate, Utc};

/// SlotsFactory управляет параметрами создания слота, такими как его размер и продолжительность.
pub trait SlotsFactory: Send + Sync {
    fn create(&self, start: DateTime<Utc>, services: Vec<Service>) -> Result<Slot, Error>;
    fn create_all(
        &self,
        date: NaiveDate,
        wp: &dyn WorkingHoursPolicy,
        sp: &dyn ServicePolicy,
    ) -> Result<Vec<Slot>, Error>;
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
    fn create(&self, start: DateTime<Utc>, services: Vec<Service>) -> Result<Slot, Error> {
        Slot::empty(
            ClosedRange {
                start,
                end: start + self.duration,
            },
            self.max_size,
            services,
        )
    }

    fn create_all(
        &self,
        date: NaiveDate,
        wp: &dyn WorkingHoursPolicy,
        sp: &dyn ServicePolicy,
    ) -> Result<Vec<Slot>, Error> {
        let start = date.and_hms_opt(0, 0, 0).unwrap().and_utc();
        let services = sp.available_services(start.date_naive());
        if services.is_empty() {
            return Ok(vec![]);
        }
        // Алгоритм следующий:
        // 1. Создаются временные стартовые метки начиная с 0:00 дня до конца (take_while)
        //    с фиксированным интервалом (можно ли сделать лучше?).
        // 2. Для каждой временной метки создаётся слот.
        // 3. Сборка слотов в вектор для выброса ошибки, если таковая есть. Теоретически, ошибки
        //    быть не может быть, так как на момент написания единственная ошибка это пустой
        //    список услуг, что проверяется строчками выше. Однако, делать unwrap() не безопасно,
        //    так как теоретически количество ошибок может увеличиться, а код ниже исправлен не
        //    будет.
        // 4. Фильтрация только тех слотов, которые доступны согласно политике рабочего времени.
        //    Вообще говоря можно сделать так, чтобы интервалы собирались до слотов, и чтобы не
        //    создавать слоты просто так... минимальная оптимизация, кому она нужна?
        let slots = std::iter::successors(Some(start), move |&start| Some(start + self.duration))
            .take_while(move |time| time.date_naive() == date)
            .map(move |time| self.create(time, services.clone()))
            .collect::<Result<Vec<Slot>, Error>>()?;
        Ok(slots
            .into_iter()
            .filter(move |slot| wp.is_working(slot.interval()))
            .collect())
    }
}

#[cfg(test)]
mod fixed_slots_factory_tests {
    use super::*;
    use crate::domain::services::{
        FriForConsultationsServicePolicy, Mon2ThuAndFriWithLunchWorkingHoursPolicy,
    };
    use chrono::NaiveDate;

    #[test]
    fn test_no_slots_in_weekend() {
        // GIVEN слоты размером 3 и длительностью 20 минут.
        let factory = FixedSlotsFactory::new(3, Duration::minutes(20));
        let wp = Mon2ThuAndFriWithLunchWorkingHoursPolicy::default();

        // WHEN выходной день
        let date = NaiveDate::from_ymd_opt(2025, 7, 12).unwrap();

        // THEN слотов для записи не будет
        let slots = factory
            .create_all(date, &wp, &FriForConsultationsServicePolicy)
            .unwrap();
        assert!(slots.is_empty());
    }
}
