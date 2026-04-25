use chrono::{Local, NaiveDateTime, TimeZone};
use tracing::info;

use crate::db::models::{NewReviewSchedule, ReviewSchedule};
use crate::db::review_schedule_repo::ReviewScheduleRepository;
use crate::error::Result;

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
        todo!()
    }

    pub async fn get_due_reviews(&self, language: &str) -> Result<Vec<ReviewSchedule>> {
        todo!()
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