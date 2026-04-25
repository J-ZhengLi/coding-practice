use tracing::info;

use crate::ai::provider::AiProvider;
use crate::db::exercise_repo::ExerciseRepository;
use crate::db::models::{NewSubmission, Submission};
use crate::db::submission_repo::SubmissionRepository;
use crate::error::{AppError, Result};
use crate::scoring::models::score_to_grade;
use crate::scoring::validate_score;
use crate::submission::models::{DailyProgressResponse, DailyScoreEntry, ScoreTrendResponse, SolutionResponse};

/// Service for submitting code, retrieving submissions, and computing scores.
/// Per D-05: submit code -> AI evaluation -> store results -> return to frontend.
pub struct SubmissionService<R: SubmissionRepository> {
    submission_repo: R,
}

impl<R: SubmissionRepository> SubmissionService<R> {
    pub fn new(submission_repo: R) -> Self {
        Self { submission_repo }
    }

    /// Get a reference to the submission repository (for direct queries).
    pub fn submission_repo(&self) -> &R {
        &self.submission_repo
    }

    /// Submit code for an exercise: call AI evaluate, compute grade, store, return result.
    /// Per D-07: unlimited re-submissions allowed - each creates a new submission.
    /// Per SCORE-01: score is 0-100 based on structural completeness comparison.
    pub async fn submit_code(
        &self,
        exercise_id: i64,
        user_code: &str,
        exercise_repo: &dyn ExerciseRepository,
        ai_provider: &dyn AiProvider,
    ) -> Result<Submission> {
        // Validate input per T-03-08: reject empty user_code
        if user_code.trim().is_empty() {
            return Err(AppError::Validation("Code cannot be empty".to_string()));
        }

        // Get exercise to retrieve original_code, language, title, description, todo_comment
        let exercise = exercise_repo
            .get_by_id(exercise_id)
            .await
            .map_err(AppError::Internal)?
            .ok_or_else(|| AppError::Validation(format!("Exercise {} not found", exercise_id)))?;

        // Per Pitfall 3: original_code is stored in the exercise model, accessible only via this service

        // Call AI evaluate per D-05: submit -> evaluate -> results
        let evaluation = ai_provider
            .evaluate(
                &exercise.language,
                &exercise.title,
                &exercise.description,
                &exercise.original_code,
                user_code,
                &exercise.todo_comment,
            )
            .await
            .map_err(|e| AppError::Evaluation(e.to_string()))?;

        // Check if user_code still contains TODO markers (partial implementation per D-14)
        let has_todo = user_code.contains("TODO");
        let is_partial = evaluation.is_partial || has_todo;

        // Serialize strengths and improvements as JSON arrays for SQLite TEXT storage
        let strengths_json = serde_json::to_string(&evaluation.feedback.strengths)
            .map_err(|e| AppError::Internal(anyhow::anyhow!("Failed to serialize strengths: {}", e)))?;
        let improvements_json = serde_json::to_string(&evaluation.feedback.improvements)
            .map_err(|e| AppError::Internal(anyhow::anyhow!("Failed to serialize improvements: {}", e)))?;

        // Compute letter grade from score per SCORE-05 thresholds
        validate_score(evaluation.score as i32)?;
        let letter_grade = score_to_grade(evaluation.score).to_string();

        let new_submission = NewSubmission {
            exercise_id,
            user_code: user_code.to_string(),
            score: evaluation.score as i32,
            letter_grade,
            is_partial,
            strengths: strengths_json,
            improvements: improvements_json,
            summary: evaluation.feedback.summary,
        };

        let submission = self.submission_repo
            .insert(new_submission)
            .await
            .map_err(AppError::Internal)?;

        info!("Stored submission {} for exercise {} with score {}", submission.id, exercise_id, submission.score);
        Ok(submission)
    }

    /// Get a single submission by ID.
    pub async fn get_submission(&self, id: i64) -> Result<Option<Submission>> {
        self.submission_repo
            .get_by_id(id)
            .await
            .map_err(AppError::Internal)
    }

    /// Get all submissions for an exercise, newest first.
    /// Per EX-08: maintains exercise history for review.
    pub async fn get_submissions_by_exercise(&self, exercise_id: i64) -> Result<Vec<Submission>> {
        self.submission_repo
            .get_by_exercise_id(exercise_id)
            .await
            .map_err(AppError::Internal)
    }

    /// Get the solution (original code) for a submission.
    /// Per D-08: viewable immediately after submission.
    /// Per Pitfall 3: original_code only accessible through this dedicated endpoint.
    pub async fn get_solution(
        &self,
        submission_id: i64,
        exercise_repo: &dyn ExerciseRepository,
    ) -> Result<SolutionResponse> {
        let submission = self.submission_repo
            .get_by_id(submission_id)
            .await
            .map_err(AppError::Internal)?
            .ok_or_else(|| AppError::Validation(format!("Submission {} not found", submission_id)))?;

        let exercise = exercise_repo
            .get_by_id(submission.exercise_id)
            .await
            .map_err(AppError::Internal)?
            .ok_or_else(|| AppError::Validation(format!("Exercise {} not found", submission.exercise_id)))?;

        Ok(SolutionResponse {
            submission_id: submission.id,
            exercise_id: exercise.id,
            user_code: submission.user_code,
            original_code: exercise.original_code,
            language: exercise.language,
            score: submission.score,
            letter_grade: submission.letter_grade,
        })
    }

    /// Get the best (highest score) submission for an exercise.
    /// Per D-07: best score is tracked for the progress dashboard.
    pub async fn get_best_score(&self, exercise_id: i64) -> Result<Option<Submission>> {
        self.submission_repo
            .get_best_score_for_exercise(exercise_id)
            .await
            .map_err(AppError::Internal)
    }

    /// Get daily progress data.
    /// Per D-11: daily average score + letter grade.
    pub async fn get_daily_progress(
        &self,
        date: &str,
    ) -> Result<DailyProgressResponse> {
        let submissions = self.submission_repo
            .get_daily_submissions(date)
            .await
            .map_err(AppError::Internal)?;

        if submissions.is_empty() {
            // Per Pitfall 5: return None for days with no submissions instead of 0
            return Ok(DailyProgressResponse {
                date: date.to_string(),
                avg_score: None,
                letter_grade: None,
                completion_count: 0,
            });
        }

        let avg_score = submissions.iter().map(|s| s.score as f64).sum::<f64>()
            / submissions.len() as f64;
        let avg_rounded = avg_score.round() as u32;
        let letter_grade = score_to_grade(avg_rounded).to_string();

        Ok(DailyProgressResponse {
            date: date.to_string(),
            avg_score: Some(avg_score),
            letter_grade: Some(letter_grade),
            completion_count: submissions.len() as i64,
        })
    }

    /// Get score trend for the last N days.
    /// Per D-11: 7-day score trend (sparkline data).
    pub async fn get_score_trend(
        &self,
        days: i32,
    ) -> Result<ScoreTrendResponse> {
        let daily_scores = self.submission_repo
            .get_recent_scores(days)
            .await
            .map_err(AppError::Internal)?;

        let entries: Vec<DailyScoreEntry> = daily_scores
            .into_iter()
            .map(|ds| DailyScoreEntry {
                date: ds.date,
                avg_score: ds.avg_score,
            })
            .collect();

        Ok(ScoreTrendResponse { scores: entries })
    }

    /// Count submissions by language and difficulty.
    /// Per D-11: completion count by language and difficulty.
    pub async fn count_completions(
        &self,
        language: &str,
        difficulty: &str,
    ) -> Result<i64> {
        self.submission_repo
            .count_by_language_and_difficulty(language, difficulty)
            .await
            .map_err(AppError::Internal)
    }
}