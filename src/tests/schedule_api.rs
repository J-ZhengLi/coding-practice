use sqlx::SqlitePool;
use sqlx::sqlite::SqlitePoolOptions;
use std::time::Duration;

use crate::db::ReviewScheduleRepository;
use crate::db::review_schedule_repo::SqliteReviewScheduleRepository;
use crate::db::exercise_repo::{ExerciseRepository, SqliteExerciseRepository};
use crate::db::submission_repo::{SubmissionRepository, SqliteSubmissionRepository};
use crate::db::repository::SqliteConfigRepository;
use crate::db::material_repo::{MaterialRepository, SqliteMaterialRepository};
use crate::db::models::NewReviewSchedule;
use crate::schedule::ScheduleService;
use crate::config::service::ConfigService;
use chrono::naive::NaiveDate;

async fn setup_test_db() -> SqlitePool {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .acquire_timeout(Duration::from_secs(30))
        .connect("sqlite::memory:")
        .await
        .unwrap();

    // Run migrations
    sqlx::migrate!("./migrations").run(&pool).await.unwrap();
    pool
}

/// Helper: insert a material row and return its id.
/// Exercises have a FK on material_id, so we need a material first.
async fn insert_test_material(pool: &SqlitePool, language: &str) -> i64 {
    let material_repo = SqliteMaterialRepository::new(pool.clone());
    let material = material_repo.insert(crate::db::models::NewMaterial {
        source_url: format!("https://example.com/test-{}", language),
        source_type: "tutorial".to_string(),
        language: language.to_string(),
        title: format!("Test Material for {}", language),
        difficulty: "beginner".to_string(),
        local_path: "/tmp/test".to_string(),
    }).await.unwrap();
    material.id
}

/// Helper: save a config with daily quotas for a language.
/// ConfigService.get_config() requires all 4 language quotas, so we include all of them.
fn make_test_config(language: &str, quota: u32) -> crate::config::model::UserConfig {
    crate::config::model::UserConfig {
        preferred_language: language.to_string(),
        skill_levels: vec![crate::config::model::LanguageSkillLevel {
            language: language.to_string(),
            skill_level: "beginner".to_string(),
        }],
        daily_quotas: vec![
            crate::config::model::LanguageQuota { language: "python".to_string(), quota: if language == "python" { quota } else { 5 } },
            crate::config::model::LanguageQuota { language: "rust".to_string(), quota: if language == "rust" { quota } else { 5 } },
            crate::config::model::LanguageQuota { language: "go".to_string(), quota: if language == "go" { quota } else { 5 } },
            crate::config::model::LanguageQuota { language: "cpp".to_string(), quota: if language == "cpp" { quota } else { 5 } },
        ],
        ai_model: "test-model".to_string(),
        ai_model_type: crate::config::model::ModelType::Local,
        email: None,
        gmail_client_id: None,
        gmail_client_secret: None,
        smtp_host: None,
        smtp_port: None,
        smtp_user: None,
        reminder_time: None,
        reminders_enabled: None,
        gmail_refresh_token: None,
        last_reminded_at: None,
        smtp_password: None,
        sources_enabled: None,
        source_priority: None,
        github_repos: None,
        web_sources: None,
    }
}

#[tokio::test]
async fn test_get_daily_plan_returns_exercises() {
    // Per D-05, MEM-06: daily plan returns exercises (reviews + new)
    let pool = setup_test_db().await;
    let schedule_repo = SqliteReviewScheduleRepository::new(pool.clone());
    let schedule_service = ScheduleService::new(schedule_repo);
    let config_repo = SqliteConfigRepository::new(pool.clone());
    let config_service = ConfigService::new(config_repo);
    let exercise_repo = SqliteExerciseRepository::new(pool.clone());
    let submission_repo = SqliteSubmissionRepository::new(pool.clone());

    // Save a config so get_daily_plan can read quotas
    config_service.save_config(&make_test_config("python", 5)).await.unwrap();

    // Call get_daily_plan and verify it returns a DailyPlanResponse
    let plan = schedule_service.get_daily_plan(
        &config_service,
        &exercise_repo as &dyn ExerciseRepository,
        &submission_repo as &dyn SubmissionRepository,
        None,
    ).await.unwrap();

    // Verify response structure: exercises vec and summary with counts
    assert_eq!(plan.summary.new_count + plan.summary.review_count, plan.exercises.len() as i32);
}

#[tokio::test]
async fn test_get_daily_plan_respects_quotas() {
    // Per D-06, MEM-07: reviews are extra on top of quota, not counted
    let pool = setup_test_db().await;
    let schedule_repo = SqliteReviewScheduleRepository::new(pool.clone());
    let schedule_service = ScheduleService::new(schedule_repo);
    let config_repo = SqliteConfigRepository::new(pool.clone());
    let config_service = ConfigService::new(config_repo);
    let exercise_repo = SqliteExerciseRepository::new(pool.clone());
    let submission_repo = SqliteSubmissionRepository::new(pool.clone());

    // Save a config with quota_python = 2
    config_service.save_config(&make_test_config("python", 2)).await.unwrap();

    // Insert a material and then exercises for unscheduled concepts
    let material_id = insert_test_material(&pool, "python").await;
    let exercise_repo_ref = SqliteExerciseRepository::new(pool.clone());
    for i in 1..=5i64 {
        exercise_repo_ref.insert(crate::db::models::NewExercise {
            material_id: Some(material_id),
            title: format!("Exercise {}", i),
            description: format!("Description {}", i),
            language: "python".to_string(),
            difficulty: "beginner".to_string(),
            todo_comment: "TODO: implement".to_string(),
            original_code: "original".to_string(),
            exercise_code: "exercise".to_string(),
            concept: format!("concept_{}", i),
            start_line: 1,
            end_line: 10,
            source: "scraped".to_string(),
        }).await.unwrap();
    }

    let plan = schedule_service.get_daily_plan(
        &config_service,
        &exercise_repo as &dyn ExerciseRepository,
        &submission_repo as &dyn SubmissionRepository,
        Some("python"),
    ).await.unwrap();

    // Per D-06: new exercises count should not exceed quota (2)
    assert!(plan.summary.new_count <= 2);
}

#[tokio::test]
async fn test_daily_plan_reviews_extra() {
    // Per D-06: reviews do not count toward the daily quota
    let pool = setup_test_db().await;
    let schedule_repo = SqliteReviewScheduleRepository::new(pool.clone());
    let schedule_service = ScheduleService::new(schedule_repo);
    let config_repo = SqliteConfigRepository::new(pool.clone());
    let config_service = ConfigService::new(config_repo);
    let exercise_repo = SqliteExerciseRepository::new(pool.clone());
    let submission_repo = SqliteSubmissionRepository::new(pool.clone());

    // Save a config with quota_python = 2
    config_service.save_config(&make_test_config("python", 2)).await.unwrap();

    // Insert a material for the exercises
    let material_id = insert_test_material(&pool, "python").await;

    // Insert exercises for a concept that will have a due review
    let exercise_repo_ref = SqliteExerciseRepository::new(pool.clone());
    exercise_repo_ref.insert(crate::db::models::NewExercise {
        material_id: Some(material_id),
        title: "Review Exercise A".to_string(),
        description: "A review exercise".to_string(),
        language: "python".to_string(),
        difficulty: "beginner".to_string(),
        todo_comment: "TODO: implement".to_string(),
        original_code: "original".to_string(),
        exercise_code: "exercise".to_string(),
        concept: "reviewed_concept".to_string(),
        start_line: 1,
        end_line: 10,
        source: "scraped".to_string(),
    }).await.unwrap();
    // Insert a second exercise for the same concept (for D-01 different exercise selection)
    exercise_repo_ref.insert(crate::db::models::NewExercise {
        material_id: Some(material_id),
        title: "Review Exercise B".to_string(),
        description: "Another review exercise".to_string(),
        language: "python".to_string(),
        difficulty: "beginner".to_string(),
        todo_comment: "TODO: implement".to_string(),
        original_code: "original".to_string(),
        exercise_code: "exercise".to_string(),
        concept: "reviewed_concept".to_string(),
        start_line: 1,
        end_line: 10,
        source: "scraped".to_string(),
    }).await.unwrap();
    // Insert exercises for unscheduled concepts (new exercises)
    for i in 1..=5i64 {
        exercise_repo_ref.insert(crate::db::models::NewExercise {
            material_id: Some(material_id),
            title: format!("New Exercise {}", i),
            description: format!("Description {}", i),
            language: "python".to_string(),
            difficulty: "beginner".to_string(),
            todo_comment: "TODO: implement".to_string(),
            original_code: "original".to_string(),
            exercise_code: "exercise".to_string(),
            concept: format!("new_concept_{}", i),
            start_line: 1,
            end_line: 10,
            source: "scraped".to_string(),
        }).await.unwrap();
    }

    // Insert a due review schedule for "reviewed_concept"
    let past_time = NaiveDate::from_ymd_opt(2020, 1, 1).unwrap().and_hms_opt(0, 0, 0).unwrap();
    let even_older = NaiveDate::from_ymd_opt(2019, 12, 30).unwrap().and_hms_opt(0, 0, 0).unwrap();
    let schedule_repo_ref = SqliteReviewScheduleRepository::new(pool.clone());
    schedule_repo_ref.insert(NewReviewSchedule {
        concept: "reviewed_concept".to_string(),
        language: "python".to_string(),
        current_interval: 0,
        last_completed_at: even_older,
        next_review_at: past_time,
        last_exercise_id: 1,  // Last exercise was id=1, so D-01 should pick a different one
        status: "active".to_string(),
    }).await.unwrap();

    let plan = schedule_service.get_daily_plan(
        &config_service,
        &exercise_repo as &dyn ExerciseRepository,
        &submission_repo as &dyn SubmissionRepository,
        Some("python"),
    ).await.unwrap();

    // Per D-06: reviews are extra on top of quota
    assert!(plan.summary.review_count >= 1);
    // new_count should not exceed quota
    assert!(plan.summary.new_count <= 2);
    // Verify the review exercise is NOT the last_exercise_id (D-01: different exercise)
    let review_exercises: Vec<_> = plan.exercises.iter().filter(|e| e.is_review).collect();
    if review_exercises.len() == 1 && review_exercises[0].concept == "reviewed_concept" {
        // If there are multiple exercises for the concept, it should pick a different one
        // (last_exercise_id was 1, so it should pick exercise with id != 1)
        // But since we inserted 2 exercises for the concept, the review should use the other one
        assert_ne!(review_exercises[0].id, 1, "D-01: review should use a different exercise than last_exercise_id");
    }
}

#[tokio::test]
async fn test_get_schedule_status() {
    // Per MEM-04: schedule status returns active/completed/overdue counts
    let pool = setup_test_db().await;
    let schedule_repo = SqliteReviewScheduleRepository::new(pool.clone());
    let schedule_service = ScheduleService::new(schedule_repo);

    let status = schedule_service.get_schedule_status().await.unwrap();

    // Empty database: all counts should be 0
    assert_eq!(status.active_concepts, 0);
    assert_eq!(status.completed_concepts, 0);
    assert_eq!(status.overdue_reviews, 0);
}

#[tokio::test]
async fn test_completed_concepts_excluded_from_new() {
    // Per D-03 and D-04: mastered (completed) concepts do not appear as new exercises
    let pool = setup_test_db().await;
    let schedule_repo = SqliteReviewScheduleRepository::new(pool.clone());
    let schedule_service = ScheduleService::new(schedule_repo);
    let config_repo = SqliteConfigRepository::new(pool.clone());
    let config_service = ConfigService::new(config_repo);
    let exercise_repo = SqliteExerciseRepository::new(pool.clone());
    let submission_repo = SqliteSubmissionRepository::new(pool.clone());

    // Save a config
    config_service.save_config(&make_test_config("python", 5)).await.unwrap();

    // Insert a material for the exercises
    let material_id = insert_test_material(&pool, "python").await;

    // Insert exercises for a concept that will be marked completed (mastered)
    let exercise_repo_ref = SqliteExerciseRepository::new(pool.clone());
    exercise_repo_ref.insert(crate::db::models::NewExercise {
        material_id: Some(material_id),
        title: "Mastered Exercise".to_string(),
        description: "Should not appear as new".to_string(),
        language: "python".to_string(),
        difficulty: "beginner".to_string(),
        todo_comment: "TODO: implement".to_string(),
        original_code: "original".to_string(),
        exercise_code: "exercise".to_string(),
        concept: "mastered_concept".to_string(),
        start_line: 1,
        end_line: 10,
        source: "scraped".to_string(),
    }).await.unwrap();

    // Insert a completed schedule for "mastered_concept"
    let now = chrono::Utc::now().naive_utc();
    let future = now + chrono::Duration::days(999);
    let schedule_repo_ref = SqliteReviewScheduleRepository::new(pool.clone());
    schedule_repo_ref.insert(NewReviewSchedule {
        concept: "mastered_concept".to_string(),
        language: "python".to_string(),
        current_interval: 3,
        last_completed_at: now,
        next_review_at: future,
        last_exercise_id: 1,
        status: "completed".to_string(),
    }).await.unwrap();

    let plan = schedule_service.get_daily_plan(
        &config_service,
        &exercise_repo as &dyn ExerciseRepository,
        &submission_repo as &dyn SubmissionRepository,
        Some("python"),
    ).await.unwrap();

    // Per D-03/D-04: the mastered concept should NOT appear as a new exercise
    let mastered_new = plan.exercises.iter()
        .filter(|e| e.concept == "mastered_concept" && !e.is_review)
        .count();
    assert_eq!(mastered_new, 0, "Mastered concept should not appear as a new exercise");
}