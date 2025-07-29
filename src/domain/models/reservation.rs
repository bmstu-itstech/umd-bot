use crate::domain::models::{Service, User};

#[derive(Debug, Clone, PartialEq)]
pub struct Reservation {
    by: User,
    service: Service,
}

impl Reservation {
    pub fn new(by: User, service: Service) -> Self {
        Self { by, service }
    }

    pub fn by(&self) -> &User {
        &self.by
    }

    pub fn service(&self) -> &Service {
        &self.service
    }
}
