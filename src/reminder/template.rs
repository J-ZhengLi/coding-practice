use chrono::Datelike;

/// Encouraging email templates for daily coding practice reminders.

/// Random encouraging messages to include in reminder emails.
const ENCOURAGING_MESSAGES: [&str; 5] = [
    "Every line of code makes you a better developer. Let's practice!",
    "Consistency is the key to mastery. Time for today's session!",
    "Your future self will thank you for practicing today.",
    "Small steps, big progress. Ready to code?",
    "Learning never stops — and neither do you. Let's go!",
];

/// Get a random encouraging message.
/// Uses the current day of year as a simple deterministic selection
/// so the same day always shows the same message.
pub fn random_encouraging_message() -> &'static str {
    let day_of_year = chrono::Local::now().ordinal0() as usize;
    ENCOURAGING_MESSAGES[day_of_year % ENCOURAGING_MESSAGES.len()]
}

/// Build an HTML email body with an encouraging message and a link to the app.
pub fn build_html_email(message: &str, app_url: &str) -> String {
    format!(
        r#"<!DOCTYPE html>
<html>
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
</head>
<body style="margin: 0; padding: 0; background-color: #f5f5f5; font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;">
<table width="100%" cellpadding="0" cellspacing="0" style="background-color: #f5f5f5; padding: 20px 0;">
  <tr>
    <td align="center">
      <table width="600" cellpadding="0" cellspacing="0" style="background-color: #ffffff; border-radius: 8px; overflow: hidden; box-shadow: 0 2px 8px rgba(0,0,0,0.1);">
        <tr>
          <td style="background: linear-gradient(135deg, #667eea 0%, #764ba2 100%); padding: 30px 40px; text-align: center;">
            <h1 style="margin: 0; color: #ffffff; font-size: 24px; font-weight: 600;">Coding Practice</h1>
          </td>
        </tr>
        <tr>
          <td style="padding: 30px 40px;">
            <p style="margin: 0 0 16px 0; font-size: 18px; color: #333333; line-height: 1.6;">
              {message}
            </p>
            <p style="margin: 0 0 24px 0; font-size: 14px; color: #666666; line-height: 1.5;">
              Keep your streak going — even a few minutes of practice each day builds lasting skills.
            </p>
            <table width="100%" cellpadding="0" cellspacing="0">
              <tr>
                <td align="center">
                  <a href="{app_url}" style="display: inline-block; background: linear-gradient(135deg, #667eea 0%, #764ba2 100%); color: #ffffff; text-decoration: none; padding: 14px 32px; border-radius: 6px; font-size: 16px; font-weight: 600;">
                    Start Practicing
                  </a>
                </td>
              </tr>
            </table>
          </td>
        </tr>
        <tr>
          <td style="padding: 20px 40px; background-color: #f9f9f9; border-top: 1px solid #eeeeee;">
            <p style="margin: 0; font-size: 12px; color: #999999; text-align: center;">
              You're receiving this because you enabled daily reminders in Coding Practice.
            </p>
          </td>
        </tr>
      </table>
    </td>
  </tr>
</table>
</body>
</html>"#,
        message = message,
        app_url = app_url,
    )
}

/// Build a plain-text email body as a fallback for clients that don't support HTML.
pub fn build_plain_email(message: &str, app_url: &str) -> String {
    format!(
        "Coding Practice Reminder\n\n{}\n\nKeep your streak going — even a few minutes of practice each day builds lasting skills.\n\nStart Practicing: {}\n\nYou're receiving this because you enabled daily reminders in Coding Practice.",
        message, app_url
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_random_message_is_valid() {
        let msg = random_encouraging_message();
        assert!(!msg.is_empty());
        assert!(ENCOURAGING_MESSAGES.contains(&msg));
    }

    #[test]
    fn test_build_html_email_contains_message() {
        let html = build_html_email("Test message", "http://localhost:8001");
        assert!(html.contains("Test message"));
        assert!(html.contains("http://localhost:8001"));
        assert!(html.contains("<html>"));
        assert!(html.contains("Start Practicing"));
    }

    #[test]
    fn test_build_plain_email_contains_message() {
        let plain = build_plain_email("Test message", "http://localhost:8001");
        assert!(plain.contains("Test message"));
        assert!(plain.contains("http://localhost:8001"));
        assert!(plain.contains("Start Practicing"));
    }
}