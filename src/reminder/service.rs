use std::sync::Arc;
use std::time::Duration;

use chrono::{Local, NaiveTime};
use tokio::time::sleep;

use crate::config::service::ConfigService;
use crate::db::repository::SqliteConfigRepository;
use crate::error::Result;

use super::models::{NotificationTier, ReminderConfig};
use super::notifier::Notifier;
use super::gmail::GmailNotifier;
use super::smtp::SmtpNotifier;
use super::desktop::DesktopNotifier;
use super::template::{random_encouraging_message, build_html_email, build_plain_email};

/// Default app URL used in email templates.
const DEFAULT_APP_URL: &str = "http://localhost:8001";

/// Background service that sends daily learning reminders.
///
/// The scheduler loop:
/// 1. Computes sleep duration until the next configured reminder time
/// 2. Sleeps until that time
/// 3. Checks deduplication (max 1 per day via `last_reminded_at`)
/// 4. Sends a reminder via the highest-priority available notification tier
/// 5. Updates `last_reminded_at` and repeats
pub struct ReminderService {
    config_service: Arc<ConfigService<SqliteConfigRepository>>,
}

impl ReminderService {
    pub fn new(config_service: Arc<ConfigService<SqliteConfigRepository>>) -> Self {
        Self { config_service }
    }

    /// Run the scheduler loop forever. Call via `tokio::spawn`.
    pub async fn run_scheduler(self: Arc<Self>) {
        loop {
            match self.compute_sleep_duration().await {
                Some(duration) => {
                    tracing::info!(
                        "Reminder scheduler: sleeping for {}s until next reminder time",
                        duration.as_secs()
                    );
                    sleep(duration).await;

                    // After sleeping, check if we should actually send
                    if self.should_send_today().await {
                        self.send_reminder().await;
                    } else {
                        tracing::info!("Reminder scheduler: already sent today, skipping");
                    }
                }
                None => {
                    // Reminders disabled or no time configured — sleep 5 minutes and recheck
                    sleep(Duration::from_secs(300)).await;
                }
            }
        }
    }

    /// Compute the duration to sleep until the next reminder time.
    ///
    /// Returns `None` if reminders are disabled or no reminder_time is set.
    /// If the configured time has already passed today, schedules for tomorrow.
    pub async fn compute_sleep_duration(&self) -> Option<Duration> {
        let config = self.load_reminder_config().await?;

        if !config.reminders_enabled {
            return None;
        }

        let reminder_time = Self::parse_reminder_time(&config.reminder_time)?;

        let now = Local::now();
        let today_target = now.date_naive().and_time(reminder_time)
            .and_local_timezone(now.timezone())
            .single()?;

        let target = if now < today_target {
            today_target
        } else {
            // Time already passed today; schedule for tomorrow
            today_target + chrono::Duration::days(1)
        };

        let duration = target - now;
        Some(duration.to_std().ok()?)
    }

    /// Check whether a reminder should be sent today.
    ///
    /// Returns `true` if `last_reminded_at` is not set or is from a previous day.
    /// Returns `false` if a reminder was already sent today.
    pub async fn should_send_today(&self) -> bool {
        match self.config_service.get_config().await {
            Ok(config) => {
                match &config.last_reminded_at {
                    Some(last) => {
                        // Parse the ISO 8601 date and compare with today
                        let last_date = last
                            .parse::<chrono::NaiveDateTime>()
                            .or_else(|_| {
                                // Try date-only format
                                last.parse::<chrono::NaiveDate>()
                                    .map(|d| d.and_hms_opt(0, 0, 0).unwrap())
                            });
                        match last_date {
                            Ok(dt) => dt.date() < Local::now().date_naive(),
                            Err(_) => {
                                // If we can't parse, allow sending (safer default)
                                tracing::warn!("Could not parse last_reminded_at: {}", last);
                                true
                            }
                        }
                    }
                    None => true,
                }
            }
            Err(_) => {
                // Not configured yet — don't send
                false
            }
        }
    }

    /// Build email content (subject + HTML body + plain text body).
    fn build_reminder_content() -> (String, String, String) {
        let message = random_encouraging_message();
        let html = build_html_email(message, DEFAULT_APP_URL);
        let plain = build_plain_email(message, DEFAULT_APP_URL);
        let subject = "Time for your daily coding practice!".to_string();
        (subject, html, plain)
    }

    /// Build test reminder content (used by the /api/reminders/test endpoint).
    fn build_reminder_content_test() -> (String, String, String) {
        let html = build_html_email(
            "This is a test notification from Coding Practice.",
            DEFAULT_APP_URL,
        );
        let plain = build_plain_email(
            "This is a test notification from Coding Practice.",
            DEFAULT_APP_URL,
        );
        let subject = "Test Reminder".to_string();
        (subject, html, plain)
    }

    /// Try sending via a specific notification tier.
    /// Returns Ok(()) on success, Err with the error on failure.
    async fn try_send_tier(config: &ReminderConfig, tier: NotificationTier, subject: &str, html_body: &str, plain_body: &str) -> anyhow::Result<()> {
        match tier {
            NotificationTier::Gmail => {
                let notifier = GmailNotifier::new(
                    config.gmail_refresh_token.clone().unwrap_or_default(),
                    config.gmail_client_id.clone().unwrap_or_default(),
                    config.gmail_client_secret.clone().unwrap_or_default(),
                    config.email.clone().unwrap_or_default(),
                );
                // Gmail API expects the full HTML body
                notifier.send(subject, html_body).await
            }
            NotificationTier::Smtp => {
                let notifier = SmtpNotifier::new(
                    config.smtp_host.clone().unwrap_or_default(),
                    config.smtp_port.unwrap_or(587),
                    config.smtp_user.clone().unwrap_or_default(),
                    config.smtp_password.clone().unwrap_or_default(),
                    config.email.clone().unwrap_or_default(),
                );
                // SMTP sends HTML email
                notifier.send(subject, html_body).await
            }
            NotificationTier::Desktop => {
                let notifier = DesktopNotifier::new();
                // Desktop notifications use plain text (short format)
                notifier.send(subject, plain_body).await
            }
            NotificationTier::None => {
                anyhow::bail!("No notification tier available");
            }
        }
    }

    /// Send a reminder notification via the highest-priority available tier,
    /// with fallback to lower-priority tiers on failure.
    pub async fn send_reminder(&self) {
        let config = match self.load_reminder_config().await {
            Some(c) => c,
            None => {
                tracing::debug!("ReminderService: no config, skipping send");
                return;
            }
        };

        let (subject, html_body, plain_body) = Self::build_reminder_content();
        let best_tier = config.best_tier();

        // Fallback chain: try each tier from highest to lowest
        let tiers = match best_tier {
            NotificationTier::Gmail => vec![
                NotificationTier::Gmail,
                NotificationTier::Smtp,
                NotificationTier::Desktop,
            ],
            NotificationTier::Smtp => vec![
                NotificationTier::Smtp,
                NotificationTier::Desktop,
            ],
            NotificationTier::Desktop => vec![
                NotificationTier::Desktop,
            ],
            NotificationTier::None => {
                tracing::warn!("No notification tier available for reminder");
                return;
            }
        };

        for tier in &tiers {
            match Self::try_send_tier(&config, *tier, &subject, &html_body, &plain_body).await {
                Ok(()) => {
                    tracing::info!("Reminder sent successfully via {} tier", tier);
                    // Update last_reminded_at to now
                    let now = chrono::Utc::now().to_rfc3339();
                    if let Err(e) = self.update_last_reminded_at(&now).await {
                        tracing::error!("Failed to update last_reminded_at: {}", e);
                    }
                    return;
                }
                Err(e) => {
                    tracing::error!("Failed to send reminder via {}: {}", tier, e);
                    // Continue to next tier in the fallback chain
                }
            }
        }

        // All tiers failed — log but do not crash the scheduler
        tracing::error!("All notification tiers failed. Scheduler will continue running.");
    }

    /// Load the ReminderConfig from the persisted UserConfig.
    async fn load_reminder_config(&self) -> Option<ReminderConfig> {
        let config = self.config_service.get_config().await.ok()?;

        let reminder_time = config.reminder_time.clone()?;
        let reminders_enabled = config.reminders_enabled.unwrap_or(false);

        Some(ReminderConfig {
            reminder_time,
            reminders_enabled,
            email: config.email.clone(),
            gmail_refresh_token: config.gmail_refresh_token.clone(),
            gmail_client_id: config.gmail_client_id.clone(),
            gmail_client_secret: config.gmail_client_secret.clone(),
            smtp_host: config.smtp_host.clone(),
            smtp_port: config.smtp_port,
            smtp_user: config.smtp_user.clone(),
            smtp_password: config.smtp_password.clone(),
        })
    }

    /// Parse "HH:MM" format into a NaiveTime.
    fn parse_reminder_time(time_str: &str) -> Option<NaiveTime> {
        NaiveTime::parse_from_str(time_str, "%H:%M").ok()
    }

    /// Update the `last_reminded_at` config key.
    async fn update_last_reminded_at(&self, timestamp: &str) -> Result<()> {
        // We need to load current config and re-save with updated field.
        // This uses the config_service's key-value store directly.
        let mut config = self.config_service.get_config().await?;
        config.last_reminded_at = Some(timestamp.to_string());
        self.config_service.save_config(&config).await
    }
}

/// Send a test reminder immediately (for the /api/reminders/test endpoint).
/// Uses the fallback chain just like the scheduler does.
pub async fn send_test_reminder(
    config_service: Arc<ConfigService<SqliteConfigRepository>>,
) -> Result<NotificationTier> {
    let config_service_inner = config_service.clone();
    let config = config_service.get_config().await?;

    let reminder_config = match (
        config.reminder_time.clone(),
        config.reminders_enabled.unwrap_or(false),
    ) {
        (Some(time), _) => ReminderConfig {
            reminder_time: time,
            reminders_enabled: true, // Force enabled for test
            email: config.email.clone(),
            gmail_refresh_token: config.gmail_refresh_token.clone(),
            gmail_client_id: config.gmail_client_id.clone(),
            gmail_client_secret: config.gmail_client_secret.clone(),
            smtp_host: config.smtp_host.clone(),
            smtp_port: config.smtp_port,
            smtp_user: config.smtp_user.clone(),
            smtp_password: config.smtp_password.clone(),
        },
        _ => {
            return Err(crate::error::AppError::Validation(
                "No reminder configuration found. Set reminder_time first.".to_string(),
            ));
        }
    };

    let (subject, html_body, plain_body) = ReminderService::build_reminder_content_test();
    let best_tier = reminder_config.best_tier();

    // Fallback chain: try each tier from highest to lowest
    let tiers = match best_tier {
        NotificationTier::Gmail => vec![
            NotificationTier::Gmail,
            NotificationTier::Smtp,
            NotificationTier::Desktop,
        ],
        NotificationTier::Smtp => vec![
            NotificationTier::Smtp,
            NotificationTier::Desktop,
        ],
        NotificationTier::Desktop => vec![
            NotificationTier::Desktop,
        ],
        NotificationTier::None => {
            return Err(crate::error::AppError::Validation(
                "No notification tier available. Configure Gmail, SMTP, or enable desktop notifications.".to_string(),
            ));
        }
    };

    let mut last_tier = NotificationTier::None;
    let mut last_error: Option<String> = None;

    for tier in &tiers {
        match ReminderService::try_send_tier(&reminder_config, *tier, &subject, &html_body, &plain_body).await {
            Ok(()) => {
                // Update last_reminded_at for test
                let now = chrono::Utc::now().to_rfc3339();
                let mut cfg = config_service_inner.get_config().await?;
                cfg.last_reminded_at = Some(now);
                config_service_inner.save_config(&cfg).await?;

                return Ok(*tier);
            }
            Err(e) => {
                tracing::error!("Test reminder failed via {}: {}", tier, e);
                last_error = Some(format!("{} (tried {})", e, tier));
                last_tier = *tier;
            }
        }
    }

    Err(crate::error::AppError::Validation(format!(
        "Failed to send test reminder via all tiers. Last error: {}",
        last_error.unwrap_or_else(|| format!("tier {:?} failed", last_tier))
    )))
}