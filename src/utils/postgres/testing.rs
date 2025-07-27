use crate::utils::postgres::migrations::golang_migrate;
use deadpool_postgres::{Config, ManagerConfig, Pool, RecyclingMethod, Runtime};
use std::path::{Path, PathBuf};
use tokio_postgres::NoTls;

fn migrations_uri() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("migrations")
}

pub async fn test_db_setup() -> Pool {
    let uri = std::env::var("DATABASE_URI").expect("Environment variable 'DATABASE_URI' not set");
    let mut cfg = Config::new();
    cfg.url = Some(
        uri.parse()
            .expect("failed to parse database connection string"),
    );

    cfg.manager = Some(ManagerConfig {
        recycling_method: RecyclingMethod::Fast,
    });

    let pool = cfg
        .create_pool(Some(Runtime::Tokio1), NoTls)
        .expect("failed to create pool");

    let exit_code = golang_migrate(&uri, &migrations_uri()).await;
    assert!(exit_code.success());

    pool
}
