use async_trait::async_trait;

use super::models::{AiError, AnalysisResult, ExerciseResult, ExerciseSection, EvaluationResult};

/// Core trait for AI provider abstraction.
///
/// Every AI call goes through this trait, enabling pluggable providers
/// (Ollama, OpenAI, Anthropic) per D-04 and two-step pipeline per D-05.
#[async_trait]
pub trait AiProvider: Send + Sync {
    /// Analyze source code to identify exercise-worthy sections.
    ///
    /// Returns an `AnalysisResult` containing identified sections with
    /// line ranges, difficulty levels, concepts, and reasons.
    async fn analyze(
        &self,
        prompt: &str,
        code: &str,
        language: &str,
    ) -> Result<AnalysisResult, AiError>;

    /// Generate a TODO-based exercise from an identified code section.
    ///
    /// Takes an `ExerciseSection` identified during analysis and produces
    /// an `ExerciseResult` with title, description, TODO markers, and
    /// the exercise code.
    async fn generate(
        &self,
        prompt: &str,
        analysis: &ExerciseSection,
        code: &str,
        language: &str,
        difficulty: &str,
    ) -> Result<ExerciseResult, AiError>;

    /// Evaluate a user's code submission against the expected solution.
    ///
    /// Per AI-06: evaluates user-submitted code against expected functionality.
    /// Per AI-07: provides personalized feedback.
    /// Per AI-08: suggests specific improvements when code is suboptimal.
    async fn evaluate(
        &self,
        language: &str,
        title: &str,
        description: &str,
        original_code: &str,
        user_code: &str,
        todo_comment: &str,
    ) -> Result<EvaluationResult, AiError>;
}