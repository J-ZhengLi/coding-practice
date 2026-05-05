use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use tracing::{info, warn};

use serde_json;

use crate::ai::provider::AiProvider;
use crate::config::model::{ModelType, UserConfig};
use crate::config::service::ConfigService;
use crate::db::exercise_repo::ExerciseRepository;
use crate::db::models::{Exercise, NewExercise};
use crate::db::repository::SqliteConfigRepository;
use crate::error::{AppError, Result};
use crate::exercise::generator::ExerciseGenerator;
use crate::exercise::models::{validate_difficulty, validate_language};
use crate::material::fetcher::MaterialService;

/// Service orchestrating exercise generation, caching, and retrieval.
///
/// Implements D-09: batch generation on first use with SQLite caching.
/// Subsequent requests for the same language/difficulty return cached exercises.
pub struct ExerciseService {
    generator: ExerciseGenerator,
    exercise_repo: Arc<dyn ExerciseRepository>,
    data_dir: std::path::PathBuf,
    material_service: Arc<MaterialService>,
    config_service: Arc<ConfigService<SqliteConfigRepository>>,
}

impl ExerciseService {
    /// Create a new ExerciseService with all dependencies.
    pub fn new(
        generator: ExerciseGenerator,
        exercise_repo: Arc<dyn ExerciseRepository>,
        data_dir: std::path::PathBuf,
        material_service: Arc<MaterialService>,
        config_service: Arc<ConfigService<SqliteConfigRepository>>,
    ) -> Self {
        Self {
            generator,
            exercise_repo,
            data_dir,
            material_service,
            config_service,
        }
    }

    /// Get exercises for a language and difficulty.
    ///
    /// Checks cache first (D-09). If cached exercises exist, returns them.
    /// If cache miss, triggers batch generation via generate_exercises_for_language().
    pub async fn get_exercises(
        &self,
        language: &str,
        difficulty: &str,
    ) -> Result<Vec<Exercise>> {
        validate_language(language).map_err(AppError::Validation)?;
        validate_difficulty(difficulty).map_err(AppError::Validation)?;

        // Check cache first (D-09: cache forever)
        let cached = self
            .exercise_repo
            .get_by_language_and_difficulty(language, difficulty)
            .await
            .map_err(|e| AppError::Internal(e))?;

        if !cached.is_empty() {
            info!(
                "Found {} cached exercises for {}/{}",
                cached.len(),
                language,
                difficulty
            );
            return Ok(cached);
        }

        // Cache miss: generate exercises
        info!("No cached exercises for {}/{}, generating...", language, difficulty);
        self.generate_exercises_for_language(language, difficulty).await
    }

    /// Generate exercises for a language and difficulty using round-robin source routing (D-09/D-10).
    ///
    /// 1. Determine batch size from user's daily quota config
    /// 2. Load user config and parse source configuration (enabled sources in priority order)
    /// 3. Round-robin across enabled sources, taking 1 exercise per source per round
    /// 4. Skip exhausted or errored sources (D-10)
    /// 5. Cache generated exercises in SQLite
    pub async fn generate_exercises_for_language(
        &self,
        language: &str,
        difficulty: &str,
    ) -> Result<Vec<Exercise>> {
        validate_language(language).map_err(AppError::Validation)?;
        validate_difficulty(difficulty).map_err(AppError::Validation)?;

        // Step 1: Determine batch size from user's daily quota config (D-09)
        let batch_size = self.get_batch_size(language).await;

        // Step 2: Load config and parse source configuration.
        // When no config exists, parse_source_config defaults to all sources enabled.
        let config = match self.config_service.get_config().await {
            Ok(cfg) => cfg,
            Err(_) => {
                // No config in DB — use a minimal config; parse_source_config
                // will default to all sources enabled with default priority.
                UserConfig {
                    sources_enabled: None,
                    source_priority: None,
                    ..Default::default()
                }
            }
        };
        let provider = Self::create_provider_from_config(&config);
        let generator = ExerciseGenerator::new(provider);
        let (enabled_sources, _has_github, _has_web, _has_ai) =
            Self::parse_source_config(&config);

        // Step 3: Round-robin across enabled sources (D-09/D-10)
        let mut exercises: Vec<Exercise> = Vec::new();
        let mut exhausted: HashSet<String> = HashSet::new();
        let mut round = 0usize;

        while exercises.len() < batch_size as usize && exhausted.len() < enabled_sources.len() {
            let source_idx = round % enabled_sources.len();
            let source_key = &enabled_sources[source_idx];

            if exhausted.contains(source_key) {
                round += 1;
                continue;
            }

            // Try to generate 1 exercise from this source
            let result = match source_key.as_str() {
                "github" => {
                    self.generate_one_from_materials(language, difficulty, &generator, "github")
                        .await
                }
                "web" => {
                    self.generate_one_from_materials(language, difficulty, &generator, "web")
                        .await
                }
                "ai_generated" => {
                    self.generate_one_from_scratch(language, difficulty, &generator).await
                }
                _ => {
                    exhausted.insert(source_key.clone());
                    round += 1;
                    continue;
                }
            };

            match result {
                Ok(Some(exercise)) => {
                    exercises.push(exercise);
                }
                Ok(None) => {
                    // Source has no more material — mark exhausted
                    exhausted.insert(source_key.clone());
                }
                Err(e) => {
                    warn!(
                        "Source {} failed for {}/{}: {}",
                        source_key, language, difficulty, e
                    );
                    exhausted.insert(source_key.clone());
                }
            }

            round += 1;
        }

        info!(
            "Generated and cached {} exercises for {}/{} via round-robin (sources: {:?})",
            exercises.len(),
            language,
            difficulty,
            enabled_sources
        );

        Ok(exercises)
    }

    /// Generate one exercise from materials (GitHub or Web source).
    /// Fetches materials, generates exercise from first available material, inserts into DB.
    /// Returns Ok(None) when materials are exhausted.
    async fn generate_one_from_materials(
        &self,
        language: &str,
        difficulty: &str,
        generator: &ExerciseGenerator,
        source_label: &str,
    ) -> Result<Option<Exercise>> {
        // Fetch materials (reuses existing fetch_materials which covers default GitHub + curated web)
        let materials = match self
            .material_service
            .fetch_materials(language, difficulty)
            .await
        {
            Ok(mats) => mats,
            Err(e) => {
                warn!(
                    "Material fetch failed for source {}: {}",
                    source_label, e
                );
                return Err(AppError::Material(e.to_string()));
            }
        };

        if materials.is_empty() {
            return Ok(None);
        }

        // Take first material and generate exercise
        let material = &materials[0];
        let material_id = material.id;

        let generated = generator
            .generate_for_materials(&[material.clone()], language, difficulty, &self.data_dir)
            .await
            .map_err(|e| AppError::Ai(e.to_string()))?;

        if let Some((_mid, generated_ex)) = generated.into_iter().next() {
            let new_exercise = NewExercise {
                material_id: Some(material_id),
                title: generated_ex.result.title,
                description: generated_ex.result.description,
                language: generated_ex.result.language,
                difficulty: generated_ex.result.difficulty,
                todo_comment: generated_ex.result.todo_comment,
                original_code: generated_ex.result.original_code,
                exercise_code: generated_ex.result.exercise_code,
                concept: generated_ex.concept,
                start_line: generated_ex.start_line as i64,
                end_line: generated_ex.end_line as i64,
                source: source_label.to_string(),
            };
            match self.exercise_repo.insert(new_exercise).await {
                Ok(exercise) => {
                    info!(
                        "Cached {} exercise: {} ({})",
                        source_label, exercise.title, exercise.language
                    );
                    return Ok(Some(exercise));
                }
                Err(e) => {
                    warn!("Failed to insert {} exercise: {}", source_label, e);
                    return Err(AppError::Internal(e));
                }
            }
        }

        Ok(None)
    }

    /// Generate one exercise from scratch via AI (no source materials needed).
    async fn generate_one_from_scratch(
        &self,
        language: &str,
        difficulty: &str,
        generator: &ExerciseGenerator,
    ) -> Result<Option<Exercise>> {
        let scratch_exercises = generator
            .generate_from_scratch(language, difficulty, 1)
            .await
            .map_err(|e| AppError::Ai(e.to_string()))?;

        if let Some(scratch_ex) = scratch_exercises.into_iter().next() {
            let line_count = scratch_ex.original_code.lines().count() as i64;
            let new_exercise = NewExercise {
                material_id: None,
                title: scratch_ex.title,
                description: scratch_ex.description,
                language: scratch_ex.language,
                difficulty: scratch_ex.difficulty,
                todo_comment: scratch_ex.todo_comment,
                original_code: scratch_ex.original_code,
                exercise_code: scratch_ex.exercise_code,
                concept: scratch_ex.concept,
                start_line: 1,
                end_line: line_count,
                source: "ai_generated".to_string(),
            };
            match self.exercise_repo.insert(new_exercise).await {
                Ok(exercise) => {
                    info!(
                        "Cached AI-generated exercise: {} ({})",
                        exercise.title, exercise.language
                    );
                    return Ok(Some(exercise));
                }
                Err(e) => {
                    warn!("Failed to insert AI-generated exercise: {}", e);
                    return Err(AppError::Internal(e));
                }
            }
        }

        Ok(None)
    }

    /// Get a single exercise by ID.
    pub async fn get_exercise_by_id(&self, id: i64) -> Result<Option<Exercise>> {
        self.exercise_repo
            .get_by_id(id)
            .await
            .map_err(|e| AppError::Internal(e))
    }

    /// Delete an exercise by ID.
    pub async fn delete_exercise(&self, id: i64) -> Result<()> {
        self.exercise_repo
            .delete(id)
            .await
            .map_err(|e| AppError::Internal(e))
    }

    /// Get a reference to the exercise repository (for direct queries).
    pub fn exercise_repo(&self) -> Arc<dyn ExerciseRepository> {
        self.exercise_repo.clone()
    }

    /// Parse source configuration from UserConfig, applying defaults when missing.
    /// Returns (enabled_source_keys_in_priority_order, has_github_enabled, has_web_enabled, has_ai_enabled).
    fn parse_source_config(config: &UserConfig) -> (Vec<String>, bool, bool, bool) {
        // Default priority order when no config exists
        let default_priority = vec![
            "github".to_string(),
            "web".to_string(),
            "ai_generated".to_string(),
        ];

        // Parse source_priority JSON string, defaulting to ["github","web","ai_generated"]
        let priority: Vec<String> = config
            .source_priority
            .as_ref()
            .and_then(|s| serde_json::from_str(s).ok())
            .unwrap_or(default_priority);

        // Parse sources_enabled JSON string, defaulting to all true
        let enabled: HashMap<String, bool> = config
            .sources_enabled
            .as_ref()
            .and_then(|s| serde_json::from_str(s).ok())
            .unwrap_or_else(|| {
                let mut m = HashMap::new();
                m.insert("github".to_string(), true);
                m.insert("web".to_string(), true);
                m.insert("ai_generated".to_string(), true);
                m
            });

        // Filter priority list to only enabled sources
        let enabled_priority: Vec<String> = priority
            .into_iter()
            .filter(|key| enabled.get(key).copied().unwrap_or(false))
            .collect();

        let has_github = enabled.get("github").copied().unwrap_or(false);
        let has_web = enabled.get("web").copied().unwrap_or(false);
        let has_ai = enabled.get("ai_generated").copied().unwrap_or(false);

        (enabled_priority, has_github, has_web, has_ai)
    }

    /// Get the batch size for a language from user config (D-09).
    async fn get_batch_size(&self, language: &str) -> u32 {
        // Try to get config; use default if not configured or on error
        match self.config_service.get_config().await {
            Ok(config) => config
                .daily_quotas
                .iter()
                .find(|q| q.language == language)
                .map(|q| q.quota)
                .unwrap_or(5), // Default batch size
            _ => 5, // Default batch size if config not available
        }
    }

    /// Create an AI provider based on user configuration (D-04).
    ///
    /// - Local model type -> OllamaProvider
    /// - Api model type with "gpt" prefix -> OpenAiProvider
    /// - Api model type with "claude" prefix -> AnthropicProvider
    pub fn create_provider_from_config(config: &UserConfig) -> Box<dyn AiProvider> {
        match config.ai_model_type {
            ModelType::Local => {
                Box::new(crate::ai::openai::OllamaProvider::new_with_config(&config.ai_model))
            }
            ModelType::Api => {
                if config.ai_model.starts_with("gpt") {
                    // OpenAI provider requires an API key from environment
                    let api_key = std::env::var("OPENAI_API_KEY")
                        .unwrap_or_else(|_| "".to_string());
                    Box::new(crate::ai::openai::OpenAiProvider::new(&api_key, &config.ai_model))
                } else if config.ai_model.starts_with("claude") {
                    // Anthropic provider requires an API key from environment
                    let api_key = std::env::var("ANTHROPIC_API_KEY")
                        .unwrap_or_else(|_| "".to_string());
                    Box::new(crate::ai::anthropic::AnthropicProvider::new(&api_key, &config.ai_model))
                } else {
                    // Default to Ollama for unknown API model prefixes
                    Box::new(crate::ai::openai::OllamaProvider::new_with_config(&config.ai_model))
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::model::{ModelType, UserConfig};

    fn make_config(
        sources_enabled: Option<&str>,
        source_priority: Option<&str>,
    ) -> UserConfig {
        UserConfig {
            preferred_language: "rust".to_string(),
            skill_levels: vec![],
            daily_quotas: vec![],
            ai_model: "test".to_string(),
            ai_model_type: ModelType::Local,
            sources_enabled: sources_enabled.map(|s| s.to_string()),
            source_priority: source_priority.map(|s| s.to_string()),
            ..Default::default()
        }
    }

    #[test]
    fn test_all_sources_enabled_default_priority() {
        // Test 1: All sources enabled with default priority order
        let config = make_config(
            Some(r#"{"github":true,"web":true,"ai_generated":true}"#),
            Some(r#"["github","web","ai_generated"]"#),
        );
        let (enabled, _, _, _) = ExerciseService::parse_source_config(&config);
        assert_eq!(enabled, vec!["github", "web", "ai_generated"]);
    }

    #[test]
    fn test_web_disabled() {
        // Test 2: Web source disabled — only github and ai_generated remain
        let config = make_config(
            Some(r#"{"github":true,"web":false,"ai_generated":true}"#),
            Some(r#"["github","web","ai_generated"]"#),
        );
        let (enabled, _, _, _) = ExerciseService::parse_source_config(&config);
        assert_eq!(enabled, vec!["github", "ai_generated"]);
    }

    #[test]
    fn test_no_config_defaults_all_enabled() {
        // Test 4: No source config exists → defaults to all enabled with default priority
        let config = make_config(None, None);
        let (enabled, has_github, has_web, has_ai) =
            ExerciseService::parse_source_config(&config);
        assert_eq!(enabled, vec!["github", "web", "ai_generated"]);
        assert!(has_github);
        assert!(has_web);
        assert!(has_ai);
    }

    #[test]
    fn test_custom_priority_order_respected() {
        // Custom priority order: ai_generated first, github second, web last
        let config = make_config(
            Some(r#"{"github":true,"web":true,"ai_generated":true}"#),
            Some(r#"["ai_generated","github","web"]"#),
        );
        let (enabled, _, _, _) = ExerciseService::parse_source_config(&config);
        assert_eq!(enabled, vec!["ai_generated", "github", "web"]);
    }

    #[test]
    fn test_disabled_at_start_of_priority() {
        // When the first source in priority order is disabled
        let config = make_config(
            Some(r#"{"github":false,"web":true,"ai_generated":true}"#),
            Some(r#"["github","web","ai_generated"]"#),
        );
        let (enabled, has_github, _, _) = ExerciseService::parse_source_config(&config);
        assert_eq!(enabled, vec!["web", "ai_generated"]);
        assert!(!has_github);
    }

    #[test]
    fn test_malformed_enabled_json_falls_back_to_defaults() {
        // Invalid JSON for sources_enabled → should default to all enabled
        let config = make_config(
            Some("not-valid-json"),
            Some(r#"["github","web","ai_generated"]"#),
        );
        let (enabled, has_github, has_web, has_ai) =
            ExerciseService::parse_source_config(&config);
        assert_eq!(enabled, vec!["github", "web", "ai_generated"]);
        assert!(has_github);
        assert!(has_web);
        assert!(has_ai);
    }

    #[test]
    fn test_malformed_priority_json_falls_back_to_defaults() {
        // Invalid JSON for source_priority → should default to default priority
        let config = make_config(
            Some(r#"{"github":true,"web":true,"ai_generated":true}"#),
            Some("not-valid-json"),
        );
        let (enabled, _, _, _) = ExerciseService::parse_source_config(&config);
        assert_eq!(enabled, vec!["github", "web", "ai_generated"]);
    }
}
