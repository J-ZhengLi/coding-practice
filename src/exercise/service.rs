use std::sync::Arc;

use tracing::{info, warn};

use crate::ai::provider::AiProvider;
use crate::config::model::{ModelType, UserConfig};
use crate::config::service::ConfigService;
use crate::db::exercise_repo::ExerciseRepository;
use crate::db::models::{Exercise, NewExercise};
use crate::db::repository::SqliteConfigRepository;
use crate::error::{AppError, Result};
use crate::exercise::generator::{ExerciseGenerator, GeneratedExercise};
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

    /// Generate exercises for a language and difficulty (D-09: batch generation).
    ///
    /// 1. Fetch materials via material_service
    /// 2. Read user config for daily_quota to determine batch size
    /// 3. Call generator.generate_for_materials() for up to batch_size materials
    /// 4. For each GeneratedExercise, create NewExercise and insert via exercise_repo
    /// 5. Return newly generated exercises
    pub async fn generate_exercises_for_language(
        &self,
        language: &str,
        difficulty: &str,
    ) -> Result<Vec<Exercise>> {
        validate_language(language).map_err(AppError::Validation)?;
        validate_difficulty(difficulty).map_err(AppError::Validation)?;

        // Step 1: Fetch materials (D-03: on-demand)
        let materials = self
            .material_service
            .fetch_materials(language, difficulty)
            .await
            .map_err(|e| AppError::Material(e.to_string()))?;

        if materials.is_empty() {
            warn!("No materials available for {}/{}", language, difficulty);
            return Ok(Vec::new());
        }

        // Step 2: Determine batch size from user's daily quota config (D-09)
        let batch_size = self.get_batch_size(language).await;

        // Limit materials to batch_size
        let materials_batch: Vec<_> = materials.into_iter().take(batch_size as usize).collect();

        // Step 3: Generate exercises from materials
        let generated = self
            .generator
            .generate_for_materials(&materials_batch, language, difficulty, &self.data_dir)
            .await
            .map_err(|e| AppError::Ai(e.to_string()))?;

        // Step 4: Store generated exercises in SQLite
        let mut exercises = Vec::new();
        for (material_id, generated_ex) in generated {
            let new_exercise = generated_to_new_exercise(material_id, &generated_ex);
            match self.exercise_repo.insert(new_exercise).await {
                Ok(exercise) => {
                    info!("Cached exercise: {} ({})", exercise.title, exercise.language);
                    exercises.push(exercise);
                }
                Err(e) => {
                    warn!("Failed to insert exercise: {}", e);
                }
            }
        }

        info!(
            "Generated and cached {} exercises for {}/{}",
            exercises.len(),
            language,
            difficulty
        );

        Ok(exercises)
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

/// Convert a GeneratedExercise into a NewExercise for database insertion.
fn generated_to_new_exercise(material_id: i64, generated_ex: &GeneratedExercise) -> NewExercise {
    NewExercise {
        material_id,
        title: generated_ex.result.title.clone(),
        description: generated_ex.result.description.clone(),
        language: generated_ex.result.language.clone(),
        difficulty: generated_ex.result.difficulty.clone(),
        todo_comment: generated_ex.result.todo_comment.clone(),
        original_code: generated_ex.result.original_code.clone(),
        exercise_code: generated_ex.result.exercise_code.clone(),
        concept: generated_ex.concept.clone(),
        start_line: generated_ex.start_line as i64,
        end_line: generated_ex.end_line as i64,
    }
}