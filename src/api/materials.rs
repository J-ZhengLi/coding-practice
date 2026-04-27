use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::material::fetcher::MaterialService;

use super::AppState;

#[derive(Debug, Deserialize)]
pub struct MaterialQueryParams {
    pub language: Option<String>,
    pub difficulty: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct FetchMaterialRequest {
    pub language: String,
    pub difficulty: String,
}

#[derive(Debug, Serialize)]
pub struct MaterialResponse {
    pub id: i64,
    pub source_url: String,
    pub source_type: String,
    pub language: String,
    pub title: String,
    pub difficulty: String,
    pub local_path: String,
    pub fetched_at: String,
}

impl From<crate::db::models::Material> for MaterialResponse {
    fn from(m: crate::db::models::Material) -> Self {
        Self {
            id: m.id,
            source_url: m.source_url,
            source_type: m.source_type,
            language: m.language,
            title: m.title,
            difficulty: m.difficulty,
            local_path: m.local_path,
            fetched_at: m.fetched_at.to_rfc3339(),
        }
    }
}

/// GET /api/materials - List cached materials with optional filtering.
pub async fn get_materials_handler(
    State((_, _, material_service, _, _, _, _)): State<AppState>,
    Query(params): Query<MaterialQueryParams>,
) -> Result<impl IntoResponse> {
    let language = params.language.as_deref();
    let difficulty = params.difficulty.as_deref();

    let materials = material_service
        .cache
        .list_materials(language, difficulty)
        .await
        .map_err(|e| crate::error::AppError::Material(e.to_string()))?;

    let responses: Vec<MaterialResponse> =
        materials.into_iter().map(MaterialResponse::from).collect();

    Ok((StatusCode::OK, Json(responses)))
}

/// POST /api/materials/fetch - Trigger material fetching for a language and difficulty.
pub async fn fetch_materials_handler(
    State((_, _, material_service, _, _, _, _)): State<AppState>,
    Json(request): Json<FetchMaterialRequest>,
) -> Result<impl IntoResponse> {
    let materials = material_service
        .fetch_materials(&request.language, &request.difficulty)
        .await
        .map_err(|e| crate::error::AppError::Material(e.to_string()))?;

    let responses: Vec<MaterialResponse> =
        materials.into_iter().map(MaterialResponse::from).collect();

    Ok((StatusCode::OK, Json(responses)))
}

/// POST /api/materials/:id/refresh - Refresh a specific cached material.
pub async fn refresh_material_handler(
    State((_, _, material_service, _, _, _, _)): State<AppState>,
    Path(id): Path<i64>,
) -> Result<impl IntoResponse> {
    let material = material_service
        .refresh_material(id)
        .await
        .map_err(|e| crate::error::AppError::Material(e.to_string()))?;

    Ok((StatusCode::OK, Json(MaterialResponse::from(material))))
}

/// DELETE /api/materials/:id - Delete a cached material.
pub async fn delete_material_handler(
    State((_, _, material_service, _, _, _, _)): State<AppState>,
    Path(id): Path<i64>,
) -> Result<impl IntoResponse> {
    // Delete from cache (DB + filesystem)
    material_service
        .cache
        .delete_material(id)
        .await
        .map_err(|e| crate::error::AppError::Material(e.to_string()))?;

    Ok((StatusCode::OK, Json(serde_json::json!({"status": "deleted", "id": id}))))
}