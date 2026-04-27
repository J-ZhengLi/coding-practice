use anyhow::Result;
use async_trait::async_trait;
use lettre::message::header::ContentType;
use lettre::message::Mailbox;
use lettre::transport::smtp::authentication::Credentials;
use lettre::AsyncSmtpTransport;
use lettre::Tokio1Executor;

use super::models::NotificationTier;
use super::notifier::Notifier;

/// SMTP email notifier that sends notifications via an SMTP relay.
pub struct SmtpNotifier {
    pub host: String,
    pub port: u16,
    pub user: String,
    pub password: String,
    pub recipient: String,
}

impl SmtpNotifier {
    pub fn new(
        host: String,
        port: u16,
        user: String,
        password: String,
        recipient: String,
    ) -> Self {
        Self {
            host,
            port,
            user,
            password,
            recipient,
        }
    }
}

#[async_trait]
impl Notifier for SmtpNotifier {
    async fn send(&self, title: &str, body: &str) -> Result<()> {
        tracing::info!("SmtpNotifier: sending to {} via {}:{}",
            self.recipient, self.host, self.port);

        let from_mailbox: Mailbox = self.user.parse()
            .map_err(|e| anyhow::anyhow!("Invalid SMTP sender address '{}': {}", self.user, e))?;
        let to_mailbox: Mailbox = self.recipient.parse()
            .map_err(|e| anyhow::anyhow!("Invalid SMTP recipient address '{}': {}", self.recipient, e))?;

        let email = lettre::Message::builder()
            .from(from_mailbox)
            .to(to_mailbox)
            .subject(title)
            .header(ContentType::TEXT_HTML)
            .body(body.to_string())
            .map_err(|e| anyhow::anyhow!("Failed to build email: {}", e))?;

        let creds = Credentials::new(self.user.clone(), self.password.clone());

        let mailer: AsyncSmtpTransport<Tokio1Executor> = if self.port == 465 {
            // Implicit TLS (port 465)
            AsyncSmtpTransport::<Tokio1Executor>::relay(&self.host)?
                .port(self.port)
                .credentials(creds)
                .build()
        } else {
            // STARTTLS (port 587 and others)
            AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&self.host)?
                .port(self.port)
                .credentials(creds)
                .build()
        };

        lettre::AsyncTransport::send(&mailer, email).await?;

        tracing::info!("SmtpNotifier: email sent successfully to {}", self.recipient);
        Ok(())
    }

    fn tier(&self) -> NotificationTier {
        NotificationTier::Smtp
    }
}