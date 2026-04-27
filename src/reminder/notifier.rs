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