use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};
use std::path::PathBuf;
use std::time::Duration;
use anyhow::Result;

pub fn get_database_path() -> PathBuf {
    if let Some(base_dir) = dirs::data_local_dir() {
        base_dir.join("coding-practice").join("app.db")
    } else {
        PathBuf::from("app.db")
    }
}

pub async fn create_pool(database_path: Option<PathBuf>) -> Result<SqlitePool> {
    let db_path = database_path.unwrap_or_else(get_database_path);

    // Ensure parent directory exists
    if let Some(parent) = db_path.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }

    // Create database URL with rwc mode (read/write/create)
    let database_url = format!(
        "sqlite:{}?mode=rwc",
        db_path.display()
    );

    // Create connection pool
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(30))
        .connect(&database_url)
        .await?;

    // Run migrations
    sqlx::migrate!("./migrations").run(&pool).await?;

    Ok(pool)
}