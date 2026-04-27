pub mod models;
pub mod notifier;
pub mod gmail;
pub mod smtp;
pub mod desktop;
pub mod service;

pub use models::{NotificationTier, ReminderConfig, ReminderStatusResponse};
pub use notifier::Notifier;
pub use gmail::GmailNotifier;
pub use smtp::SmtpNotifier;
pub use desktop::DesktopNotifier;
pub use service::{ReminderService, send_test_reminder};