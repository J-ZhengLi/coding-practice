use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewScheduleResponse {
    pub id: i64,
    pub concept: String,
    pub language: String,
    pub current_interval: i32,
    pub last_completed_at: String,
    pub next_review_at: String,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyPlanExercise {
    pub id: i64,
    pub title: String,
    pub description: String,
    pub language: String,
    pub difficulty: String,
    pub todo_comment: String,
    pub exercise_code: String,
    pub concept: String,
    pub is_review: bool,
    pub review_interval: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyPlanResponse {
    pub exercises: Vec<DailyPlanExercise>,
    pub summary: DailyPlanSummary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyPlanSummary {
    pub new_count: i32,
    pub review_count: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleStatusResponse {
    pub active_concepts: i32,
    pub completed_concepts: i32,
    pub overdue_reviews: i32,
}