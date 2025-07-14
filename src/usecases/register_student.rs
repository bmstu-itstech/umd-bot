use std::sync::{Arc, Mutex};

use crate::domain;
use crate::domain::interfaces::StudentsRepository;
use crate::domain::models;
use crate::usecases::Student;


#[derive(thiserror::Error, Debug)]
pub enum RegisterStudentError {
    #[error("student already registered: {0}")]
    AlreadyRegistered(i64),
    
    #[error("invalid student data")]
    InvalidStudentData(String),
    
    #[error(transparent)]
    Internal(#[from] Box<dyn std::error::Error + Send + Sync>),
}

pub struct RegisterStudentUseCase {
    student_repos: Arc<Mutex<dyn StudentsRepository>>,
}

impl RegisterStudentUseCase {
    pub fn new(student_repos: Arc<Mutex<dyn StudentsRepository>>) -> Self {
        Self { student_repos }
    }
    
    pub async fn register(&self, s: &Student) -> Result<(), RegisterStudentError> {
        let id = models::TelegramID::new(s.id);
        let username = models::TelegramUsername::new(s.username.clone());
        let full_name_lat = models::OnlyLatin::new(s.full_name_lat.clone())
            .map_err(|err| match err {
                domain::Error::InvalidValue(s) => RegisterStudentError::InvalidStudentData(s),
                _ => RegisterStudentError::Internal(Box::new(err)),
            })?;
        let full_name_cyr = models::OnlyCyrillic::new(s.full_name_cyr.clone())
            .map_err(|err| match err {
                domain::Error::InvalidValue(s) => RegisterStudentError::InvalidStudentData(s),
                _ => RegisterStudentError::Internal(Box::new(err)),
            })?;
        let citizenship: models::Citizenship = s.citizenship.clone().into();
        let arrival_date = s.arrival_date.clone();
        
        let student = models::Student::new(
            id, username, full_name_lat, full_name_cyr, citizenship, arrival_date
        );

        let student_repos = self.student_repos.lock().unwrap();
        student_repos.save(student)
            .await
            .map_err(|err| match err {
                domain::Error::StudentAlreadyExists(id) => RegisterStudentError::AlreadyRegistered(id.as_i64()),
                _ => RegisterStudentError::Internal(Box::new(err)),
            })
    }
}
