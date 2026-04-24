use serde::{Deserialize, Serialize};

/// Letter grades assigned based on structural completeness scores.
/// Per SCORE-05: A (90-100), B (80-89), C (70-79), D (60-69), F (0-59).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum LetterGrade {
    A,
    B,
    C,
    D,
    F,
}

impl std::fmt::Display for LetterGrade {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LetterGrade::A => write!(f, "A"),
            LetterGrade::B => write!(f, "B"),
            LetterGrade::C => write!(f, "C"),
            LetterGrade::D => write!(f, "D"),
            LetterGrade::F => write!(f, "F"),
        }
    }
}

impl LetterGrade {
    pub fn from_str_opt(s: &str) -> Option<Self> {
        match s.to_uppercase().as_str() {
            "A" => Some(LetterGrade::A),
            "B" => Some(LetterGrade::B),
            "C" => Some(LetterGrade::C),
            "D" => Some(LetterGrade::D),
            "F" => Some(LetterGrade::F),
            _ => None,
        }
    }
}

/// Maps a 0-100 score to a letter grade per SCORE-05 thresholds.
/// A=90-100, B=80-89, C=70-79, D=60-69, F=0-59.
pub fn score_to_grade(score: u32) -> &'static str {
    match score {
        90..=100 => "A",
        80..=89 => "B",
        70..=79 => "C",
        60..=69 => "D",
        0..=59 => "F",
        _ => "F",
    }
}

/// Returns a hex color string for each letter grade per D-06 and UI-SPEC grade color palette.
pub fn grade_color(grade: &str) -> &'static str {
    match grade {
        "A" => "#22c55e",
        "B" => "#84cc16",
        "C" => "#eab308",
        "D" => "#f97316",
        "F" => "#ef4444",
        _ => "#6b7280",
    }
}

/// Returns CSS class names for grade background and text colors per UI-SPEC.
pub fn grade_to_css(grade: &str) -> (&'static str, &'static str) {
    match grade {
        "A" => ("bg-green-100 text-green-800", "A"),
        "B" => ("bg-lime-100 text-lime-800", "B"),
        "C" => ("bg-yellow-100 text-yellow-800", "C"),
        "D" => ("bg-orange-100 text-orange-800", "D"),
        "F" => ("bg-red-100 text-red-800", "F"),
        _ => ("bg-gray-100 text-gray-800", "?"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_score_to_grade() {
        assert_eq!(score_to_grade(100), "A");
        assert_eq!(score_to_grade(90), "A");
        assert_eq!(score_to_grade(89), "B");
        assert_eq!(score_to_grade(80), "B");
        assert_eq!(score_to_grade(79), "C");
        assert_eq!(score_to_grade(70), "C");
        assert_eq!(score_to_grade(69), "D");
        assert_eq!(score_to_grade(60), "D");
        assert_eq!(score_to_grade(59), "F");
        assert_eq!(score_to_grade(0), "F");
    }

    #[test]
    fn test_grade_color() {
        assert_eq!(grade_color("A"), "#22c55e");
        assert_eq!(grade_color("F"), "#ef4444");
    }

    #[test]
    fn test_letter_grade_display() {
        assert_eq!(LetterGrade::A.to_string(), "A");
        assert_eq!(LetterGrade::F.to_string(), "F");
    }

    #[test]
    fn test_letter_grade_from_str() {
        assert_eq!(LetterGrade::from_str_opt("A"), Some(LetterGrade::A));
        assert_eq!(LetterGrade::from_str_opt("b"), Some(LetterGrade::B));
        assert_eq!(LetterGrade::from_str_opt("X"), None);
    }
}