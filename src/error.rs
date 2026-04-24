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
        let (status, message) = match &self {
            AppError::Database(e) => (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
            AppError::Config(msg) => (axum::http::StatusCode::BAD_REQUEST, msg.clone()),
            AppError::Ollama(msg) => (axum::http::StatusCode::SERVICE_UNAVAILABLE, msg.clone()),
            AppError::Material(msg) => (axum::http::StatusCode::BAD_GATEWAY, msg.clone()),
            AppError::Ai(msg) => (axum::http::StatusCode::BAD_GATEWAY, msg.clone()),
            AppError::Submission(msg) => (axum::http::StatusCode::BAD_REQUEST, msg.clone()),
            AppError::Evaluation(msg) => (axum::http::StatusCode::BAD_GATEWAY, msg.clone()),
            AppError::NotConfigured => (axum::http::StatusCode::NOT_FOUND, "Configuration not found".to_string()),
            AppError::Validation(msg) => (axum::http::StatusCode::BAD_REQUEST, msg.clone()),
            AppError::Internal(e) => (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
        };

        let body = serde_json::json!({
            "error": message,
            "status": status.as_u16()
        });

        (status, axum::Json(body)).into_response()
    }
}

pub type Result<T> = std::result::Result<T, AppError>;