use sqlx::SqlitePool;
use async_trait::async_trait;
use crate::db::models::Config;
use anyhow::Result;

#[async_trait]
pub trait ConfigRepository: Send + Sync {
    async fn get(&self, key: &str) -> Result<Option<Config>>;
    async fn set(&self, key: &str, value: &str) -> Result<()>;
    async fn get_all(&self) -> Result<Vec<Config>>;
    async fn exists(&self) -> Result<bool>;
    async fn delete(&self, key: &str) -> Result<()>;
}

pub struct SqliteConfigRepository {
    pool: SqlitePool,
}

impl SqliteConfigRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ConfigRepository for SqliteConfigRepository {
    async fn get(&self, key: &str) -> Result<Option<Config>> {
        sqlx::query_as::<_, Config>(
            "SELECT key, value, updated_at FROM config WHERE key = ?"
        )
        .bind(key)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to fetch config: {}", e))
    }

    async fn set(&self, key: &str, value: &str) -> Result<()> {
        sqlx::query(
            "INSERT INTO config (key, value, updated_at) VALUES (?, ?, datetime('now'))
             ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = datetime('now')"
        )
        .bind(key)
        .bind(value)
        .execute(&self.pool)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to save config: {}", e))?;
        Ok(())
    }

    async fn get_all(&self) -> Result<Vec<Config>> {
        sqlx::query_as::<_, Config>("SELECT key, value, updated_at FROM config")
            .fetch_all(&self.pool)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to fetch all configs: {}", e))
    }

    async fn exists(&self) -> Result<bool> {
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM config")
            .fetch_one(&self.pool)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to check config existence: {}", e))?;
        Ok(count > 0)
    }

    async fn delete(&self, key: &str) -> Result<()> {
        sqlx::query("DELETE FROM config WHERE key = ?")
            .bind(key)
            .execute(&self.pool)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to delete config: {}", e))?;
        Ok(())
    }
}