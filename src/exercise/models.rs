use serde::{Deserialize, Serialize};

use crate::db::models::Exercise;

/// Difficulty levels for exercises (D-07: strict skill-level mapping).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Difficulty {
    Beginner,
    Intermediate,
    Advanced,
}

impl std::fmt::Display for Difficulty {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Difficulty::Beginner => write!(f, "beginner"),
            Difficulty::Intermediate => write!(f, "intermediate"),
            Difficulty::Advanced => write!(f, "advanced"),
        }
    }
}

impl Difficulty {
    /// Parse a difficulty string into a Difficulty enum.
    /// Returns None if the string is not a valid difficulty level.
    pub fn from_str_opt(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "beginner" => Some(Difficulty::Beginner),
            "intermediate" => Some(Difficulty::Intermediate),
            "advanced" => Some(Difficulty::Advanced),
            _ => None,
        }
    }
}

/// API response for a single exercise.
/// Omits original_code and internal fields from the Exercise model.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExerciseResponse {
    pub id: i64,
    pub title: String,
    pub description: String,
    pub language: String,
    pub difficulty: String,
    pub todo_comment: String,
    pub exercise_code: String,
    pub concept: String,
}

impl From<Exercise> for ExerciseResponse {
    fn from(ex: Exercise) -> Self {
        Self {
            id: ex.id,
            title: ex.title,
            description: ex.description,
            language: ex.language,
            difficulty: ex.difficulty,
            todo_comment: ex.todo_comment,
            exercise_code: ex.exercise_code,
            concept: ex.concept,
        }
    }
}

/// Request body for POST /api/exercises/generate.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateRequest {
    pub language: String,
    pub difficulty: String,
    /// Number of exercises to generate. Defaults to user's daily quota if not specified.
    pub count: Option<u32>,
}

/// Response body for POST /api/exercises/generate.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateResponse {
    pub exercises: Vec<ExerciseResponse>,
    pub generated_count: usize,
    pub cached: bool,
}

/// Allowed language values for validation (T-02-16).
pub const ALLOWED_LANGUAGES: &[&str] = &["python", "rust", "go", "cpp"];

/// Allowed difficulty values for validation (T-02-16).
pub const ALLOWED_DIFFICULTIES: &[&str] = &["beginner", "intermediate", "advanced"];

/// Validate that a language is in the allowed set.
pub fn validate_language(language: &str) -> Result<(), String> {
    if !ALLOWED_LANGUAGES.contains(&language) {
        return Err(format!(
            "Unsupported language: {}. Must be one of: {}",
            language,
            ALLOWED_LANGUAGES.join(", ")
        ));
    }
    Ok(())
}

/// Validate that a difficulty is in the allowed set.
pub fn validate_difficulty(difficulty: &str) -> Result<(), String> {
    if !ALLOWED_DIFFICULTIES.contains(&difficulty) {
        return Err(format!(
            "Unsupported difficulty: {}. Must be one of: {}",
            difficulty,
            ALLOWED_DIFFICULTIES.join(", ")
        ));
    }
    Ok(())
}

/// Return the language-appropriate TODO comment syntax (D-06).
pub fn todo_prefix_for_language(language: &str) -> &'static str {
    match language {
        "python" => "#",
        _ => "//",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_difficulty_display() {
        assert_eq!(Difficulty::Beginner.to_string(), "beginner");
        assert_eq!(Difficulty::Intermediate.to_string(), "intermediate");
        assert_eq!(Difficulty::Advanced.to_string(), "advanced");
    }

    #[test]
    fn test_difficulty_from_str() {
        assert_eq!(Difficulty::from_str_opt("beginner"), Some(Difficulty::Beginner));
        assert_eq!(Difficulty::from_str_opt("intermediate"), Some(Difficulty::Intermediate));
        assert_eq!(Difficulty::from_str_opt("advanced"), Some(Difficulty::Advanced));
        assert_eq!(Difficulty::from_str_opt("expert"), None);
    }

    #[test]
    fn test_validate_language() {
        assert!(validate_language("python").is_ok());
        assert!(validate_language("rust").is_ok());
        assert!(validate_language("go").is_ok());
        assert!(validate_language("cpp").is_ok());
        assert!(validate_language("javascript").is_err());
    }

    #[test]
    fn test_validate_difficulty() {
        assert!(validate_difficulty("beginner").is_ok());
        assert!(validate_difficulty("intermediate").is_ok());
        assert!(validate_difficulty("advanced").is_ok());
        assert!(validate_difficulty("expert").is_err());
    }

    #[test]
    fn test_todo_prefix() {
        assert_eq!(todo_prefix_for_language("python"), "#");
        assert_eq!(todo_prefix_for_language("rust"), "//");
        assert_eq!(todo_prefix_for_language("go"), "//");
        assert_eq!(todo_prefix_for_language("cpp"), "//");
    }
}