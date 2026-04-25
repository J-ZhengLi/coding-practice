use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::Deserialize;

use crate::error::{AppError, Result};
use crate::exercise::models::{
    ExerciseResponse, GenerateRequest, GenerateResponse, validate_language, validate_difficulty,
};

use super::AppState;

#[derive(Debug, Deserialize)]
pub struct ExerciseQueryParams {
    pub language: Option<String>,
    pub difficulty: Option<String>,
}

/// GET /api/exercises - List exercises with optional language/difficulty filtering.
pub async fn get_exercises_handler(
    State((_, _, _, exercise_service, _, _)): State<AppState>,
    Query(params): Query<ExerciseQueryParams>,
) -> Result<impl IntoResponse> {
    let language = params.language.as_deref();
    let difficulty = params.difficulty.as_deref();

    // Validate params if provided (T-02-16)
    if let Some(lang) = language {
        validate_language(lang).map_err(AppError::Validation)?;
    }
    if let Some(diff) = difficulty {
        validate_difficulty(diff).map_err(AppError::Validation)?;
    }

    let exercises = match (language, difficulty) {
        (Some(lang), Some(diff)) => {
            exercise_service.get_exercises(lang, diff).await?
        }
        (Some(lang), None) => {
            // Get all exercises for a language regardless of difficulty
            exercise_service.exercise_repo()
                .get_by_language(lang)
                .await
                .map_err(|e| AppError::Internal(e))?
        }
        _ => {
            // Get all exercises
            exercise_service.exercise_repo()
                .get_all()
                .await
                .map_err(|e| AppError::Internal(e))?
        }
    };

    let responses: Vec<ExerciseResponse> =
        exercises.into_iter().map(ExerciseResponse::from).collect();

    Ok((StatusCode::OK, Json(responses)))
}

/// POST /api/exercises/generate - Trigger exercise generation for a language and difficulty.
pub async fn generate_exercises_handler(
    State((_, _, _, exercise_service, _, _)): State<AppState>,
    Json(request): Json<GenerateRequest>,
) -> Result<impl IntoResponse> {
    // Validate language and difficulty (T-02-16)
    validate_language(&request.language).map_err(AppError::Validation)?;
    validate_difficulty(&request.difficulty).map_err(AppError::Validation)?;

    // Check if exercises are already cached
    let cached = exercise_service
        .exercise_repo()
        .get_by_language_and_difficulty(&request.language, &request.difficulty)
        .await
        .map_err(|e| AppError::Internal(e))?;

    if !cached.is_empty() {
        // Return cached exercises (D-09)
        let responses: Vec<ExerciseResponse> =
            cached.into_iter().map(ExerciseResponse::from).collect();
        return Ok((StatusCode::OK, Json(GenerateResponse {
            exercises: responses.clone(),
            generated_count: responses.len(),
            cached: true,
        })));
    }

    // Generate new exercises
    let exercises = exercise_service
        .generate_exercises_for_language(&request.language, &request.difficulty)
        .await?;

    let responses: Vec<ExerciseResponse> =
        exercises.into_iter().map(ExerciseResponse::from).collect();
    let generated_count = responses.len();

    Ok((StatusCode::OK, Json(GenerateResponse {
        exercises: responses,
        generated_count,
        cached: false,
    })))
}

/// GET /api/exercises/:id - Get a single exercise by ID.
pub async fn get_exercise_by_id_handler(
    State((_, _, _, exercise_service, _, _)): State<AppState>,
    Path(id): Path<i64>,
) -> Result<impl IntoResponse> {
    let exercise = exercise_service
        .get_exercise_by_id(id)
        .await?
        .ok_or_else(|| AppError::Validation(format!("Exercise {} not found", id)))?;

    Ok((StatusCode::OK, Json(ExerciseResponse::from(exercise))))
}

/// DELETE /api/exercises/:id - Delete an exercise by ID.
pub async fn delete_exercise_handler(
    State((_, _, _, exercise_service, _, _)): State<AppState>,
    Path(id): Path<i64>,
) -> Result<impl IntoResponse> {
    exercise_service.delete_exercise(id).await?;

    Ok((StatusCode::OK, Json(serde_json::json!({
        "status": "deleted",
        "id": id
    }))))
}