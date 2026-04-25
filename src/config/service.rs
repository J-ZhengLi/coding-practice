use crate::db::repository::ConfigRepository;
use crate::config::model::{UserConfig, LanguageQuota, LanguageSkillLevel};
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
        let ai_model = get_value("ai_model")?;
        let ai_model_type = get_value("ai_model_type")?;

        // Load per-language skill levels from skill_level_{language} keys
        // Backward compat: if only legacy "skill_level" key exists, use it for all languages
        let languages = ["python", "rust", "go", "cpp"];
        let skill_levels = if let Ok(global_level) = get_value("skill_level") {
            // Legacy: single global skill level
            languages.iter().map(|lang| LanguageSkillLevel {
                language: lang.to_string(),
                skill_level: global_level.clone(),
            }).collect()
        } else {
            // New: per-language skill levels — only include languages that have stored values
            languages.iter().filter_map(|lang| {
                get_value(&format!("skill_level_{}", lang)).ok().map(|level| {
                    LanguageSkillLevel {
                        language: lang.to_string(),
                        skill_level: level,
                    }
                })
            }).collect()
        };

        // Load per-language daily quotas dynamically from quota_{language} keys.
        // Only include languages that have stored quota values, so we don't fail
        // when the user didn't select all four languages during configuration.
        let get_optional_value = |key: &str| -> Option<String> {
            configs
                .iter()
                .find(|c| c.key == key)
                .map(|c| c.value.clone())
        };

        let daily_quotas: Vec<LanguageQuota> = languages.iter()
            .filter_map(|lang| {
                get_optional_value(&format!("quota_{}", lang)).map(|quota_str| {
                    let quota: u32 = quota_str.parse().ok()?;
                    Some(LanguageQuota {
                        language: lang.to_string(),
                        quota,
                    })
                })
            })
            .flatten()
            .collect();

        // If no per-language quotas found, fall back to legacy hard-coded approach
        // for backward compatibility with existing databases
        let daily_quotas = if daily_quotas.is_empty() {
            vec![
                LanguageQuota {
                    language: "python".to_string(),
                    quota: get_value("quota_python")?.parse()
                        .map_err(|e| AppError::Validation(format!("Invalid quota for python: {}", e)))?,
                },
            ]
        } else {
            daily_quotas
        };

        let get_optional = |key: &str| -> Option<String> {
            configs
                .iter()
                .find(|c| c.key == key)
                .map(|c| c.value.clone())
        };

        Ok(UserConfig {
            preferred_language,
            skill_levels,
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
        if config.preferred_language.is_empty() {
            return Err(AppError::Validation("Preferred language is required".to_string()));
        }
        if config.ai_model.is_empty() {
            return Err(AppError::Validation("AI model is required".to_string()));
        }

        let valid_languages = ["python", "rust", "go", "cpp"];
        if !valid_languages.contains(&config.preferred_language.as_str()) {
            return Err(AppError::Validation("Invalid preferred language. Must be one of: python, rust, go, cpp".to_string()));
        }

        let valid_levels = ["beginner", "intermediate", "advanced"];
        for sl in &config.skill_levels {
            if !valid_languages.contains(&sl.language.as_str()) {
                return Err(AppError::Validation(format!("Invalid language: {}", sl.language)));
            }
            if !valid_levels.contains(&sl.skill_level.as_str()) {
                return Err(AppError::Validation(format!("Invalid skill level for {}. Must be: beginner, intermediate, advanced", sl.language)));
            }
        }

        for quota in &config.daily_quotas {
            if quota.quota == 0 {
                return Err(AppError::Validation(format!("Daily quota for {} must be greater than 0", quota.language)));
            }
        }

        self.repository.set("preferred_language", &config.preferred_language).await
            .map_err(AppError::from)?;
        self.repository.set("ai_model", &config.ai_model).await
            .map_err(AppError::from)?;
        self.repository.set(
            "ai_model_type",
            &serde_json::to_string(&config.ai_model_type)
                .map_err(|e| AppError::Validation(format!("Failed to serialize ai_model_type: {}", e)))?,
        ).await
            .map_err(AppError::from)?;

        for sl in &config.skill_levels {
            self.repository
                .set(&format!("skill_level_{}", sl.language), &sl.skill_level)
                .await
                .map_err(AppError::from)?;
        }

        for quota in &config.daily_quotas {
            self.repository
                .set(&format!("quota_{}", quota.language), &quota.quota.to_string())
                .await
                .map_err(AppError::from)?;
        }

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