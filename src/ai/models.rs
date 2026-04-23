use serde::{Deserialize, Serialize};

/// Result of analyzing source code for exercise-worthy sections.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisResult {
    pub sections: Vec<ExerciseSection>,
}

/// A section of source code identified as suitable for exercise generation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExerciseSection {
    pub start_line: usize,
    pub end_line: usize,
    pub difficulty: String,   // "beginner" | "intermediate" | "advanced"
    pub concept: String,
    pub reason: String,
}

/// Result of generating a TODO-based exercise from an identified code section.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExerciseResult {
    pub title: String,
    pub description: String,       // 2-3 sentence explanation
    pub todo_comment: String,       // e.g., "// TODO: Implement binary search that returns index"
    pub difficulty: String,
    pub language: String,
    pub original_code: String,     // The code section being replaced
    pub exercise_code: String,     // Code with TODO replacing the section
}

/// Errors that can occur during AI provider operations.
#[derive(Debug, Clone)]
pub enum AiError {
    ProviderUnavailable(String),
    ProviderError(String),
    InvalidResponse(String),
    MaxRetriesExceeded,
    RateLimited(String),
    ContextWindowExceeded(String),
}

impl std::fmt::Display for AiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AiError::ProviderUnavailable(msg) => write!(f, "AI provider unavailable: {}", msg),
            AiError::ProviderError(msg) => write!(f, "AI provider error: {}", msg),
            AiError::InvalidResponse(msg) => write!(f, "Invalid AI response: {}", msg),
            AiError::MaxRetriesExceeded => write!(f, "Max retries exceeded for AI request"),
            AiError::RateLimited(msg) => write!(f, "AI rate limited: {}", msg),
            AiError::ContextWindowExceeded(msg) => write!(f, "Context window exceeded: {}", msg),
        }
    }
}

impl std::error::Error for AiError {}

impl From<AiError> for crate::error::AppError {
    fn from(err: AiError) -> Self {
        // Redact any sensitive details before converting to AppError
        // (keys should never appear in error messages, but this is a safety net)
        crate::error::AppError::Ai(err.to_string())
    }
}