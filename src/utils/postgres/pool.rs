use std::str::FromStr;

use deadpool_postgres::{Manager, ManagerConfig, Pool, RecyclingMethod};
use thiserror::Error;
use tokio_postgres::{Config, NoTls};


#[derive(Error, Debug)]
pub enum ConnectionError {
    #[error("database error: {0}")]
    PostgresError(#[from] tokio_postgres::Error),

    #[error("build error: {0}")]
    BuildError(#[from] deadpool_postgres::BuildError),
}

pub fn connect(uri: &str) -> Result<Pool, ConnectionError> {
    let config = Config::from_str(uri)?;
    let manager = ManagerConfig {
        recycling_method: RecyclingMethod::Fast,
    };

    let manager = Manager::from_config(config, NoTls, manager);
    let pool = Pool::builder(manager)
        .max_size(16)
        .build()?;
    
    Ok(pool)
}
