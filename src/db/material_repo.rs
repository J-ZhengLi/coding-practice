use sqlx::SqlitePool;
use async_trait::async_trait;
use crate::db::models::{Material, NewMaterial};
use anyhow::Result;

#[async_trait]
pub trait MaterialRepository: Send + Sync {
    async fn get_by_id(&self, id: i64) -> Result<Option<Material>>;
    async fn get_by_language(&self, language: &str) -> Result<Vec<Material>>;
    async fn get_by_language_and_difficulty(&self, language: &str, difficulty: &str) -> Result<Vec<Material>>;
    async fn get_by_source_url(&self, url: &str) -> Result<Option<Material>>;
    async fn insert(&self, material: NewMaterial) -> Result<Material>;
    async fn delete(&self, id: i64) -> Result<()>;
    async fn get_all(&self) -> Result<Vec<Material>>;
}

pub struct SqliteMaterialRepository {
    pool: SqlitePool,
}

impl SqliteMaterialRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl MaterialRepository for SqliteMaterialRepository {
    async fn get_by_id(&self, id: i64) -> Result<Option<Material>> {
        sqlx::query_as::<_, Material>(
            "SELECT id, source_url, source_type, language, title, difficulty, local_path, fetched_at FROM materials WHERE id = ?"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to fetch material: {}", e))
    }

    async fn get_by_language(&self, language: &str) -> Result<Vec<Material>> {
        sqlx::query_as::<_, Material>(
            "SELECT id, source_url, source_type, language, title, difficulty, local_path, fetched_at FROM materials WHERE language = ?"
        )
        .bind(language)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to fetch materials by language: {}", e))
    }

    async fn get_by_language_and_difficulty(&self, language: &str, difficulty: &str) -> Result<Vec<Material>> {
        sqlx::query_as::<_, Material>(
            "SELECT id, source_url, source_type, language, title, difficulty, local_path, fetched_at FROM materials WHERE language = ? AND difficulty = ?"
        )
        .bind(language)
        .bind(difficulty)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to fetch materials by language and difficulty: {}", e))
    }

    async fn get_by_source_url(&self, url: &str) -> Result<Option<Material>> {
        sqlx::query_as::<_, Material>(
            "SELECT id, source_url, source_type, language, title, difficulty, local_path, fetched_at FROM materials WHERE source_url = ?"
        )
        .bind(url)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to fetch material by source URL: {}", e))
    }

    async fn insert(&self, material: NewMaterial) -> Result<Material> {
        sqlx::query_as::<_, Material>(
            "INSERT INTO materials (source_url, source_type, language, title, difficulty, local_path) VALUES (?, ?, ?, ?, ?, ?) RETURNING id, source_url, source_type, language, title, difficulty, local_path, fetched_at"
        )
        .bind(material.source_url)
        .bind(material.source_type)
        .bind(material.language)
        .bind(material.title)
        .bind(material.difficulty)
        .bind(material.local_path)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to insert material: {}", e))
    }

    async fn delete(&self, id: i64) -> Result<()> {
        sqlx::query("DELETE FROM materials WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to delete material: {}", e))?;
        Ok(())
    }

    async fn get_all(&self) -> Result<Vec<Material>> {
        sqlx::query_as::<_, Material>(
            "SELECT id, source_url, source_type, language, title, difficulty, local_path, fetched_at FROM materials"
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to fetch all materials: {}", e))
    }
}