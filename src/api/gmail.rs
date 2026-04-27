use axum::{
    extract::State,
    response::IntoResponse,
    Json,
    http::StatusCode,
};
use serde::{Deserialize, Serialize};

use crate::error::{AppError, Result};

use super::AppState;

/// Response from Google's device code endpoint.
#[derive(Debug, Serialize)]
pub struct DeviceCodeResponse {
    pub device_code: String,
    pub user_code: String,
    pub verification_url: String,
    pub expires_in: u64,
    pub interval: u64,
}

/// Request body for the token polling endpoint.
#[derive(Debug, Deserialize)]
pub struct TokenPollRequest {
    pub device_code: String,
}

/// Successful token response (access token returned to frontend, refresh stored server-side).
#[derive(Debug, Serialize)]
pub struct TokenSuccessResponse {
    pub status: String,
}

/// POST /api/gmail/connect - Initiate Gmail OAuth2 device code flow.
///
/// Reads `gmail_client_id` and `gmail_client_secret` from config,
/// requests a device code from Google, and returns it to the frontend
/// so the user can visit the verification URL and enter the user code.
pub async fn connect_gmail_handler(
    State((config_service, _, _, _, _, _, _, _)): State<AppState>,
) -> Result<impl IntoResponse> {
    let config = config_service.get_config().await?;

    let client_id = config.gmail_client_id.ok_or_else(|| {
        AppError::Validation("Gmail client ID not configured. Set gmail_client_id in settings first.".to_string())
    })?;
    let client_secret = config.gmail_client_secret.ok_or_else(|| {
        AppError::Validation("Gmail client secret not configured. Set gmail_client_secret in settings first.".to_string())
    })?;

    // Request device code from Google OAuth2
    let client = reqwest::Client::new();
    let params = [
        ("client_id", client_id.as_str()),
        ("scope", "https://www.googleapis.com/auth/gmail.send"),
    ];

    let resp = client
        .post("https://oauth2.googleapis.com/device/code")
        .form(&params)
        .send()
        .await
        .map_err(|e| AppError::Validation(format!("Failed to reach Google OAuth: {}", e)))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        tracing::error!("Google device code error: {} - {}", status, body);
        return Err(AppError::Validation(format!(
            "Google OAuth device code request failed: HTTP {}",
            status
        )));
    }

    let google_resp: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| AppError::Validation(format!("Invalid Google device code response: {}", e)))?;

    let device_code = google_resp["device_code"]
        .as_str()
        .ok_or_else(|| AppError::Validation("Missing device_code in Google response".to_string()))?
        .to_string();
    let user_code = google_resp["user_code"]
        .as_str()
        .ok_or_else(|| AppError::Validation("Missing user_code in Google response".to_string()))?
        .to_string();
    let verification_url = google_resp["verification_url"]
        .as_str()
        .unwrap_or("https://www.google.com/device")
        .to_string();
    let expires_in = google_resp["expires_in"]
        .as_u64()
        .unwrap_or(900);
    let interval = google_resp["interval"]
        .as_u64()
        .unwrap_or(5);

    // Store the client_id, client_secret, and device_code for the token polling step.
    // We store device_code temporarily in config so the token endpoint can use it.
    // In practice, the frontend sends it back, but we also keep client credentials server-side.
    let response = DeviceCodeResponse {
        device_code,
        user_code,
        verification_url,
        expires_in,
        interval,
    };

    Ok((StatusCode::OK, Json(response)))
}

/// POST /api/gmail/token - Poll for Gmail OAuth2 token after user authorizes.
///
/// Frontend polls this endpoint with the device_code. We forward the request
/// to Google's token endpoint. On success, we store the refresh_token in config
/// and return success. The access_token is ephemeral and not stored.
pub async fn poll_gmail_token_handler(
    State((config_service, _, _, _, _, _, _, _)): State<AppState>,
    Json(req): Json<TokenPollRequest>,
) -> Result<impl IntoResponse> {
    let config = config_service.get_config().await?;

    let client_id = config.gmail_client_id.ok_or_else(|| {
        AppError::Validation("Gmail client ID not configured".to_string())
    })?;
    let client_secret = config.gmail_client_secret.ok_or_else(|| {
        AppError::Validation("Gmail client secret not configured".to_string())
    })?;

    let client = reqwest::Client::new();
    let params = [
        ("client_id", client_id.as_str()),
        ("client_secret", client_secret.as_str()),
        ("device_code", req.device_code.as_str()),
        ("grant_type", "urn:ietf:params:oauth:grant-type:device_code"),
    ];

    let resp = client
        .post("https://oauth2.googleapis.com/token")
        .form(&params)
        .send()
        .await
        .map_err(|e| AppError::Validation(format!("Failed to reach Google token endpoint: {}", e)))?;

    let token_resp: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| AppError::Validation(format!("Invalid Google token response: {}", e)))?;

    // Check for authorization_pending or slow_down errors
    if let Some(error) = token_resp["error"].as_str() {
        match error {
            "authorization_pending" => {
                return Ok((
                    StatusCode::ACCEPTED,
                    Json(serde_json::json!({
                        "status": "pending",
                        "message": "User has not yet authorized the request"
                    })),
                ));
            }
            "slow_down" => {
                return Ok((
                    StatusCode::ACCEPTED,
                    Json(serde_json::json!({
                        "status": "slow_down",
                        "message": "Polling too fast, increase interval"
                    })),
                ));
            }
            "expired_token" => {
                return Err(AppError::Validation(
                    "Device code expired. Please start the connection flow again.".to_string(),
                ));
            }
            "access_denied" => {
                return Err(AppError::Validation(
                    "User denied access to Gmail.".to_string(),
                ));
            }
            other => {
                return Err(AppError::Validation(format!(
                    "Google OAuth error: {}",
                    other
                )));
            }
        }
    }

    // Success: extract refresh_token and store it
    let refresh_token = token_resp["refresh_token"]
        .as_str()
        .ok_or_else(|| AppError::Validation("No refresh_token in Google response. User may need to re-authorize.".to_string()))?;

    // Store refresh_token in config
    let mut updated_config = config_service.get_config().await?;
    updated_config.gmail_refresh_token = Some(refresh_token.to_string());
    config_service.save_config(&updated_config).await?;

    tracing::info!("Gmail OAuth2 refresh token stored successfully");

    Ok((
        StatusCode::OK,
        Json(serde_json::json!({
            "status": "success"
        })),
    ))
}

/// POST /api/gmail/disconnect - Disconnect Gmail by clearing the stored refresh_token.
///
/// This is called when a Gmail token is invalid/expired and the user
/// needs to re-authorize, or when the user wants to disconnect Gmail.
pub async fn disconnect_gmail_handler(
    State((config_service, _, _, _, _, _, _, _)): State<AppState>,
) -> Result<impl IntoResponse> {
    let mut config = config_service.get_config().await?;

    if config.gmail_refresh_token.is_none() {
        return Ok((
            StatusCode::OK,
            Json(serde_json::json!({
                "status": "already_disconnected",
                "message": "Gmail is not connected"
            })),
        ));
    }

    config.gmail_refresh_token = None;
    config_service.save_config(&config).await?;

    tracing::info!("Gmail disconnected: refresh_token cleared");

    Ok((
        StatusCode::OK,
        Json(serde_json::json!({
            "status": "disconnected"
        })),
    ))
}