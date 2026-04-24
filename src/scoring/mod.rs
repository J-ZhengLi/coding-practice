pub mod models;
pub mod service;

pub use models::{LetterGrade, score_to_grade, grade_color, grade_to_css};
pub use service::{ScoringService, validate_score};