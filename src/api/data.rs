use axum::{
    extract::State,
    http::{HeaderValue, header},
    response::IntoResponse,
    Json,
};
use crate::data_export::DataExportService;
use crate::error::{AppError, Result};

use super::AppState;

/// GET /api/data/export — Download all tables as a JSON file.
pub async fn export_data_handler(
    State((_, _, _, _, _, _, _, data_export_service)): State<AppState>,
) -> Result<impl IntoResponse> {
    let data = data_export_service.export_all().await
        .map_err(|e| AppError::Internal(e))?;

    let json_bytes = serde_json::to_vec_pretty(&data)
        .map_err(|e| AppError::Internal(anyhow::anyhow!("Failed to serialize export: {}", e)))?;

    Ok((
        [
            (header::CONTENT_TYPE, HeaderValue::from_static("application/json")),
            (header::CONTENT_DISPOSITION, HeaderValue::from_static("attachment; filename=\"coding-practice-export.json\"")),
        ],
        json_bytes,
    ))
}

/// POST /api/data/import — Replace all data from an uploaded JSON file.
pub async fn import_data_handler(
    State((_, _, _, _, _, _, _, data_export_service)): State<AppState>,
    Json(body): Json<serde_json::Value>,
) -> Result<impl IntoResponse> {
    data_export_service.import_all(&body).await
        .map_err(|e| AppError::Internal(e))?;

    Ok(Json(serde_json::json!({ "status": "success" })))
}