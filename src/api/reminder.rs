use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    Json,
};

use crate::error::Result;
use crate::reminder::models::ReminderStatusResponse;

use super::AppState;

/// POST /api/reminders/test - Send a test reminder notification.
/// Triggers an immediate notification via the highest-priority available tier.
pub async fn test_reminder_handler(
    State((config_service, _, _, _, _, _, _)): State<AppState>,
) -> Result<impl IntoResponse> {
    let tier = crate::reminder::send_test_reminder(config_service).await?;

    Ok((
        StatusCode::OK,
        Json(serde_json::json!({
            "status": "sent",
            "tier": tier.to_string(),
        })),
    ))
}

/// GET /api/reminders/status - Get current reminder configuration status.
/// Returns the active notification tier and whether reminders are enabled.
pub async fn get_reminder_status_handler(
    State((config_service, _, _, _, _, _, _)): State<AppState>,
) -> Result<impl IntoResponse> {
    let config = config_service.get_config().await?;

    let enabled = config.reminders_enabled.unwrap_or(false);
    let reminder_time = config.reminder_time.clone();

    // Determine tier from available config
    let tier = if config.gmail_refresh_token.is_some() {
        crate::reminder::models::NotificationTier::Gmail
    } else if config.smtp_host.is_some() && config.smtp_user.is_some() {
        crate::reminder::models::NotificationTier::Smtp
    } else {
        crate::reminder::models::NotificationTier::Desktop
    };

    let response = ReminderStatusResponse {
        tier,
        enabled,
        reminder_time,
    };

    Ok((StatusCode::OK, Json(response)))
}