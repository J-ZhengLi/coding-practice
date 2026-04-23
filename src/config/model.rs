use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserConfig {
    pub preferred_language: String,
    pub skill_level: String,
    pub daily_quotas: Vec<LanguageQuota>,
    pub ai_model: String,
    pub ai_model_type: ModelType,
    #[serde(default)]
    pub email: Option<String>,
    #[serde(default)]
    pub gmail_client_id: Option<String>,
    #[serde(default)]
    pub gmail_client_secret: Option<String>,
    #[serde(default)]
    pub smtp_host: Option<String>,
    #[serde(default)]
    pub smtp_port: Option<u16>,
    #[serde(default)]
    pub smtp_user: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguageQuota {
    pub language: String,
    pub quota: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ModelType {
    Local,
    Api,
}

#[derive(Debug, Deserialize)]
pub struct ConfigRequest {
    pub preferred_language: String,
    pub skill_level: String,
    pub daily_quotas: Vec<LanguageQuota>,
    pub ai_model: String,
    pub ai_model_type: ModelType,
    #[serde(default)]
    pub email: Option<String>,
    #[serde(default)]
    pub gmail_client_id: Option<String>,
    #[serde(default)]
    pub gmail_client_secret: Option<String>,
    #[serde(default)]
    pub smtp_host: Option<String>,
    #[serde(default)]
    pub smtp_port: Option<u16>,
    #[serde(default)]
    pub smtp_user: Option<String>,
}

impl From<ConfigRequest> for UserConfig {
    fn from(req: ConfigRequest) -> Self {
        Self {
            preferred_language: req.preferred_language,
            skill_level: req.skill_level,
            daily_quotas: req.daily_quotas,
            ai_model: req.ai_model,
            ai_model_type: req.ai_model_type,
            email: req.email,
            gmail_client_id: req.gmail_client_id,
            gmail_client_secret: req.gmail_client_secret,
            smtp_host: req.smtp_host,
            smtp_port: req.smtp_port,
            smtp_user: req.smtp_user,
        }
    }
}