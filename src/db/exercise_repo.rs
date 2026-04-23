use sqlx::SqlitePool;
use async_trait::async_trait;
use crate::db::models::{Exercise, NewExercise};
use anyhow::Result;

#[async_trait]
pub trait ExerciseRepository: Send + Sync {
    async fn get_by_id(&self, id: i64) -> Result<Option<Exercise>>;
    async fn get_by_language(&self, language: &str) -> Result<Vec<Exercise>>;
    async fn get_by_language_and_difficulty(&self, language: &str, difficulty: &str) -> Result<Vec<Exercise>>;
    async fn get_by_material_id(&self, material_id: i64) -> Result<Vec<Exercise>>;
    async fn insert(&self, exercise: NewExercise) -> Result<Exercise>;
    async fn delete(&self, id: i64) -> Result<()>;
    async fn get_all(&self) -> Result<Vec<Exercise>>;
    async fn count_by_language_and_difficulty(&self, language: &str, difficulty: &str) -> Result<i64>;
}

pub struct SqliteExerciseRepository {
    pool: SqlitePool,
}

impl SqliteExerciseRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ExerciseRepository for SqliteExerciseRepository {
    async fn get_by_id(&self, id: i64) -> Result<Option<Exercise>> {
        sqlx::query_as::<_, Exercise>(
            "SELECT id, material_id, title, description, language, difficulty, todo_comment, original_code, exercise_code, concept, start_line, end_line, generated_at FROM exercises WHERE id = ?"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to fetch exercise: {}", e))
    }

    async fn get_by_language(&self, language: &str) -> Result<Vec<Exercise>> {
        sqlx::query_as::<_, Exercise>(
            "SELECT id, material_id, title, description, language, difficulty, todo_comment, original_code, exercise_code, concept, start_line, end_line, generated_at FROM exercises WHERE language = ?"
        )
        .bind(language)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to fetch exercises by language: {}", e))
    }

    async fn get_by_language_and_difficulty(&self, language: &str, difficulty: &str) -> Result<Vec<Exercise>> {
        sqlx::query_as::<_, Exercise>(
            "SELECT id, material_id, title, description, language, difficulty, todo_comment, original_code, exercise_code, concept, start_line, end_line, generated_at FROM exercises WHERE language = ? AND difficulty = ?"
        )
        .bind(language)
        .bind(difficulty)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to fetch exercises by language and difficulty: {}", e))
    }

    async fn get_by_material_id(&self, material_id: i64) -> Result<Vec<Exercise>> {
        sqlx::query_as::<_, Exercise>(
            "SELECT id, material_id, title, description, language, difficulty, todo_comment, original_code, exercise_code, concept, start_line, end_line, generated_at FROM exercises WHERE material_id = ?"
        )
        .bind(material_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to fetch exercises by material: {}", e))
    }

    async fn insert(&self, exercise: NewExercise) -> Result<Exercise> {
        sqlx::query_as::<_, Exercise>(
            "INSERT INTO exercises (material_id, title, description, language, difficulty, todo_comment, original_code, exercise_code, concept, start_line, end_line) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?) RETURNING id, material_id, title, description, language, difficulty, todo_comment, original_code, exercise_code, concept, start_line, end_line, generated_at"
        )
        .bind(exercise.material_id)
        .bind(exercise.title)
        .bind(exercise.description)
        .bind(exercise.language)
        .bind(exercise.difficulty)
        .bind(exercise.todo_comment)
        .bind(exercise.original_code)
        .bind(exercise.exercise_code)
        .bind(exercise.concept)
        .bind(exercise.start_line)
        .bind(exercise.end_line)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to insert exercise: {}", e))
    }

    async fn delete(&self, id: i64) -> Result<()> {
        sqlx::query("DELETE FROM exercises WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to delete exercise: {}", e))?;
        Ok(())
    }

    async fn get_all(&self) -> Result<Vec<Exercise>> {
        sqlx::query_as::<_, Exercise>(
            "SELECT id, material_id, title, description, language, difficulty, todo_comment, original_code, exercise_code, concept, start_line, end_line, generated_at FROM exercises"
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to fetch all exercises: {}", e))
    }

    async fn count_by_language_and_difficulty(&self, language: &str, difficulty: &str) -> Result<i64> {
        let count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM exercises WHERE language = ? AND difficulty = ?"
        )
        .bind(language)
        .bind(difficulty)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to count exercises: {}", e))?;
        Ok(count)
    }
}