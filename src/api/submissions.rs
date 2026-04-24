use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::Deserialize;

use crate::error::{AppError, Result};
use crate::exercise::service::ExerciseService;
use crate::submission::models::{SubmitCodeRequest, SubmissionResponse};

use super::AppState;

#[derive(Debug, Deserialize)]
pub struct SubmissionQueryParams {
    pub exercise_id: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct ProgressQueryParams {
    pub date: Option<String>,
    pub language: Option<String>,
    pub difficulty: Option<String>,
    pub days: Option<i32>,
}

/// POST /api/submissions - Submit code for evaluation (D-05, EX-05).
pub async fn submit_code_handler(
    State(state): State<AppState>,
    Json(request): Json<SubmitCodeRequest>,
) -> Result<impl IntoResponse> {
    let (config_service, _, _, exercise_service, submission_service) = state;

    // Validate exercise exists (T-03-08: validate exercise_id before processing)
    let _exercise = exercise_service
        .get_exercise_by_id(request.exercise_id)
        .await?
        .ok_or_else(|| {
            AppError::Validation(format!("Exercise {} not found", request.exercise_id))
        })?;

    // Validate non-empty code (T-03-08: reject empty user_code)
    if request.code.trim().is_empty() {
        return Err(AppError::Validation("Code cannot be empty".to_string()));
    }

    // Create AI provider from user config (same pattern as ExerciseService)
    let config = config_service
        .get_config()
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!("Config error: {}", e)))?;
    let provider = ExerciseService::create_provider_from_config(&config);

    let submission = submission_service
        .submit_code(
            request.exercise_id,
            &request.code,
            exercise_service.exercise_repo().as_ref(),
            provider.as_ref(),
        )
        .await?;

    let response = SubmissionResponse::from(submission);
    Ok((StatusCode::CREATED, Json(response)))
}

/// GET /api/submissions/:id - Get a single submission by ID (EX-06, EX-08).
pub async fn get_submission_handler(
    State((_, _, _, _, submission_service)): State<AppState>,
    Path(id): Path<i64>,
) -> Result<impl IntoResponse> {
    let submission = submission_service
        .get_submission(id)
        .await?
        .ok_or_else(|| AppError::Validation(format!("Submission {} not found", id)))?;

    Ok((StatusCode::OK, Json(SubmissionResponse::from(submission))))
}

/// GET /api/submissions?exercise_id=X - Get submissions for an exercise (EX-08).
pub async fn get_submissions_handler(
    State((_, _, _, _, submission_service)): State<AppState>,
    Query(params): Query<SubmissionQueryParams>,
) -> Result<impl IntoResponse> {
    let submissions = match params.exercise_id {
        Some(exercise_id) => {
            submission_service
                .get_submissions_by_exercise(exercise_id)
                .await?
        }
        None => {
            return Err(AppError::Validation(
                "exercise_id query parameter is required".to_string(),
            ));
        }
    };

    let responses: Vec<SubmissionResponse> =
        submissions.into_iter().map(SubmissionResponse::from).collect();

    Ok((StatusCode::OK, Json(responses)))
}

/// GET /api/submissions/:id/solution - View solution comparison (D-08, WEB-11).
pub async fn get_solution_handler(
    State((_, _, _, exercise_service, submission_service)): State<AppState>,
    Path(id): Path<i64>,
) -> Result<impl IntoResponse> {
    let solution = submission_service
        .get_solution(id, exercise_service.exercise_repo().as_ref())
        .await?;
    Ok((StatusCode::OK, Json(solution)))
}

/// GET /api/progress/daily - Get daily progress metrics (D-11, SCORE-03, SCORE-04).
pub async fn get_daily_progress_handler(
    State((_, _, _, _, submission_service)): State<AppState>,
    Query(params): Query<ProgressQueryParams>,
) -> Result<impl IntoResponse> {
    let date = params.date.unwrap_or_else(|| {
        chrono::Utc::now().format("%Y-%m-%d").to_string()
    });
    let progress = submission_service.get_daily_progress(&date).await?;
    Ok((StatusCode::OK, Json(progress)))
}

/// GET /api/progress/trend - Get 7-day score trend (D-11).
pub async fn get_score_trend_handler(
    State((_, _, _, _, submission_service)): State<AppState>,
    Query(params): Query<ProgressQueryParams>,
) -> Result<impl IntoResponse> {
    let days = params.days.unwrap_or(7);
    let trend = submission_service.get_score_trend(days).await?;
    Ok((StatusCode::OK, Json(trend)))
}