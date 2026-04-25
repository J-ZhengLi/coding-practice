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
        todo!()
    }

    async fn get_by_concept_and_language(&self, concept: &str, language: &str) -> Result<Option<ReviewSchedule>> {
        todo!()
    }

    async fn get_due_reviews(&self, language: &str, today_utc: &NaiveDateTime) -> Result<Vec<ReviewSchedule>> {
        todo!()
    }

    async fn update_schedule(&self, id: i64, current_interval: i32, last_completed_at: NaiveDateTime, next_review_at: NaiveDateTime, last_exercise_id: i64, status: &str) -> Result<ReviewSchedule> {
        todo!()
    }

    async fn retire_concept(&self, id: i64) -> Result<()> {
        todo!()
    }
}