#[cfg(test)]
mod tests {
    use sqlx::SqlitePool;
    use crate::db::{ConfigRepository, SqliteConfigRepository, create_pool};

    #[tokio::test]
    async fn test_database_setup() {
        // Create a test database in memory
        let pool = create_pool(Some("test.db".into())).await.unwrap();

        // Test creating a repository
        let repo = SqliteConfigRepository::new(pool);

        // Test setting a config value
        repo.set("test_key", "test_value").await.unwrap();

        // Test getting the config value
        let config = repo.get("test_key").await.unwrap();
        assert!(config.is_some());
        assert_eq!(config.unwrap().key, "test_key");
        assert_eq!(config.unwrap().value, "test_value");

        // Test getting all configs
        let all = repo.get_all().await.unwrap();
        assert_eq!(all.len(), 1);
        assert_eq!(all[0].key, "test_key");

        // Test checking existence
        let exists = repo.exists().await.unwrap();
        assert!(exists);

        // Test deleting config
        repo.delete("test_key").await.unwrap();

        // Verify deletion
        let deleted = repo.get("test_key").await.unwrap();
        assert!(deleted.is_none());

        let exists_after = repo.exists().await.unwrap();
        assert!(!exists_after);
    }
}