use sqlx::SqlitePool;
use sqlx::sqlite::SqlitePoolOptions;
use crate::db::ReviewScheduleRepository;
use crate::db::review_schedule_repo::SqliteReviewScheduleRepository;
use crate::schedule::ScheduleService;
use crate::db::models::NewReviewSchedule;
use chrono::NaiveDateTime;
use chrono::naive::NaiveDate;
use std::time::Duration;

async fn setup_test_db() -> (SqlitePool, ScheduleService<SqliteReviewScheduleRepository>) {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .acquire_timeout(Duration::from_secs(30))
        .connect("sqlite::memory:")
        .await
        .unwrap();

    // Run migrations
    sqlx::migrate!("./migrations").run(&pool).await.unwrap();

    let repo = SqliteReviewScheduleRepository::new(pool.clone());
    let service = ScheduleService::new(repo);
    (pool, service)
}

#[tokio::test]
async fn test_on_submission_completed_creates_new_schedule() {
    // Per D-01, D-02: completing an exercise creates a schedule entry
    // Per MEM-01: intervals are 1/2/3/8
    let (pool, service) = setup_test_db().await;

    service.on_submission_completed("sorting", "python", 80, 42).await.unwrap();

    let repo = SqliteReviewScheduleRepository::new(pool.clone());
    let schedule = repo.get_by_concept_and_language("sorting", "python").await.unwrap();
    assert!(schedule.is_some());
    let s = schedule.unwrap();
    assert_eq!(s.concept, "sorting");
    assert_eq!(s.language, "python");
    assert_eq!(s.current_interval, 0);
    assert_eq!(s.status, "active");
    assert_eq!(s.last_exercise_id, 42);
}

#[tokio::test]
async fn test_100_percent_score_retires_concept() {
    // Per D-03: 100% score immediately retires the concept
    let (pool, service) = setup_test_db().await;

    // First create a schedule with score 80
    service.on_submission_completed("sorting", "python", 80, 42).await.unwrap();

    // Now score 100% -- should retire
    service.on_submission_completed("sorting", "python", 100, 43).await.unwrap();

    let repo = SqliteReviewScheduleRepository::new(pool.clone());
    let schedule = repo.get_by_concept_and_language("sorting", "python").await.unwrap().unwrap();
    assert_eq!(schedule.status, "completed");
}

#[tokio::test]
async fn test_interval_advancement_relative_to_last_completion() {
    // Per D-02: intervals relative to last completion, not first attempt
    let (pool, service) = setup_test_db().await;

    // First submission -- creates schedule with current_interval=0
    service.on_submission_completed("sorting", "python", 80, 42).await.unwrap();
    let repo = SqliteReviewScheduleRepository::new(pool.clone());
    let s = repo.get_by_concept_and_language("sorting", "python").await.unwrap().unwrap();
    assert_eq!(s.current_interval, 0);
    assert_eq!(s.last_exercise_id, 42);

    // Second submission -- advances to interval 1
    service.on_submission_completed("sorting", "python", 80, 43).await.unwrap();
    let s = repo.get_by_concept_and_language("sorting", "python").await.unwrap().unwrap();
    assert_eq!(s.current_interval, 1);
    assert_eq!(s.last_exercise_id, 43);

    // Third submission -- advances to interval 2
    service.on_submission_completed("sorting", "python", 80, 44).await.unwrap();
    let s = repo.get_by_concept_and_language("sorting", "python").await.unwrap().unwrap();
    assert_eq!(s.current_interval, 2);
    assert_eq!(s.last_exercise_id, 44);
}

#[tokio::test]
async fn test_concept_retirement_after_all_intervals() {
    // Per D-04: after 4 intervals, concept is permanently retired
    let (pool, service) = setup_test_db().await;

    // Create initial schedule (interval 0)
    service.on_submission_completed("sorting", "python", 80, 42).await.unwrap();
    // Advance through intervals 1, 2, 3
    service.on_submission_completed("sorting", "python", 80, 43).await.unwrap();
    service.on_submission_completed("sorting", "python", 80, 44).await.unwrap();
    service.on_submission_completed("sorting", "python", 80, 45).await.unwrap();

    let repo = SqliteReviewScheduleRepository::new(pool.clone());
    let s = repo.get_by_concept_and_language("sorting", "python").await.unwrap().unwrap();
    assert_eq!(s.current_interval, 3);
    assert_eq!(s.status, "active");

    // Fifth submission -- all intervals done, should retire
    service.on_submission_completed("sorting", "python", 80, 46).await.unwrap();
    let s = repo.get_by_concept_and_language("sorting", "python").await.unwrap().unwrap();
    assert_eq!(s.status, "completed");
}

#[tokio::test]
async fn test_due_reviews_returns_active_schedules() {
    // Per D-10: timezone-aware "due today" check
    let (pool, service) = setup_test_db().await;

    let repo = SqliteReviewScheduleRepository::new(pool.clone());

    // Insert a schedule with next_review_at in the past (should be due)
    let past_time = NaiveDate::from_ymd_opt(2020, 1, 1).unwrap().and_hms_opt(0, 0, 0).unwrap();
    let even_older = NaiveDate::from_ymd_opt(2019, 12, 30).unwrap().and_hms_opt(0, 0, 0).unwrap();
    repo.insert(NewReviewSchedule {
        concept: "loops".to_string(),
        language: "python".to_string(),
        current_interval: 0,
        last_completed_at: even_older,
        next_review_at: past_time,
        last_exercise_id: 10,
        status: "active".to_string(),
    }).await.unwrap();

    let due = service.get_due_reviews("python").await.unwrap();
    assert_eq!(due.len(), 1);
    assert_eq!(due[0].concept, "loops");
}

#[tokio::test]
async fn test_already_completed_concept_noop() {
    // Completed concepts should not be modified
    let (pool, service) = setup_test_db().await;

    let repo = SqliteReviewScheduleRepository::new(pool.clone());

    // Insert a completed schedule
    let now = chrono::Utc::now().naive_utc();
    let future = now + chrono::Duration::days(999);
    repo.insert(NewReviewSchedule {
        concept: "variables".to_string(),
        language: "python".to_string(),
        current_interval: 2,
        last_completed_at: now,
        next_review_at: future,
        last_exercise_id: 50,
        status: "completed".to_string(),
    }).await.unwrap();

    // Calling on_submission_completed on completed concept should be a no-op
    service.on_submission_completed("variables", "python", 80, 99).await.unwrap();

    let s = repo.get_by_concept_and_language("variables", "python").await.unwrap().unwrap();
    assert_eq!(s.status, "completed");
    // last_exercise_id should remain unchanged
    assert_eq!(s.last_exercise_id, 50);
}

#[tokio::test]
async fn test_last_exercise_id_stored_and_updated() {
    // Per D-01: on_submission_completed stores the exercise_id
    // so the next review can select a DIFFERENT exercise
    let (pool, service) = setup_test_db().await;

    let repo = SqliteReviewScheduleRepository::new(pool.clone());

    // First submission with exercise_id=10
    service.on_submission_completed("loops", "python", 80, 10).await.unwrap();
    let s = repo.get_by_concept_and_language("loops", "python").await.unwrap().unwrap();
    assert_eq!(s.last_exercise_id, 10);

    // Second submission with exercise_id=20 -- should update last_exercise_id
    service.on_submission_completed("loops", "python", 80, 20).await.unwrap();
    let s = repo.get_by_concept_and_language("loops", "python").await.unwrap().unwrap();
    assert_eq!(s.last_exercise_id, 20);
}