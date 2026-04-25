use axum::{
    extract::State,
    response::IntoResponse,
    Json, http::StatusCode,
    body::Bytes,
};
use crate::config::{ConfigService, ConfigRequest, UserConfig};
use crate::error::{AppError, Result};

use super::AppState;

pub async fn get_config_handler(
    State((config_service, _, _, _, _, _)): State<AppState>,
) -> Result<impl IntoResponse> {
    let config = config_service.get_config().await?;
    Ok((StatusCode::OK, Json(config)))
}

pub async fn save_config_handler(
    State((config_service, _, _, _, _, _)): State<AppState>,
    body: Bytes,
) -> Result<impl IntoResponse> {
    tracing::info!("Config save request received ({} bytes)", body.len());
    tracing::debug!("Raw config request body: {}", String::from_utf8_lossy(&body));

    let request: ConfigRequest = serde_json::from_slice(&body)
        .map_err(|e| {
            tracing::error!("Config request deserialization failed: {}", e);
            AppError::Validation(format!("Invalid request body: {}", e))
        })?;

    let config = UserConfig::from(request);
    config_service.save_config(&config).await?;
    Ok((StatusCode::OK, Json(serde_json::json!({"status": "success"}))))
}

pub async fn check_configured_handler(
    State((config_service, _, _, _, _, _)): State<AppState>,
) -> Result<impl IntoResponse> {
    let is_configured = config_service.is_configured().await?;
    Ok((StatusCode::OK, Json(serde_json::json!({"configured": is_configured}))))
}