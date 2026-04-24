use serde::{Deserialize, Serialize};
use crate::db::models::Submission;

/// API response for a single submission.
/// Omits user_code for list views (use get_submission for full detail).
/// Per security requirement: original_code is NOT included here - only accessible via solution endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubmissionResponse {
    pub id: i64,
    pub exercise_id: i64,
    pub score: i32,
    pub letter_grade: String,
    pub is_partial: bool,
    pub strengths: Vec<String>,
    pub improvements: Vec<String>,
    pub summary: String,
    pub submitted_at: String,
}

impl From<Submission> for SubmissionResponse {
    fn from(sub: Submission) -> Self {
        Self {
            id: sub.id,
            exercise_id: sub.exercise_id,
            score: sub.score,
            letter_grade: sub.letter_grade,
            is_partial: sub.is_partial,
            strengths: serde_json::from_str(&sub.strengths).unwrap_or_default(),
            improvements: serde_json::from_str(&sub.improvements).unwrap_or_default(),
            summary: sub.summary,
            submitted_at: sub.submitted_at.to_rfc3339(),
        }
    }
}

/// Request body for POST /api/submissions - submit completed code for evaluation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubmitCodeRequest {
    pub exercise_id: i64,
    pub code: String,
}

/// API response for viewing the solution (original code) of a submission.
/// Per D-08: solution available immediately after submission via dedicated endpoint.
/// Per Pitfall 3: this is the ONLY endpoint that returns original_code.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SolutionResponse {
    pub submission_id: i64,
    pub exercise_id: i64,
    pub user_code: String,
    pub original_code: String,
    pub language: String,
    pub score: i32,
    pub letter_grade: String,
}

/// Response for daily progress endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyProgressResponse {
    pub date: String,
    pub avg_score: Option<f64>,
    pub letter_grade: Option<String>,
    pub completion_count: i64,
}

/// Response for score trend endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoreTrendResponse {
    pub scores: Vec<DailyScoreEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyScoreEntry {
    pub date: String,
    pub avg_score: f64,
}