use sqlx::SqlitePool;
use async_trait::async_trait;
use crate::db::models::{Submission, NewSubmission, DailyScore};
use anyhow::Result;

#[async_trait]
pub trait SubmissionRepository: Send + Sync {
    async fn insert(&self, submission: NewSubmission) -> Result<Submission>;
    async fn get_by_id(&self, id: i64) -> Result<Option<Submission>>;
    async fn get_by_exercise_id(&self, exercise_id: i64) -> Result<Vec<Submission>>;
    async fn get_best_score_for_exercise(&self, exercise_id: i64) -> Result<Option<Submission>>;
    async fn get_daily_submissions(&self, date: &str) -> Result<Vec<Submission>>;
    async fn get_recent_scores(&self, days: i32) -> Result<Vec<DailyScore>>;
    async fn count_by_language_and_difficulty(&self, language: &str, difficulty: &str) -> Result<i64>;
}

pub struct SqliteSubmissionRepository {
    pool: SqlitePool,
}

impl SqliteSubmissionRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl SubmissionRepository for SqliteSubmissionRepository {
    async fn insert(&self, submission: NewSubmission) -> Result<Submission> {
        sqlx::query_as::<_, Submission>(
            "INSERT INTO submissions (exercise_id, user_code, score, letter_grade, is_partial, strengths, improvements, summary) VALUES (?, ?, ?, ?, ?, ?, ?, ?) RETURNING id, exercise_id, user_code, score, letter_grade, is_partial, strengths, improvements, summary, submitted_at"
        )
        .bind(submission.exercise_id)
        .bind(submission.user_code)
        .bind(submission.score)
        .bind(submission.letter_grade)
        .bind(submission.is_partial)
        .bind(submission.strengths)
        .bind(submission.improvements)
        .bind(submission.summary)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to insert submission: {}", e))
    }

    async fn get_by_id(&self, id: i64) -> Result<Option<Submission>> {
        sqlx::query_as::<_, Submission>(
            "SELECT id, exercise_id, user_code, score, letter_grade, is_partial, strengths, improvements, summary, submitted_at FROM submissions WHERE id = ?"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to fetch submission: {}", e))
    }

    async fn get_by_exercise_id(&self, exercise_id: i64) -> Result<Vec<Submission>> {
        sqlx::query_as::<_, Submission>(
            "SELECT id, exercise_id, user_code, score, letter_grade, is_partial, strengths, improvements, summary, submitted_at FROM submissions WHERE exercise_id = ? ORDER BY submitted_at DESC"
        )
        .bind(exercise_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to fetch submissions by exercise: {}", e))
    }

    async fn get_best_score_for_exercise(&self, exercise_id: i64) -> Result<Option<Submission>> {
        sqlx::query_as::<_, Submission>(
            "SELECT id, exercise_id, user_code, score, letter_grade, is_partial, strengths, improvements, summary, submitted_at FROM submissions WHERE exercise_id = ? ORDER BY score DESC LIMIT 1"
        )
        .bind(exercise_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to fetch best score for exercise: {}", e))
    }

    async fn get_daily_submissions(&self, date: &str) -> Result<Vec<Submission>> {
        sqlx::query_as::<_, Submission>(
            "SELECT id, exercise_id, user_code, score, letter_grade, is_partial, strengths, improvements, summary, submitted_at FROM submissions WHERE DATE(submitted_at) = ? ORDER BY submitted_at DESC"
        )
        .bind(date)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to fetch daily submissions: {}", e))
    }

    async fn get_recent_scores(&self, days: i32) -> Result<Vec<DailyScore>> {
        sqlx::query_as::<_, DailyScore>(
            "SELECT DATE(submitted_at) as date, AVG(CAST(score AS REAL)) as avg_score, COUNT(*) as count FROM submissions WHERE submitted_at >= datetime('now', '-' || ? || ' days') GROUP BY DATE(submitted_at) ORDER BY date ASC"
        )
        .bind(days)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to fetch recent scores: {}", e))
    }

    async fn count_by_language_and_difficulty(&self, language: &str, difficulty: &str) -> Result<i64> {
        let count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM submissions s JOIN exercises e ON s.exercise_id = e.id WHERE e.language = ? AND e.difficulty = ?"
        )
        .bind(language)
        .bind(difficulty)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to count submissions by language and difficulty: {}", e))?;
        Ok(count)
    }
}