use std::sync::{Arc, Mutex};
use chrono::Utc;

use crate::domain;
use crate::domain::interfaces::StudentsProvider;
use crate::domain::models::TelegramID;
use crate::domain::services::DeadlinePolicy;


#[derive(thiserror::Error, Debug)]
pub enum CheckDeadlineError {
    #[error("student not found: {0}")]
    StudentNotFound(i64),
    
    #[error(transparent)]
    Internal(#[from] Box<dyn std::error::Error + Send + Sync>),
}

pub struct CheckDeadlineUseCase<D>
where
    D: DeadlinePolicy,
{
    deadline_policy: D,
    student_provider: Arc<Mutex<dyn StudentsProvider>>,
}

impl<D> CheckDeadlineUseCase<D>
where
    D: DeadlinePolicy,
{
    
    pub fn new(deadline_policy: D, student_provider: Arc<Mutex<dyn StudentsProvider>>) -> Self {
        Self { deadline_policy, student_provider }
    }
    
    pub async fn check_deadline(&self, student_id: i64) -> Result<bool, CheckDeadlineError> {
        let student_provider = self.student_provider.lock().unwrap();
        let student = student_provider
            .student(TelegramID::new(student_id))
            .await
            .map_err(|err| match err {
                domain::Error::StudentNotFoundError(id) => 
                    CheckDeadlineError::StudentNotFound(id.as_i64()),
                _ => CheckDeadlineError::Internal(Box::new(err)),
            })?;
        
        let today = Utc::now().date_naive();
        let deadline = student
            .arrival_data()
            .checked_add_days(self.deadline_policy.deadline(student.citizenship()))
            .unwrap();
        
        Ok(deadline <= today)
    }
}
