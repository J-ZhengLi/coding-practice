use crate::db::submission_repo::SubmissionRepository;
use crate::error::{AppError, Result};
use crate::scoring::models::{score_to_grade, LetterGrade};

/// Validate that a score is within the 0-100 range per SCORE-01.
/// Standalone function so it can be called without constructing a ScoringService.
pub fn validate_score(score: i32) -> Result<()> {
    if score < 0 || score > 100 {
        return Err(AppError::Validation(format!(
            "Score must be between 0 and 100, got {}",
            score
        )));
    }
    Ok(())
}

/// Service for computing scoring metrics: daily averages, letter grades, trends.
/// Per SCORE-03: calculates daily average score across multiple exercises.
/// Per SCORE-04: assigns letter grade based on daily average.
pub struct ScoringService<R: SubmissionRepository> {
    submission_repo: R,
}

impl<R: SubmissionRepository> ScoringService<R> {
    pub fn new(submission_repo: R) -> Self {
        Self { submission_repo }
    }

    /// Calculate letter grade from a numeric score per SCORE-05 thresholds.
    /// Per SCORE-04: system assigns letter grade based on daily average.
    pub fn calculate_letter_grade(&self, avg_score: f64) -> String {
        score_to_grade(avg_score.round() as u32).to_string()
    }

    /// Validate that a score is within the 0-100 range per SCORE-01.
    pub fn validate_score(&self, score: i32) -> Result<()> {
        validate_score(score)
    }

    /// Get the LetterGrade enum from a string representation.
    pub fn letter_grade_from_str(&self, grade: &str) -> Option<LetterGrade> {
        LetterGrade::from_str_opt(grade)
    }
}