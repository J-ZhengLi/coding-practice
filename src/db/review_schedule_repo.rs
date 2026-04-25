use chrono::NaiveDateTime;
use sqlx::SqlitePool;
use async_trait::async_trait;
use crate::db::models::{ReviewSchedule, NewReviewSchedule};
use anyhow::Result;

#[async_trait]
pub trait ReviewScheduleRepository: Send + Sync {
    async fn insert(&self, schedule: NewReviewSchedule) -> Result<ReviewSchedule>;
    async fn get_by_concept_and_language(&self, concept: &str, language: &str) -> Result<Option<ReviewSchedule>>;
    async fn get_due_reviews(&self, language: &str, today_utc: &NaiveDateTime) -> Result<Vec<ReviewSchedule>>;
    async fn update_schedule(&self, id: i64, current_interval: i32, last_completed_at: NaiveDateTime, next_review_at: NaiveDateTime, last_exercise_id: i64, status: &str) -> Result<ReviewSchedule>;
    async fn retire_concept(&self, id: i64) -> Result<()>;
    async fn get_all_active(&self) -> Result<Vec<ReviewSchedule>>;
    async fn count_completed(&self) -> Result<i64>;
    async fn get_all_by_language(&self, language: &str) -> Result<Vec<ReviewSchedule>>;
}

pub struct SqliteReviewScheduleRepository {
    pool: SqlitePool,
}

impl SqliteReviewScheduleRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ReviewScheduleRepository for SqliteReviewScheduleRepository {
    async fn insert(&self, schedule: NewReviewSchedule) -> Result<ReviewSchedule> {
        sqlx::query_as::<_, ReviewSchedule>(
            "INSERT INTO review_schedule (concept, language, current_interval, last_completed_at, next_review_at, last_exercise_id, status) VALUES (?, ?, ?, ?, ?, ?, ?) RETURNING id, concept, language, current_interval, last_completed_at, next_review_at, last_exercise_id, status"
        )
        .bind(schedule.concept)
        .bind(schedule.language)
        .bind(schedule.current_interval)
        .bind(schedule.last_completed_at)
        .bind(schedule.next_review_at)
        .bind(schedule.last_exercise_id)
        .bind(schedule.status)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to insert review schedule: {}", e))
    }

    async fn get_by_concept_and_language(&self, concept: &str, language: &str) -> Result<Option<ReviewSchedule>> {
        sqlx::query_as::<_, ReviewSchedule>(
            "SELECT id, concept, language, current_interval, last_completed_at, next_review_at, last_exercise_id, status FROM review_schedule WHERE concept = ? AND language = ?"
        )
        .bind(concept)
        .bind(language)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to fetch review schedule by concept and language: {}", e))
    }

    async fn get_due_reviews(&self, language: &str, today_utc: &NaiveDateTime) -> Result<Vec<ReviewSchedule>> {
        sqlx::query_as::<_, ReviewSchedule>(
            "SELECT id, concept, language, current_interval, last_completed_at, next_review_at, last_exercise_id, status FROM review_schedule WHERE language = ? AND status = 'active' AND next_review_at <= ?"
        )
        .bind(language)
        .bind(today_utc)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to fetch due reviews: {}", e))
    }

    async fn update_schedule(&self, id: i64, current_interval: i32, last_completed_at: NaiveDateTime, next_review_at: NaiveDateTime, last_exercise_id: i64, status: &str) -> Result<ReviewSchedule> {
        sqlx::query_as::<_, ReviewSchedule>(
            "UPDATE review_schedule SET current_interval = ?, last_completed_at = ?, next_review_at = ?, last_exercise_id = ?, status = ? WHERE id = ? RETURNING id, concept, language, current_interval, last_completed_at, next_review_at, last_exercise_id, status"
        )
        .bind(current_interval)
        .bind(last_completed_at)
        .bind(next_review_at)
        .bind(last_exercise_id)
        .bind(status)
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to update review schedule: {}", e))
    }

    async fn retire_concept(&self, id: i64) -> Result<()> {
        sqlx::query(
            "UPDATE review_schedule SET status = 'completed' WHERE id = ?"
        )
        .bind(id)
        .execute(&self.pool)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to retire concept: {}", e))?;
        Ok(())
    }

    async fn get_all_active(&self) -> Result<Vec<ReviewSchedule>> {
        sqlx::query_as::<_, ReviewSchedule>(
            "SELECT id, concept, language, current_interval, last_completed_at, next_review_at, last_exercise_id, status FROM review_schedule WHERE status = 'active'"
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to fetch active schedules: {}", e))
    }

    async fn count_completed(&self) -> Result<i64> {
        let count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM review_schedule WHERE status = 'completed'"
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to count completed schedules: {}", e))?;
        Ok(count)
    }

    async fn get_all_by_language(&self, language: &str) -> Result<Vec<ReviewSchedule>> {
        sqlx::query_as::<_, ReviewSchedule>(
            "SELECT id, concept, language, current_interval, last_completed_at, next_review_at, last_exercise_id, status FROM review_schedule WHERE language = ?"
        )
        .bind(language)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to fetch all schedules by language: {}", e))
    }
}