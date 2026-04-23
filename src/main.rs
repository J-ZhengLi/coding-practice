mod db;
mod config;
mod api;
mod ai;
mod error;
mod material;
mod exercise;

use axum::{
    routing::{get, post, delete},
    Router,
    Json,
    response::IntoResponse,
};
use tower_http::cors::{Any, CorsLayer};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;

#[derive(serde::Deserialize)]
struct ServerConfig {
    server: ServerSettings,
}

#[derive(serde::Deserialize)]
struct ServerSettings {
    port: u16,
}

impl Default for ServerSettings {
    fn default() -> Self {
        Self { port: 8001 }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load configuration from config.toml if exists
    let port = if let Ok(config_content) = std::fs::read_to_string("config.toml") {
        let config: ServerConfig = toml::from_str(&config_content)
            .unwrap_or_else(|e| {
                eprintln!("Failed to parse config.toml: {}, using defaults", e);
                ServerConfig {
                    server: ServerSettings::default(),
                }
            });
        config.server.port
    } else {
        8001 // Default port per D-05
    };

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

    // Setup CORS
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // Build router with 4-tuple state including ExerciseService
    let app = Router::new()
        .route("/api/config", get(api::get_config_handler).post(api::save_config_handler))
        .route("/api/config/check", get(api::check_configured_handler))
        .route("/api/ollama/models", get(api::get_ollama_models_handler))
        .route("/api/materials", get(api::get_materials_handler))
        .route("/api/materials/fetch", post(api::fetch_materials_handler))
        .route("/api/materials/{id}/refresh", post(api::refresh_material_handler))
        .route("/api/materials/{id}", delete(api::delete_material_handler))
        .route("/api/exercises", get(api::get_exercises_handler))
        .route("/api/exercises/generate", post(api::generate_exercises_handler))
        .route("/api/exercises/{id}", get(api::get_exercise_by_id_handler).delete(api::delete_exercise_handler))
        .route("/health", get(health_check))
        .layer(cors)
        .with_state((config_service.clone(), ollama_service.clone(), material_service.clone(), exercise_service.clone()));

    // Start server
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    let listener = TcpListener::bind(addr).await?;

    println!("Server listening on http://{}", addr);
    println!("Config file: config.toml (optional)");
    println!("Database: {:?}", db::get_database_path());

    axum::serve(listener, app).await?;

    Ok(())
}

async fn health_check() -> impl IntoResponse {
    Json(serde_json::json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}