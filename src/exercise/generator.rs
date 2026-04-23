use std::collections::HashSet;

use tokio::sync::Semaphore;
use tracing::{debug, info, warn};

use crate::ai::models::{AiError, ExerciseResult, ExerciseSection};
use crate::ai::provider::AiProvider;
use crate::db::models::Material;
use crate::exercise::models::{todo_prefix_for_language, Difficulty};

const MAX_CONCURRENT_AI_CALLS: usize = 3;

/// An exercise result paired with the section it was generated from.
/// This carries the `concept` field from the analysis section, since
/// ExerciseResult does not include concept.
pub struct GeneratedExercise {
    pub result: ExerciseResult,
    pub concept: String,
    pub start_line: usize,
    pub end_line: usize,
}

/// Exercise generator implementing the two-step AI pipeline (D-05).
///
/// Step 1 (Analyze): AI identifies exercise-worthy sections in source code.
/// Step 2 (Generate): AI creates a TODO-based exercise for each identified section.
///
/// Uses a semaphore to limit concurrent AI calls to MAX_CONCURRENT_AI_CALLS (AI-SPEC).
pub struct ExerciseGenerator {
    provider: Box<dyn AiProvider>,
    semaphore: Semaphore,
}

impl ExerciseGenerator {
    /// Create a new ExerciseGenerator with the given AI provider.
    pub fn new(provider: Box<dyn AiProvider>) -> Self {
        Self {
            provider,
            semaphore: Semaphore::new(MAX_CONCURRENT_AI_CALLS),
        }
    }

    /// Generate exercises from source code using the two-step AI pipeline (D-05).
    ///
    /// 1. Call provider.analyze() to identify exercise-worthy sections
    /// 2. Filter sections by difficulty level (D-07)
    /// 3. For each matching section, call provider.generate() to create the exercise
    /// 4. Deduplicate by (language, concept, difficulty) tuple
    /// 5. Return the generated exercises
    pub async fn generate_exercises(
        &self,
        source_code: &str,
        language: &str,
        difficulty: &str,
    ) -> Result<Vec<GeneratedExercise>, AiError> {
        // Step 1: Analyze source code (D-05)
        info!("Analyzing source code for {} at {} difficulty", language, difficulty);
        let analysis = self.provider.analyze("", source_code, language).await?;

        if analysis.sections.is_empty() {
            debug!("No exercise-worthy sections found in source code");
            return Ok(Vec::new());
        }

        // Step 2: Filter by difficulty level (D-07: strict skill-level mapping)
        let target_difficulty = Difficulty::from_str_opt(difficulty);
        let filtered_sections: Vec<&ExerciseSection> = analysis
            .sections
            .iter()
            .filter(|section| {
                match &target_difficulty {
                    Some(target) => {
                        Difficulty::from_str_opt(&section.difficulty)
                            .map(|d| d == *target)
                            .unwrap_or(false)
                    }
                    None => true,
                }
            })
            .collect();

        debug!(
            "Found {} sections, {} matching difficulty {}",
            analysis.sections.len(),
            filtered_sections.len(),
            difficulty
        );

        if filtered_sections.is_empty() {
            info!("No sections matching difficulty {} found", difficulty);
            return Ok(Vec::new());
        }

        // Step 3: Generate exercises for each matching section with concurrency limit
        let mut results = Vec::new();
        let mut seen_keys = HashSet::new();

        for section in &filtered_sections {
            // Acquire semaphore permit (limits concurrent AI calls per AI-SPEC)
            let _permit = self.semaphore.acquire().await
                .map_err(|e| AiError::ProviderError(format!("Semaphore error: {}", e)))?;

            match self.provider.generate("", section, source_code, language, difficulty).await {
                Ok(result) => {
                    // Step 4: Deduplicate by (language, concept, difficulty) tuple
                    // Use section.concept since ExerciseResult doesn't have concept
                    let key = format!("{}:{}:{}", language, section.concept, difficulty);
                    if seen_keys.insert(key) {
                        // Ensure TODO comment uses language-appropriate syntax (D-06)
                        let prefix = todo_prefix_for_language(language);
                        let exercise_result = if !result.todo_comment.starts_with(prefix) {
                            // Re-prefix the TODO comment with the correct syntax
                            ExerciseResult {
                                todo_comment: format!("{} {}", prefix, result.todo_comment.trim_start_matches("// ").trim_start_matches("# ")),
                                ..result
                            }
                        } else {
                            result
                        };
                        results.push(GeneratedExercise {
                            result: exercise_result,
                            concept: section.concept.clone(),
                            start_line: section.start_line,
                            end_line: section.end_line,
                        });
                    } else {
                        debug!("Deduplicated exercise for concept: {}", section.concept);
                    }
                }
                Err(e) => {
                    warn!("Failed to generate exercise for section {}: {}", section.concept, e);
                    // Continue with other sections rather than failing entirely
                }
            }
        }

        info!("Generated {} exercises for {} at {}", results.len(), language, difficulty);
        Ok(results)
    }

    /// Generate exercises for a list of materials (D-09: batch generation).
    ///
    /// Reads source code from each material's local_path and generates exercises.
    /// Returns pairs of (material_id, GeneratedExercise).
    pub async fn generate_for_materials(
        &self,
        materials: &[Material],
        language: &str,
        difficulty: &str,
        data_dir: &std::path::Path,
    ) -> Result<Vec<(i64, GeneratedExercise)>, AiError> {
        let mut all_results = Vec::new();

        for material in materials {
            // Read source code from material's local_path
            let full_path = data_dir.join(&material.local_path);
            let source_code = match std::fs::read_to_string(&full_path) {
                Ok(content) => content,
                Err(e) => {
                    warn!("Failed to read source file {:?}: {}", full_path, e);
                    continue;
                }
            };

            match self.generate_exercises(&source_code, language, &material.difficulty).await {
                Ok(exercises) => {
                    for generated in exercises {
                        all_results.push((material.id, generated));
                    }
                }
                Err(e) => {
                    warn!("Failed to generate exercises for material {}: {}", material.id, e);
                    // Continue with other materials
                }
            }
        }

        Ok(all_results)
    }
}