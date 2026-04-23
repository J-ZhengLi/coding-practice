use crate::db::repository::ConfigRepository;
use crate::config::model::{UserConfig, LanguageQuota};
use crate::error::{AppError, Result};

pub struct ConfigService<R: ConfigRepository> {
    repository: R,
}

impl<R: ConfigRepository> ConfigService<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    pub async fn is_configured(&self) -> Result<bool> {
        self.repository.exists().await
            .map_err(AppError::from)
    }

    pub async fn get_config(&self) -> Result<UserConfig> {
        if !self.is_configured().await? {
            return Err(AppError::NotConfigured);
        }

        let configs = self.repository.get_all().await
            .map_err(AppError::from)?;

        let get_value = |key: &str| -> Result<String> {
            configs
                .iter()
                .find(|c| c.key == key)
                .map(|c| c.value.clone())
                .ok_or_else(|| AppError::Config(format!("Config key '{}' not found", key)))
        };

        let preferred_language = get_value("preferred_language")?;
        let skill_level = get_value("skill_level")?;
        let ai_model = get_value("ai_model")?;
        let ai_model_type = get_value("ai_model_type")?;

        let daily_quotas = vec![
            LanguageQuota {
                language: "python".to_string(),
                quota: get_value("quota_python")?.parse()
                    .map_err(|e| AppError::Validation(format!("Invalid quota for python: {}", e)))?,
            },
            LanguageQuota {
                language: "rust".to_string(),
                quota: get_value("quota_rust")?.parse()
                    .map_err(|e| AppError::Validation(format!("Invalid quota for rust: {}", e)))?,
            },
            LanguageQuota {
                language: "go".to_string(),
                quota: get_value("quota_go")?.parse()
                    .map_err(|e| AppError::Validation(format!("Invalid quota for go: {}", e)))?,
            },
            LanguageQuota {
                language: "cpp".to_string(),
                quota: get_value("quota_cpp")?.parse()
                    .map_err(|e| AppError::Validation(format!("Invalid quota for cpp: {}", e)))?,
            },
        ];

        let get_optional = |key: &str| -> Option<String> {
            configs
                .iter()
                .find(|c| c.key == key)
                .map(|c| c.value.clone())
        };

        Ok(UserConfig {
            preferred_language,
            skill_level,
            daily_quotas,
            ai_model,
            ai_model_type: serde_json::from_str(&ai_model_type)
                .map_err(|e| AppError::Validation(format!("Invalid ai_model_type: {}", e)))?,
            email: get_optional("email"),
            gmail_client_id: get_optional("gmail_client_id"),
            gmail_client_secret: get_optional("gmail_client_secret"),
            smtp_host: get_optional("smtp_host"),
            smtp_port: get_optional("smtp_port").and_then(|v| v.parse().ok()),
            smtp_user: get_optional("smtp_user"),
        })
    }

    pub async fn save_config(&self, config: &UserConfig) -> Result<()> {
        // Validate all required fields (no defaults per D-06)
        if config.preferred_language.is_empty() {
            return Err(AppError::Validation("Preferred language is required".to_string()));
        }
        if config.skill_level.is_empty() {
            return Err(AppError::Validation("Skill level is required".to_string()));
        }
        if config.ai_model.is_empty() {
            return Err(AppError::Validation("AI model is required".to_string()));
        }

        // Validate language selection (CONF-01)
        let valid_languages = ["python", "rust", "go", "cpp"];
        if !valid_languages.contains(&config.preferred_language.as_str()) {
            return Err(AppError::Validation("Invalid preferred language. Must be one of: python, rust, go, cpp".to_string()));
        }

        // Validate skill level (CONF-02)
        let valid_levels = ["beginner", "intermediate", "advanced"];
        if !valid_levels.contains(&config.skill_level.as_str()) {
            return Err(AppError::Validation("Invalid skill level. Must be one of: beginner, intermediate, advanced".to_string()));
        }

        // Validate daily quotas (CONF-03) - must be > 0
        for quota in &config.daily_quotas {
            if quota.quota == 0 {
                return Err(AppError::Validation(format!("Daily quota for {} must be greater than 0", quota.language)));
            }
        }

        // Save all config values
        self.repository.set("preferred_language", &config.preferred_language).await
            .map_err(AppError::from)?;
        self.repository.set("skill_level", &config.skill_level).await
            .map_err(AppError::from)?;
        self.repository.set("ai_model", &config.ai_model).await
            .map_err(AppError::from)?;
        self.repository.set(
            "ai_model_type",
            &serde_json::to_string(&config.ai_model_type)
                .map_err(|e| AppError::Validation(format!("Failed to serialize ai_model_type: {}", e)))?,
        ).await
            .map_err(AppError::from)?;

        for quota in &config.daily_quotas {
            self.repository
                .set(&format!("quota_{}", quota.language), &quota.quota.to_string())
                .await
                .map_err(AppError::from)?;
        }

        // Save optional email/notification fields (CONF-07/CONF-08)
        if let Some(ref email) = config.email {
            self.repository.set("email", email).await.map_err(AppError::from)?;
        }
        if let Some(ref cid) = config.gmail_client_id {
            self.repository.set("gmail_client_id", cid).await.map_err(AppError::from)?;
        }
        if let Some(ref cs) = config.gmail_client_secret {
            self.repository.set("gmail_client_secret", cs).await.map_err(AppError::from)?;
        }
        if let Some(ref host) = config.smtp_host {
            self.repository.set("smtp_host", host).await.map_err(AppError::from)?;
        }
        if let Some(ref port) = config.smtp_port {
            self.repository.set("smtp_port", &port.to_string()).await.map_err(AppError::from)?;
        }
        if let Some(ref user) = config.smtp_user {
            self.repository.set("smtp_user", user).await.map_err(AppError::from)?;
        }

        Ok(())
    }
}