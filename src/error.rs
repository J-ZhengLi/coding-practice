use tracing::error;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Ollama API error: {0}")]
    Ollama(String),

    #[error("Material error: {0}")]
    Material(String),

    #[error("AI error: {0}")]
    Ai(String),

    #[error("Submission error: {0}")]
    Submission(String),

    #[error("Evaluation error: {0}")]
    Evaluation(String),

    #[error("Not configured")]
    NotConfigured,

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Internal error: {0}")]
    Internal(#[from] anyhow::Error),
}

impl axum::response::IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        // Log full error details server-side for Database and Internal errors
        if let AppError::Database(e) = &self {
            error!("Database error: {}", e);
        }
        if let AppError::Internal(e) = &self {
            error!("Internal error: {}", e);
        }

        let (status, message) = match &self {
            AppError::Database(_) => (axum::http::StatusCode::INTERNAL_SERVER_ERROR, "Internal database error".to_string()),
            AppError::Config(msg) => (axum::http::StatusCode::BAD_REQUEST, msg.clone()),
            AppError::Ollama(msg) => (axum::http::StatusCode::SERVICE_UNAVAILABLE, msg.clone()),
            AppError::Material(msg) => (axum::http::StatusCode::BAD_GATEWAY, msg.clone()),
            AppError::Ai(msg) => (axum::http::StatusCode::BAD_GATEWAY, msg.clone()),
            AppError::Submission(msg) => (axum::http::StatusCode::BAD_REQUEST, msg.clone()),
            AppError::Evaluation(msg) => (axum::http::StatusCode::BAD_GATEWAY, msg.clone()),
            AppError::NotConfigured => (axum::http::StatusCode::NOT_FOUND, "Configuration not found".to_string()),
            AppError::Validation(msg) => (axum::http::StatusCode::BAD_REQUEST, msg.clone()),
            AppError::Internal(_) => (axum::http::StatusCode::INTERNAL_SERVER_ERROR, "Internal server error".to_string()),
        };

        let body = serde_json::json!({
            "error": message,
            "status": status.as_u16()
        });

        (status, axum::Json(body)).into_response()
    }
}

pub type Result<T> = std::result::Result<T, AppError>;