use chrono::{DateTime, Utc};

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