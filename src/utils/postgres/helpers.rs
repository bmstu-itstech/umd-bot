use tokio_postgres::error::{SqlState};
use tokio_postgres::Error as PgError;


pub fn is_unique_violation(error: &PgError) -> bool {
    error
        .as_db_error()
        .map(|e| e.code() == &SqlState::UNIQUE_VIOLATION)
        .unwrap_or(false)
}

pub fn is_foreign_key_violation(error: &PgError) -> bool {
    error
        .as_db_error()
        .map(|e| e.code() == &SqlState::FOREIGN_KEY_VIOLATION)
        .unwrap_or(false)
}

pub fn is_not_data_found(error: &PgError) -> bool {
    error
        .as_db_error()
        .map(|e| e.code() == &SqlState::NO_DATA_FOUND)
        .unwrap_or(false)
}
