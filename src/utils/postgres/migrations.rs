use std::path::Path;
use std::process::ExitStatus;

pub async fn golang_migrate(database_url: &str, migrations_dir: &Path) -> ExitStatus {
    use std::process::Command;
    
    Command::new("migrate")
        .args(&[
            "-path",
            migrations_dir.to_str().unwrap(),
            "-database",
            database_url,
            "up",
        ])
        .status()
        .expect("failed to run migrations")
}
