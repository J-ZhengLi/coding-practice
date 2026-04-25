use sqlx::SqlitePool;
use coding_practice::db::{ConfigRepository, SqliteConfigRepository, create_pool};

#[tokio::main]
async fn main() {
    // Create a test database in the correct location
    use std::path::PathBuf;
    let db_path = dirs::data_local_dir()
        .unwrap()
        .join("coding-practice")
        .join("app.db");

    let pool = create_pool(Some(db_path)).await.unwrap();
    println!("✓ Database pool created");

    // Test creating a repository
    let repo = SqliteConfigRepository::new(pool);
    println!("✓ Repository created");

    // Test setting a config value
    repo.set("test_key", "test_value").await.unwrap();
    println!("✓ Config value set");

    // Test getting the config value
    let config = repo.get("test_key").await.unwrap();
    assert!(config.is_some());
    assert_eq!(config.unwrap().key, "test_key");
    println!("✓ Config value retrieved");

    // Test getting all configs
    let all = repo.get_all().await.unwrap();
    assert_eq!(all.len(), 1);
    println!("✓ All configs retrieved");

    // Test checking existence
    let exists = repo.exists().await.unwrap();
    assert!(exists);
    println!("✓ Config existence checked");

    // Test deleting config
    repo.delete("test_key").await.unwrap();

    // Verify deletion
    let deleted = repo.get("test_key").await.unwrap();
    assert!(deleted.is_none());

    let exists_after = repo.exists().await.unwrap();
    assert!(!exists_after);
    println!("✓ Config deleted and verified");

    println!("🎉 All database tests passed!");
}