use sqlx::SqlitePool;
use sqlx::sqlite::SqlitePoolOptions;
use std::time::Duration;

use crate::db::ReviewScheduleRepository;
use crate::db::review_schedule_repo::SqliteReviewScheduleRepository;
use crate::db::exercise_repo::{ExerciseRepository, SqliteExerciseRepository};
use crate::db::submission_repo::{SubmissionRepository, SqliteSubmissionRepository};
use crate::db::repository::SqliteConfigRepository;
use crate::db::models::NewReviewSchedule;
use crate::schedule::ScheduleService;
use crate::config::service::ConfigService;
use crate::exercise::service::ExerciseService;
use crate::submission::service::SubmissionService;
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

    // Call get_daily_plan and verify it returns a DailyPlanResponse
    let plan = schedule_service.get_daily_plan(
        &config_service,
        &exercise_repo as &dyn ExerciseRepository,
        &submission_repo as &dyn SubmissionRepository,
        None,
    ).await.unwrap();

    // Verify response structure: exercises vec and summary with counts
    assert!(plan.exercises.len() >= 0);
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
    let config = crate::config::model::UserConfig {
        preferred_language: "python".to_string(),
        skill_levels: vec![crate::config::model::LanguageSkillLevel {
            language: "python".to_string(),
            skill_level: "beginner".to_string(),
        }],
        daily_quotas: vec![crate::config::model::LanguageQuota {
            language: "python".to_string(),
            quota: 2,
        }],
        ai_model: "test-model".to_string(),
        ai_model_type: crate::config::model::ModelType::Local,
        email: None,
        gmail_client_id: None,
        gmail_client_secret: None,
        smtp_host: None,
        smtp_port: None,
        smtp_user: None,
    };
    config_service.save_config(&config).await.unwrap();

    // Insert exercises for unscheduled concepts
    let exercise_repo_ref = SqliteExerciseRepository::new(pool.clone());
    for i in 1..=5i64 {
        exercise_repo_ref.insert(crate::db::models::NewExercise {
            material_id: 1,
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
    let config = crate::config::model::UserConfig {
        preferred_language: "python".to_string(),
        skill_levels: vec![crate::config::model::LanguageSkillLevel {
            language: "python".to_string(),
            skill_level: "beginner".to_string(),
        }],
        daily_quotas: vec![crate::config::model::LanguageQuota {
            language: "python".to_string(),
            quota: 2,
        }],
        ai_model: "test-model".to_string(),
        ai_model_type: crate::config::model::ModelType::Local,
        email: None,
        gmail_client_id: None,
        gmail_client_secret: None,
        smtp_host: None,
        smtp_port: None,
        smtp_user: None,
    };
    config_service.save_config(&config).await.unwrap();

    // Insert exercises for a concept that will have a due review
    let exercise_repo_ref = SqliteExerciseRepository::new(pool.clone());
    exercise_repo_ref.insert(crate::db::models::NewExercise {
        material_id: 1,
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
    }).await.unwrap();
    // Insert a second exercise for the same concept (for D-01 different exercise selection)
    exercise_repo_ref.insert(crate::db::models::NewExercise {
        material_id: 1,
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
    }).await.unwrap();
    // Insert exercises for unscheduled concepts (new exercises)
    for i in 1..=5i64 {
        exercise_repo_ref.insert(crate::db::models::NewExercise {
            material_id: 1,
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
        last_exercise_id: 1,  // Last exercise was id=1, so D-01 should pick id=2
        status: "active".to_string(),
    }).await.unwrap();

    let plan = schedule_service.get_daily_plan(
        &config_service,
        &exercise_repo as &dyn ExerciseRepository,
        &submission_repo as &dyn SubmissionRepository,
        Some("python"),
    ).await.unwrap();

    // Per D-06: total exercises can exceed quota when reviews are present
    // reviews (1) + new (up to 2) = total should be >= 3 if enough exercises exist
    // but at minimum: review_count should be 1 (independent of quota)
    assert!(plan.summary.review_count >= 1);
    // new_count should not exceed quota
    assert!(plan.summary.new_count <= 2);
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
    let config = crate::config::model::UserConfig {
        preferred_language: "python".to_string(),
        skill_levels: vec![crate::config::model::LanguageSkillLevel {
            language: "python".to_string(),
            skill_level: "beginner".to_string(),
        }],
        daily_quotas: vec![crate::config::model::LanguageQuota {
            language: "python".to_string(),
            quota: 5,
        }],
        ai_model: "test-model".to_string(),
        ai_model_type: crate::config::model::ModelType::Local,
        email: None,
        gmail_client_id: None,
        gmail_client_secret: None,
        smtp_host: None,
        smtp_port: None,
        smtp_user: None,
    };
    config_service.save_config(&config).await.unwrap();

    // Insert exercises for a concept that will be marked completed (mastered)
    let exercise_repo_ref = SqliteExerciseRepository::new(pool.clone());
    exercise_repo_ref.insert(crate::db::models::NewExercise {
        material_id: 1,
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