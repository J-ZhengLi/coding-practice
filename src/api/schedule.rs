use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::Deserialize;

use crate::error::Result;
use crate::schedule::models::DailyPlanResponse;

use super::AppState;

#[derive(Debug, Deserialize)]
pub struct DailyPlanQueryParams {
    pub language: Option<String>,
}

/// GET /api/schedule/daily-plan - Get today's learning plan.
/// Per D-05: computed on-demand. Per D-06: reviews are extra on top of quota.
/// Per D-07: returns mixed list (frontend handles random interleaving).
/// Per D-01: review exercises are selected using last_exercise_id filtering.
/// Per D-03/D-04: mastered concepts excluded from new exercises.
pub async fn get_daily_plan_handler(
    State(state): State<AppState>,
    Query(params): Query<DailyPlanQueryParams>,
) -> Result<impl IntoResponse> {
    let (config_service, _, _, exercise_service, submission_service, schedule_service, _, _) = state;

    let daily_plan = schedule_service.get_daily_plan(
        &config_service,
        exercise_service.exercise_repo().as_ref(),
        submission_service.submission_repo() as &dyn crate::db::submission_repo::SubmissionRepository,
        params.language.as_deref(),
    ).await?;

    Ok((StatusCode::OK, Json(daily_plan)))
}

/// GET /api/schedule/status - Get review schedule status summary.
pub async fn get_schedule_status_handler(
    State((_, _, _, _, _, schedule_service, _, _)): State<AppState>,
) -> Result<impl IntoResponse> {
    let status = schedule_service.get_schedule_status().await?;
    Ok((StatusCode::OK, Json(status)))
}