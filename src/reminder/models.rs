use serde::{Deserialize, Serialize};

/// Resolved reminder configuration used by ReminderService.
/// Derived from the persisted UserConfig optional fields.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReminderConfig {
    /// Time of day to send reminder, in HH:MM format (e.g. "09:00")
    pub reminder_time: String,
    /// Whether reminders are enabled
    pub reminders_enabled: bool,
    /// User email for notification delivery
    pub email: Option<String>,
    /// Gmail OAuth2 refresh token for Gmail API sending
    pub gmail_refresh_token: Option<String>,
    /// SMTP relay hostname
    pub smtp_host: Option<String>,
    /// SMTP relay port
    pub smtp_port: Option<u16>,
    /// SMTP authentication username
    pub smtp_user: Option<String>,
    /// SMTP authentication password
    pub smtp_password: Option<String>,
}

/// Notification tier priority levels.
/// Gmail API is preferred, then SMTP relay, then desktop notification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum NotificationTier {
    Gmail,
    Smtp,
    Desktop,
    None,
}

impl std::fmt::Display for NotificationTier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NotificationTier::Gmail => write!(f, "gmail"),
            NotificationTier::Smtp => write!(f, "smtp"),
            NotificationTier::Desktop => write!(f, "desktop"),
            NotificationTier::None => write!(f, "none"),
        }
    }
}

impl ReminderConfig {
    /// Determine the highest-priority notification tier available
    /// based on the current configuration.
    pub fn best_tier(&self) -> NotificationTier {
        if self.gmail_refresh_token.is_some() {
            NotificationTier::Gmail
        } else if self.smtp_host.is_some() && self.smtp_user.is_some() {
            NotificationTier::Smtp
        } else {
            NotificationTier::Desktop
        }
    }
}

/// Response payload for GET /api/reminders/status
#[derive(Debug, Serialize)]
pub struct ReminderStatusResponse {
    pub tier: NotificationTier,
    pub enabled: bool,
    pub reminder_time: Option<String>,
}