use chrono::{Local, NaiveDateTime, TimeZone};
use tracing::info;

use crate::db::models::{NewReviewSchedule, ReviewSchedule};
use crate::db::review_schedule_repo::ReviewScheduleRepository;
use crate::error::{AppError, Result};

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