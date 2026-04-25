use chrono::{Local, NaiveDateTime, TimeZone};
use tracing::info;

use crate::db::exercise_repo::ExerciseRepository;
use crate::db::models::{NewReviewSchedule, ReviewSchedule};
use crate::db::review_schedule_repo::ReviewScheduleRepository;
use crate::db::submission_repo::SubmissionRepository;
use crate::error::{AppError, Result};
use crate::schedule::models::{DailyPlanExercise, DailyPlanResponse, DailyPlanSummary, ScheduleStatusResponse};

const INTERVALS: [i64; 4] = [1, 2, 3, 8];

pub struct ScheduleService<R: ReviewScheduleRepository> {
    schedule_repo: R,
}

impl<R: ReviewScheduleRepository> ScheduleService<R> {
    pub fn new(schedule_repo: R) -> Self {
        Self { schedule_repo }
    }

    pub async fn on_submission_completed(
        &self,
        concept: &str,
        language: &str,
        score: i32,
        exercise_id: i64,
    ) -> Result<()> {
        // Input validation per T-04-01
        if concept.trim().is_empty() {
            return Err(AppError::Validation("Concept cannot be empty".to_string()));
        }
        if language.trim().is_empty() {
            return Err(AppError::Validation("Language cannot be empty".to_string()));
        }
        if score < 0 || score > 100 {
            return Err(AppError::Validation(format!("Score must be between 0 and 100, got {}", score)));
        }
        if exercise_id <= 0 {
            return Err(AppError::Validation(format!("Exercise ID must be positive, got {}", exercise_id)));
        }

        // Per D-03: 100% score immediately retires concept
        if score == 100 {
            if let Some(schedule) = self.schedule_repo
                .get_by_concept_and_language(concept, language).await
                .map_err(|e| AppError::Internal(e))?
            {
                self.schedule_repo.retire_concept(schedule.id).await
                    .map_err(|e| AppError::Internal(e))?;
                info!("Concept '{}' mastered via 100% score shortcut per D-03", concept);
            }
            return Ok(());
        }

        match self.schedule_repo.get_by_concept_and_language(concept, language).await
            .map_err(|e| AppError::Internal(e))?
        {
            None => {
                // New concept -- create initial schedule entry per D-01, D-02
                let now = chrono::Utc::now().naive_utc();
                let next = now + chrono::Duration::days(INTERVALS[0]);
                self.schedule_repo.insert(NewReviewSchedule {
                    concept: concept.to_string(),
                    language: language.to_string(),
                    current_interval: 0,
                    last_completed_at: now,
                    next_review_at: next,
                    last_exercise_id: exercise_id,
                    status: "active".to_string(),
                }).await.map_err(|e| AppError::Internal(e))?;
                info!("Created review schedule for concept '{}' in {} (last_exercise_id={})", concept, language, exercise_id);
            }
            Some(schedule) if schedule.status == "active" => {
                // Advance to next interval per D-02 (relative to last completion)
                let next_interval = schedule.current_interval + 1;
                if next_interval as usize >= INTERVALS.len() {
                    // All intervals complete -- retire per D-04
                    self.schedule_repo.retire_concept(schedule.id).await
                        .map_err(|e| AppError::Internal(e))?;
                    info!("Concept '{}' mastered after all {} intervals per D-04", concept, INTERVALS.len());
                } else {
                    let now = chrono::Utc::now().naive_utc();
                    let next = now + chrono::Duration::days(INTERVALS[next_interval as usize]);
                    self.schedule_repo.update_schedule(
                        schedule.id, next_interval, now, next, exercise_id, "active"
                    ).await.map_err(|e| AppError::Internal(e))?;
                    info!("Advanced concept '{}' to interval {} (next review in {} days, last_exercise_id={})", concept, next_interval, INTERVALS[next_interval as usize], exercise_id);
                }
            }
            _ => {
                // Already completed -- no action needed
                info!("Concept '{}' already completed, skipping schedule update", concept);
            }
        }
        Ok(())
    }

    pub async fn get_due_reviews(&self, language: &str) -> Result<Vec<ReviewSchedule>> {
        let today_boundary = Self::today_utc_boundary();
        self.schedule_repo.get_due_reviews(language, &today_boundary).await
            .map_err(|e| AppError::Internal(e))
    }

    /// Generate the daily plan: reviews due today + new exercises up to quota, per language.
    /// Per D-05: computed on-demand.
    /// Per D-06: reviews are EXTRA on top of quotas.
    /// Per D-07: API returns reviews and new exercises; frontend handles random mixing.
    /// Per D-01: review exercise selection uses last_exercise_id to pick a DIFFERENT exercise.
    /// Per D-03/D-04: mastered (completed) concepts are excluded from new exercises.
    pub async fn get_daily_plan(
        &self,
        config_service: &crate::config::service::ConfigService<crate::db::repository::SqliteConfigRepository>,
        exercise_repo: &dyn ExerciseRepository,
        _submission_repo: &dyn SubmissionRepository,
        language: Option<&str>,
    ) -> Result<DailyPlanResponse> {
        let config = config_service.get_config().await
            .map_err(|e| AppError::Internal(anyhow::anyhow!("{}", e)))?;

        let mut all_review_exercises: Vec<DailyPlanExercise> = Vec::new();
        let mut all_new_exercises: Vec<DailyPlanExercise> = Vec::new();

        // Determine which languages to include
        let languages: Vec<&str> = match language {
            Some(lang) => vec![lang],
            None => config.daily_quotas.iter().map(|q| q.language.as_str()).collect(),
        };

        for lang in &languages {
            // Per D-06: reviews are EXTRA, not counted toward quota
            let due_reviews = self.get_due_reviews(lang).await?;

            for schedule in &due_reviews {
                // Per D-01: next review picks a different exercise from the same concept
                let concept_exercises = exercise_repo.get_by_concept(&schedule.concept, lang).await
                    .map_err(|e| AppError::Internal(e))?;

                if concept_exercises.is_empty() {
                    continue;
                }

                // Per D-01: select a DIFFERENT exercise from the one last completed.
                // If only one exercise exists for the concept, re-use it.
                let selected_exercise = concept_exercises.iter()
                    .find(|e| e.id != schedule.last_exercise_id)
                    .unwrap_or_else(|| concept_exercises.first().unwrap());

                all_review_exercises.push(DailyPlanExercise {
                    id: selected_exercise.id,
                    title: selected_exercise.title.clone(),
                    description: selected_exercise.description.clone(),
                    language: selected_exercise.language.clone(),
                    difficulty: selected_exercise.difficulty.clone(),
                    todo_comment: selected_exercise.todo_comment.clone(),
                    exercise_code: selected_exercise.exercise_code.clone(),
                    concept: selected_exercise.concept.clone(),
                    is_review: true,
                    review_interval: Some(schedule.current_interval + 1),
                });
            }

            // New exercises: up to the configured quota per language
            let quota = config.daily_quotas.iter()
                .find(|q| q.language == *lang)
                .map(|q| q.quota as usize)
                .unwrap_or(5);

            let lang_exercises = exercise_repo.get_by_language(lang).await
                .map_err(|e| AppError::Internal(e))?;

            // Per D-03 and D-04: filter out exercises whose concept has ANY schedule entry
            // (active = already being learned, completed = mastered and retired).
            // Using get_all_by_language ensures completed/mastered concepts are also excluded.
            let all_schedules = self.schedule_repo.get_all_by_language(lang).await
                .map_err(|e| AppError::Internal(e))?;
            let scheduled_concepts: Vec<String> = all_schedules.iter()
                .map(|s| s.concept.clone())
                .collect();

            let new_exercises: Vec<DailyPlanExercise> = lang_exercises.iter()
                .filter(|e| !scheduled_concepts.contains(&e.concept))
                .take(quota)
                .map(|e| DailyPlanExercise {
                    id: e.id,
                    title: e.title.clone(),
                    description: e.description.clone(),
                    language: e.language.clone(),
                    difficulty: e.difficulty.clone(),
                    todo_comment: e.todo_comment.clone(),
                    exercise_code: e.exercise_code.clone(),
                    concept: e.concept.clone(),
                    is_review: false,
                    review_interval: None,
                })
                .collect();

            all_new_exercises.extend(new_exercises);
        }

        let review_count = all_review_exercises.len() as i32;
        let new_count = all_new_exercises.len() as i32;

        // Per D-07: return as single array, let frontend handle random mixing
        let mut all_exercises = all_review_exercises;
        all_exercises.extend(all_new_exercises);

        Ok(DailyPlanResponse {
            exercises: all_exercises,
            summary: DailyPlanSummary {
                new_count,
                review_count,
            },
        })
    }

    /// Get schedule status summary for dashboard metrics.
    pub async fn get_schedule_status(
        &self,
    ) -> Result<ScheduleStatusResponse> {
        // Get all active schedules
        let all_active = self.schedule_repo.get_all_active().await
            .map_err(|e| AppError::Internal(e))?;
        let active_concepts = all_active.len() as i32;

        // Get completed schedules count
        let completed = self.schedule_repo.count_completed().await
            .map_err(|e| AppError::Internal(e))?;

        // Count overdue reviews (active + next_review_at < now)
        let today_boundary = Self::today_utc_boundary();
        let overdue = all_active.iter()
            .filter(|s| s.next_review_at < today_boundary)
            .count() as i32;

        Ok(ScheduleStatusResponse {
            active_concepts,
            completed_concepts: completed as i32,
            overdue_reviews: overdue,
        })
    }

    /// Compute "today" boundary in UTC for SQLite queries.
    /// Reviews with next_review_at <= this value are due today in local time.
    fn today_utc_boundary() -> NaiveDateTime {
        let now_local = Local::now();
        let start_of_today_local = now_local.date_naive().and_hms_opt(0, 0, 0).unwrap();
        let start_utc = Local.from_local_datetime(&start_of_today_local)
            .earliest()
            .unwrap()
            .with_timezone(&chrono::Utc);
        start_utc.naive_utc()
    }
}