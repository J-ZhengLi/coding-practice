use chrono::{DateTime, Utc, NaiveDateTime};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, sqlx::FromRow)]
pub struct Config {
    pub key: String,
    pub value: String,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, sqlx::FromRow)]
pub struct Material {
    pub id: i64,
    pub source_url: String,
    pub source_type: String,
    pub language: String,
    pub title: String,
    pub difficulty: String,
    pub local_path: String,
    pub fetched_at: DateTime<Utc>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct NewMaterial {
    pub source_url: String,
    pub source_type: String,
    pub language: String,
    pub title: String,
    pub difficulty: String,
    pub local_path: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, sqlx::FromRow)]
pub struct Exercise {
    pub id: i64,
    pub material_id: i64,
    pub title: String,
    pub description: String,
    pub language: String,
    pub difficulty: String,
    pub todo_comment: String,
    pub original_code: String,
    pub exercise_code: String,
    pub concept: String,
    pub start_line: i64,
    pub end_line: i64,
    pub generated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct NewExercise {
    pub material_id: i64,
    pub title: String,
    pub description: String,
    pub language: String,
    pub difficulty: String,
    pub todo_comment: String,
    pub original_code: String,
    pub exercise_code: String,
    pub concept: String,
    pub start_line: i64,
    pub end_line: i64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, sqlx::FromRow)]
pub struct Submission {
    pub id: i64,
    pub exercise_id: i64,
    pub user_code: String,
    pub score: i32,
    pub letter_grade: String,
    pub is_partial: bool,
    pub strengths: String,     // JSON array stored as TEXT
    pub improvements: String,  // JSON array stored as TEXT
    pub summary: String,
    pub submitted_at: DateTime<Utc>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct NewSubmission {
    pub exercise_id: i64,
    pub user_code: String,
    pub score: i32,
    pub letter_grade: String,
    pub is_partial: bool,
    pub strengths: String,
    pub improvements: String,
    pub summary: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, sqlx::FromRow)]
pub struct DailyScore {
    pub date: String,
    pub avg_score: f64,
    pub count: i64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, sqlx::FromRow)]
pub struct ReviewSchedule {
    pub id: i64,
    pub concept: String,
    pub language: String,
    pub current_interval: i32,
    pub last_completed_at: NaiveDateTime,
    pub next_review_at: NaiveDateTime,
    pub last_exercise_id: i64,
    pub status: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct NewReviewSchedule {
    pub concept: String,
    pub language: String,
    pub current_interval: i32,
    pub last_completed_at: NaiveDateTime,
    pub next_review_at: NaiveDateTime,
    pub last_exercise_id: i64,
    pub status: String,
}