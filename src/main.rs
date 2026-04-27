mod db;
mod config;
mod api;
mod ai;
mod error;
mod material;
mod exercise;
mod submission;
mod scoring;
mod schedule;
mod reminder;
mod logging;

use axum::{
    routing::{get, post, delete},
    Router,
    Json,
    response::IntoResponse,
};
use clap::Parser;
use tower_http::cors::{Any, CorsLayer};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;

#[derive(Parser)]
#[command(name = "coding-practice", about = "AI Programming Learning Assistant")]
struct Cli {
    /// Port to listen on
    #[arg(short, long, default_value_t = 8001)]
    port: u16,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging first — guard must live for the entire program
    let _logging_guard = logging::init();

    let cli = Cli::parse();
    let port = cli.port;

    // Setup database
    let db_pool = db::create_pool(None).await?;

    // Setup services
    let config_repository = db::SqliteConfigRepository::new(db_pool.clone());
    let config_service = Arc::new(config::ConfigService::new(config_repository));
    let ollama_service = Arc::new(api::OllamaService::new());

    // Setup material services
    let material_repository = Arc::new(db::SqliteMaterialRepository::new(db_pool.clone()));

    // Check for GitHub PAT in config (D-12)
    // PAT stored in config DB under "github_pat" key; will be extensible via config API
    let github_pat: Option<String> = None;

    let github_client = material::GitHubClient::new(github_pat);
    let tutorial_scraper = material::TutorialScraper::new();

    // Data directory for cached source code files (D-10: SQLite metadata + filesystem source code)
    let data_dir = dirs::data_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("coding-practice");

    let material_cache = material::MaterialCache::new(material_repository, data_dir.clone());
    let material_service = Arc::new(material::MaterialService::new(
        github_client,
        tutorial_scraper,
        material_cache,
        config_service.clone(),
    ));

    // Setup exercise services
    let exercise_repository = Arc::new(db::SqliteExerciseRepository::new(db_pool.clone()));

    // Create ExerciseGenerator from config (D-04: provider selection based on ai_model_type)
    // Default to OllamaProvider for initial setup; will use config once available
    let config_opt = config_service.get_config().await.ok();
    let provider: Box<dyn crate::ai::AiProvider> = match &config_opt {
        Some(config) => exercise::ExerciseService::create_provider_from_config(config),
        None => Box::new(crate::ai::OllamaProvider::new()),
    };

    let generator = exercise::ExerciseGenerator::new(provider);
    let exercise_service = Arc::new(exercise::ExerciseService::new(
        generator,
        exercise_repository,
        data_dir.clone(),
        material_service.clone(),
        config_service.clone(),
    ));

    // Setup submission services
    let submission_repository = db::SqliteSubmissionRepository::new(db_pool.clone());
    let submission_service = Arc::new(submission::SubmissionService::new(submission_repository));

    // Setup schedule services
    let review_schedule_repository = db::SqliteReviewScheduleRepository::new(db_pool.clone());
    let schedule_service = Arc::new(schedule::ScheduleService::new(review_schedule_repository));

    // Setup reminder service
    let reminder_service = Arc::new(reminder::ReminderService::new(config_service.clone()));
    let _reminder_task = tokio::spawn(reminder_service.clone().run_scheduler());

    // Setup CORS
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // Build router with 7-tuple state including SubmissionService, ScheduleService, and ReminderService
    let app = Router::new()
        .route("/api/config", get(api::get_config_handler).post(api::save_config_handler))
        .route("/api/config/check", get(api::check_configured_handler))
        .route("/api/ollama/models", get(api::get_llm_models_handler))
        .route("/api/materials", get(api::get_materials_handler))
        .route("/api/materials/fetch", post(api::fetch_materials_handler))
        .route("/api/materials/{id}/refresh", post(api::refresh_material_handler))
        .route("/api/materials/{id}", delete(api::delete_material_handler))
        .route("/api/exercises", get(api::get_exercises_handler))
        .route("/api/exercises/generate", post(api::generate_exercises_handler))
        .route("/api/exercises/{id}", get(api::get_exercise_by_id_handler).delete(api::delete_exercise_handler))
        .route("/api/submissions", post(api::submit_code_handler).get(api::get_submissions_handler))
        .route("/api/submissions/{id}", get(api::get_submission_handler))
        .route("/api/submissions/{id}/solution", get(api::get_solution_handler))
        .route("/api/progress/daily", get(api::get_daily_progress_handler))
        .route("/api/progress/trend", get(api::get_score_trend_handler))
        .route("/api/schedule/daily-plan", get(api::get_daily_plan_handler))
        .route("/api/schedule/status", get(api::get_schedule_status_handler))
        .route("/api/reminders/test", post(api::test_reminder_handler))
        .route("/api/reminders/status", get(api::get_reminder_status_handler))
        .route("/health", get(health_check))
        .layer(cors)
        .with_state((config_service.clone(), ollama_service.clone(), material_service.clone(), exercise_service.clone(), submission_service.clone(), schedule_service.clone(), reminder_service.clone()));

    // Start server
    let addr = SocketAddr::from(([127, 0,  0, 1], port));
    let listener = TcpListener::bind(addr).await?;

    tracing::info!("Server listening on http://{}", addr);
    tracing::info!("Database: {:?}", db::get_database_path());

    axum::serve(listener, app).await?;

    Ok(())
}

async fn health_check() -> impl IntoResponse {
    Json(serde_json::json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}