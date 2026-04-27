use anyhow::Result;
use async_trait::async_trait;

use super::models::NotificationTier;

/// Trait for sending user notifications.
/// Implementations determine their tier and provide an async send method.
#[async_trait]
pub trait Notifier: Send + Sync {
    /// Send a notification with the given title and body.
    async fn send(&self, title: &str, body: &str) -> Result<()>;

    /// Return the notification tier this implementation handles.
    fn tier(&self) -> NotificationTier;
}

/// Desktop notification via the `notify` crate.
/// Stubbed for now; full implementation in plan 05-02.
pub struct DesktopNotifier;

#[async_trait]
impl Notifier for DesktopNotifier {
    async fn send(&self, title: &str, body: &str) -> Result<()> {
        tracing::info!("Desktop notification: {} - {}", title, body);
        // TODO: Full implementation with notify crate in 05-02
        Ok(())
    }

    fn tier(&self) -> NotificationTier {
        NotificationTier::Desktop
    }
}

/// SMTP email notification via the `lettre` crate.
/// Stubbed for now; full implementation in plan 05-02.
pub struct SmtpNotifier {
    pub host: String,
    pub port: u16,
    pub user: String,
    pub password: String,
    pub recipient: String,
}

#[async_trait]
impl Notifier for SmtpNotifier {
    async fn send(&self, title: &str, body: &str) -> Result<()> {
        tracing::info!(
            "SMTP notification to {} via {}:{}: {} - {}",
            self.recipient, self.host, self.port, title, body
        );
        // TODO: Full implementation with lettre crate in 05-02
        Ok(())
    }

    fn tier(&self) -> NotificationTier {
        NotificationTier::Smtp
    }
}

/// Gmail API notification.
/// Stubbed for now; full implementation in plan 05-02.
pub struct GmailNotifier {
    pub refresh_token: String,
    pub recipient: String,
}

#[async_trait]
impl Notifier for GmailNotifier {
    async fn send(&self, title: &str, body: &str) -> Result<()> {
        tracing::info!(
            "Gmail notification to {} using refresh token: {} - {}",
            self.recipient, title, body
        );
        // TODO: Full implementation with Gmail API in 05-02
        Ok(())
    }

    fn tier(&self) -> NotificationTier {
        NotificationTier::Gmail
    }
}