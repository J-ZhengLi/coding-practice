use anyhow::Result;
use async_trait::async_trait;
use notify_rust::Notification;

use super::models::NotificationTier;
use super::notifier::Notifier;

/// Desktop notification notifier using the system notification daemon (libnotify on Linux).
pub struct DesktopNotifier;

impl DesktopNotifier {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Notifier for DesktopNotifier {
    async fn send(&self, title: &str, body: &str) -> Result<()> {
        tracing::info!("DesktopNotifier: sending desktop notification: {} - {}", title, body);

        Notification::new()
            .summary(title)
            .body(body)
            .icon("dialog-information")
            .show()?;

        tracing::info!("DesktopNotifier: notification displayed successfully");
        Ok(())
    }

    fn tier(&self) -> NotificationTier {
        NotificationTier::Desktop
    }
}