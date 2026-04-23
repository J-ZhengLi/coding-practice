use axum::{
    extract::State,
    response::IntoResponse,
    Json, http::StatusCode,
};
use crate::config::{ConfigService, ConfigRequest, UserConfig};
use crate::error::Result;
use crate::api::OllamaService;
use crate::material::fetcher::MaterialService;

type AppState = (
    std::sync::Arc<ConfigService<crate::db::repository::SqliteConfigRepository>>,
    std::sync::Arc<OllamaService>,
    std::sync::Arc<MaterialService>,
);

pub async fn get_config_handler(
    State((config_service, _, _)): State<AppState>,
) -> Result<impl IntoResponse> {
    let config = config_service.get_config().await?;
    Ok((StatusCode::OK, Json(config)))
}

pub async fn save_config_handler(
    State((config_service, _, _)): State<AppState>,
    Json(request): Json<ConfigRequest>,
) -> Result<impl IntoResponse> {
    let config = UserConfig::from(request);
    config_service.save_config(&config).await?;
    Ok((StatusCode::OK, Json(serde_json::json!({"status": "success"}))))
}

pub async fn check_configured_handler(
    State((config_service, _, _)): State<AppState>,
) -> Result<impl IntoResponse> {
    let is_configured = config_service.is_configured().await?;
    Ok((StatusCode::OK, Json(serde_json::json!({"configured": is_configured}))))
}