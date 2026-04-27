use sqlx::SqlitePool;
use sqlx::sqlite::SqlitePoolOptions;
use std::time::Duration;

use crate::data_export::DataExportService;
use crate::db::repository::SqliteConfigRepository;
use crate::config::service::ConfigService;
use crate::config::model::{UserConfig, LanguageSkillLevel, LanguageQuota, ModelType};

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

fn make_test_config(language: &str) -> UserConfig {
    UserConfig {
        preferred_language: language.to_string(),
        skill_levels: vec![LanguageSkillLevel {
            language: language.to_string(),
            skill_level: "beginner".to_string(),
        }],
        daily_quotas: vec![
            LanguageQuota { language: "python".to_string(), quota: 5 },
            LanguageQuota { language: "rust".to_string(), quota: 5 },
            LanguageQuota { language: "go".to_string(), quota: 5 },
            LanguageQuota { language: "cpp".to_string(), quota: 5 },
        ],
        ai_model: "test-model".to_string(),
        ai_model_type: ModelType::Local,
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
    }
}

#[tokio::test]
async fn test_export_returns_valid_structure() {
    let pool = setup_test_db().await;
    let service = DataExportService::new(pool.clone());

    let data = service.export_all().await.unwrap();

    // Verify top-level structure
    assert_eq!(data["version"].as_str().unwrap(), "1.0");
    assert!(data["exported_at"].is_string());
    assert!(data["tables"].is_object());

    // Verify all 5 tables exist and are arrays
    let tables = data["tables"].as_object().unwrap();
    for table in &["config", "materials", "exercises", "submissions", "review_schedule"] {
        assert!(tables.contains_key(*table), "Missing table: {}", table);
        assert!(tables[*table].is_array(), "Table {} is not an array", table);
    }
}

#[tokio::test]
async fn test_export_includes_config_data() {
    let pool = setup_test_db().await;

    // Save a config so there's data to export
    let config_repo = SqliteConfigRepository::new(pool.clone());
    let config_service = ConfigService::new(config_repo);
    config_service.save_config(&make_test_config("python")).await.unwrap();

    let service = DataExportService::new(pool.clone());
    let data = service.export_all().await.unwrap();

    // Config table should have entries
    let config_rows = data["tables"]["config"].as_array().unwrap();
    assert!(!config_rows.is_empty(), "Config table should not be empty after saving a config");
}

#[tokio::test]
async fn test_import_validates_version() {
    let pool = setup_test_db().await;
    let service = DataExportService::new(pool.clone());

    // Wrong version
    let bad_data = serde_json::json!({
        "version": "2.0",
        "exported_at": "2024-01-01T00:00:00Z",
        "tables": {
            "config": [],
            "materials": [],
            "exercises": [],
            "submissions": [],
            "review_schedule": []
        }
    });

    let result = service.import_all(&bad_data).await;
    assert!(result.is_err(), "Should reject unsupported version");
}

#[tokio::test]
async fn test_import_validates_missing_table() {
    let pool = setup_test_db().await;
    let service = DataExportService::new(pool.clone());

    // Missing review_schedule table
    let bad_data = serde_json::json!({
        "version": "1.0",
        "exported_at": "2024-01-01T00:00:00Z",
        "tables": {
            "config": [],
            "materials": [],
            "exercises": [],
            "submissions": []
        }
    });

    let result = service.import_all(&bad_data).await;
    assert!(result.is_err(), "Should reject missing table");
}

#[tokio::test]
async fn test_export_import_roundtrip() {
    let pool = setup_test_db().await;

    // Save a config so there's data to export
    let config_repo = SqliteConfigRepository::new(pool.clone());
    let config_service = ConfigService::new(config_repo);
    config_service.save_config(&make_test_config("python")).await.unwrap();

    // Export
    let service = DataExportService::new(pool.clone());
    let exported = service.export_all().await.unwrap();
    let config_count = exported["tables"]["config"].as_array().unwrap().len();
    assert!(config_count > 0, "Should have config rows after export");

    // Clear config by saving a different one, then verify it changed
    let different_config = UserConfig {
        preferred_language: "rust".to_string(),
        ..make_test_config("rust")
    };
    config_service.save_config(&different_config).await.unwrap();

    // Import the original export
    service.import_all(&exported).await.unwrap();

    // Verify config is restored
    let restored_config = config_service.get_config().await.unwrap();
    assert_eq!(restored_config.preferred_language, "python", "Config should be restored to original after import");
}

#[tokio::test]
async fn test_import_atomic_rollback_on_error() {
    let pool = setup_test_db().await;

    // Save a config
    let config_repo = SqliteConfigRepository::new(pool.clone());
    let config_service = ConfigService::new(config_repo);
    config_service.save_config(&make_test_config("python")).await.unwrap();

    let service = DataExportService::new(pool.clone());

    // Try importing data that passes validation but fails during INSERT.
    // This exercises the actual transaction rollback path (not just input validation).
    // The exercises table has a CHECK constraint on difficulty, so an invalid
    // value will cause a runtime SQL error during the insert loop.
    let bad_import = serde_json::json!({
        "version": "1.0",
        "exported_at": "2024-01-01T00:00:00Z",
        "tables": {
            "config": [],
            "materials": [],
            "exercises": [{
                "id": 1,
                "material_id": 999,
                "title": "Test Exercise",
                "description": "A test",
                "language": "python",
                "difficulty": "invalid_level",
                "todo_comment": "TODO: fix",
                "original_code": "fn main() {}",
                "exercise_code": "fn main() {}",
                "concept": "basics",
                "start_line": 1,
                "end_line": 5
            }],
            "submissions": [],
            "review_schedule": []
        }
    });

    // The import should fail because the exercises row violates the CHECK constraint
    let result = service.import_all(&bad_import).await;
    assert!(result.is_err());

    // Original config should still exist (atomic rollback)
    let config = config_service.get_config().await.unwrap();
    assert_eq!(config.preferred_language, "python", "Config should be unchanged after failed import");
}