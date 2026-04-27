use anyhow::Result;
use async_trait::async_trait;
use base64::Engine;

use super::models::NotificationTier;
use super::notifier::Notifier;

/// Gmail API notifier that sends emails via the Gmail REST API
/// using OAuth2 refresh tokens.
pub struct GmailNotifier {
    pub client: reqwest::Client,
    pub refresh_token: String,
    pub client_id: String,
    pub client_secret: String,
    pub recipient: String,
}

impl GmailNotifier {
    pub fn new(
        refresh_token: String,
        client_id: String,
        client_secret: String,
        recipient: String,
    ) -> Self {
        Self {
            client: reqwest::Client::new(),
            refresh_token,
            client_id,
            client_secret,
            recipient,
        }
    }

    /// Refresh the OAuth2 access token using the stored refresh token.
    /// Returns an error with "invalid_grant" in the message if the token has been revoked or expired.
    async fn refresh_access_token(&self) -> Result<String> {
        let params = [
            ("client_id", self.client_id.as_str()),
            ("client_secret", self.client_secret.as_str()),
            ("refresh_token", self.refresh_token.as_str()),
            ("grant_type", "refresh_token"),
        ];

        let resp = self
            .client
            .post("https://oauth2.googleapis.com/token")
            .form(&params)
            .send()
            .await?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();

            // Check for invalid_grant which means the refresh token is no longer valid
            if body.contains("invalid_grant") {
                anyhow::bail!("GMAIL_TOKEN_INVALID: Refresh token has been revoked or expired. Re-authorization required.");
            }

            anyhow::bail!("Gmail token refresh failed: HTTP {} - {}", status, body);
        }

        let token_resp: serde_json::Value = resp.json().await?;
        let access_token = token_resp["access_token"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("No access_token in refresh response"))?
            .to_string();

        Ok(access_token)
    }

    /// Send a raw MIME email message via the Gmail API.
    async fn send_raw(&self, access_token: &str, raw_message: &str) -> Result<()> {
        // Gmail API requires base64url encoding (RFC 4648)
        let encoded = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(raw_message.as_bytes());

        let body = serde_json::json!({
            "raw": encoded
        });

        let resp = self
            .client
            .post("https://gmail.googleapis.com/gmail/v1/users/me/messages/send")
            .bearer_auth(access_token)
            .json(&body)
            .send()
            .await?;

        if !resp.status().is_success() {
            let status = resp.status();
            let response_body = resp.text().await.unwrap_or_default();

            // 401 or 403 means the access token is invalid/expired
            if status.as_u16() == 401 || status.as_u16() == 403 {
                anyhow::bail!("GMAIL_TOKEN_INVALID: Access token rejected (HTTP {}). Re-authorization required.", status);
            }

            anyhow::bail!("Gmail send failed: HTTP {} - {}", status, response_body);
        }

        Ok(())
    }

    /// Build a simple MIME email message.
    fn build_mime_message(&self, subject: &str, html_body: &str) -> String {
        format!(
            "From: {}\r\n\
             To: {}\r\n\
             Subject: =?utf-8?B?{}?=\r\n\
             MIME-Version: 1.0\r\n\
             Content-Type: text/html; charset=utf-8\r\n\
             \r\n\
             {}",
            self.recipient,
            self.recipient,
            base64::engine::general_purpose::STANDARD.encode(subject.as_bytes()),
            html_body
        )
    }
}

#[async_trait]
impl Notifier for GmailNotifier {
    async fn send(&self, title: &str, body: &str) -> Result<()> {
        tracing::info!("GmailNotifier: sending to {}", self.recipient);

        let access_token = self.refresh_access_token().await?;
        let mime = self.build_mime_message(title, body);
        self.send_raw(&access_token, &mime).await?;

        tracing::info!("GmailNotifier: email sent successfully to {}", self.recipient);
        Ok(())
    }

    fn tier(&self) -> NotificationTier {
        NotificationTier::Gmail
    }
}