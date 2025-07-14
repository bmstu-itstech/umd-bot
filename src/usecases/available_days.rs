use std::sync::{Arc, Mutex};
use chrono::{NaiveDate, Utc};

use crate::domain;
use crate::domain::interfaces::{BookingsProvider, StudentsProvider};
use crate::domain::models::TelegramID;
use crate::domain::services::{AvailableSlotsFactory, DeadlinePolicy, WorkingHoursPolicy};


#[derive(thiserror::Error, Debug)]
pub enum AvailableDaysError {
    #[error("student not found: {0}")]
    StudentNotFound(i64),
    
    #[error(transparent)]
    Internal(#[from] Box<dyn std::error::Error + Send + Sync>),
}

#[derive(Clone)]
pub struct AvailableDaysUseCase<const N: usize, WH, DP, AF>
where
    WH: WorkingHoursPolicy,
    DP: DeadlinePolicy,
    AF: AvailableSlotsFactory<N>,
{
    working_hours_policy: WH,
    deadline_policy: DP,
    available_slots_factory: AF,
    bookings_provider: Arc<Mutex<dyn BookingsProvider>>,
    student_provider: Arc<Mutex<dyn StudentsProvider>>,
}

impl<const N: usize, WH, DP, AF> AvailableDaysUseCase<N, WH, DP, AF>
where
    WH: WorkingHoursPolicy,
    DP: DeadlinePolicy,
    AF: AvailableSlotsFactory<N> + Clone,
{
    pub fn new(
        working_hours_policy: WH, 
        deadline_policy: DP,
        available_slots_factory: AF,
        bookings_provider: Arc<Mutex<dyn BookingsProvider>>,
        student_provider: Arc<Mutex<dyn StudentsProvider>>,
    ) -> Self {
        Self { 
            working_hours_policy, 
            deadline_policy, 
            available_slots_factory, 
            bookings_provider, 
            student_provider 
        }
    }
    
    pub async fn available_days(&self, student_id: i64) -> Result<Vec<NaiveDate>, AvailableDaysError>{
        let student_provider = self.student_provider.lock().unwrap();
        let student = student_provider
            .student(TelegramID::new(student_id))
            .await
            .map_err(|err| match err {
                domain::Error::StudentNotFoundError(id) =>
                    AvailableDaysError::StudentNotFound(id.as_i64()),
                _ => AvailableDaysError::Internal(Box::new(err)),
            })?;
        
        let today = Utc::now().naive_utc().date();
        let deadline = student
            .arrival_data()
            .checked_add_days(self.deadline_policy.deadline(student.citizenship()))
            .unwrap();
        
        let booking_provider = self.bookings_provider.lock().unwrap();
        let mut available_days = Vec::new();
        
        let cur = today;
        while cur < deadline {
            let bookings = booking_provider
                .bookings(cur)
                .await
                .map_err(|err| AvailableDaysError::Internal(Box::new(err)))?;   // Ошибка провайдера
            
            let slots = self.available_slots_factory
                .available_slots(cur, &bookings)
                .map_err(|err| AvailableDaysError::Internal(Box::new(err)))?;   // Ошибка переполнения слота
            
            if !slots.is_empty() {
                available_days.push(cur);
            }
        }
        
        Ok(available_days)
    }
}
