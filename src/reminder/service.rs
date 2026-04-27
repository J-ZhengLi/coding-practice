use std::sync::Arc;
use std::time::Duration;

use chrono::{Local, NaiveTime};
use tokio::time::sleep;

use crate::config::service::ConfigService;
use crate::db::repository::SqliteConfigRepository;
use crate::error::Result;

use super::models::{NotificationTier, ReminderConfig};
use super::notifier::{DesktopNotifier, GmailNotifier, Notifier, SmtpNotifier};

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

    /// Send a reminder notification via the highest-priority available tier.
    pub async fn send_reminder(&self) {
        let config = match self.load_reminder_config().await {
            Some(c) => c,
            None => {
                tracing::debug!("ReminderService: no config, skipping send");
                return;
            }
        };

        let tier = config.best_tier();
        let title = "Coding Practice Reminder";
        let body = "Time for your daily coding practice! Keep the streak going.";

        let result = match tier {
            NotificationTier::Gmail => {
                let notifier = GmailNotifier {
                    refresh_token: config.gmail_refresh_token.unwrap_or_default(),
                    recipient: config.email.unwrap_or_default(),
                };
                notifier.send(title, body).await
            }
            NotificationTier::Smtp => {
                let notifier = SmtpNotifier {
                    host: config.smtp_host.unwrap_or_default(),
                    port: config.smtp_port.unwrap_or(587),
                    user: config.smtp_user.unwrap_or_default(),
                    password: config.smtp_password.unwrap_or_default(),
                    recipient: config.email.unwrap_or_default(),
                };
                notifier.send(title, body).await
            }
            NotificationTier::Desktop => {
                let notifier = DesktopNotifier;
                notifier.send(title, body).await
            }
            NotificationTier::None => {
                tracing::warn!("No notification tier available for reminder");
                return;
            }
        };

        match result {
            Ok(()) => {
                // Update last_reminded_at to now
                let now = chrono::Utc::now().to_rfc3339();
                if let Err(e) = self.update_last_reminded_at(&now).await {
                    tracing::error!("Failed to update last_reminded_at: {}", e);
                }
            }
            Err(e) => {
                tracing::error!("Failed to send reminder via {}: {}", tier, e);
            }
        }
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

    let tier = reminder_config.best_tier();
    let title = "Test Reminder";
    let body = "This is a test notification from Coding Practice.";

    let result = match tier {
        NotificationTier::Gmail => {
            let notifier = GmailNotifier {
                refresh_token: reminder_config.gmail_refresh_token.unwrap_or_default(),
                recipient: reminder_config.email.unwrap_or_default(),
            };
            notifier.send(title, body).await
        }
        NotificationTier::Smtp => {
            let notifier = SmtpNotifier {
                host: reminder_config.smtp_host.unwrap_or_default(),
                port: reminder_config.smtp_port.unwrap_or(587),
                user: reminder_config.smtp_user.unwrap_or_default(),
                password: reminder_config.smtp_password.unwrap_or_default(),
                recipient: reminder_config.email.unwrap_or_default(),
            };
            notifier.send(title, body).await
        }
        NotificationTier::Desktop => {
            let notifier = DesktopNotifier;
            notifier.send(title, body).await
        }
        NotificationTier::None => {
            return Err(crate::error::AppError::Validation(
                "No notification tier available. Configure Gmail, SMTP, or enable desktop notifications.".to_string(),
            ));
        }
    };

    result.map_err(|e| crate::error::AppError::Validation(format!("Failed to send test: {}", e)))?;

    // Update last_reminded_at for test
    let now = chrono::Utc::now().to_rfc3339();
    let mut cfg = config_service_inner.get_config().await?;
    cfg.last_reminded_at = Some(now);
    config_service_inner.save_config(&cfg).await?;

    Ok(tier)
}