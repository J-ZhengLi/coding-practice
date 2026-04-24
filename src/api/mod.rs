pub mod config;
pub mod ollama;
pub mod materials;
pub mod exercises;
pub mod submissions;

use std::sync::Arc;

use crate::config::service::ConfigService;
use crate::db::repository::SqliteConfigRepository;
use crate::material::fetcher::MaterialService;
use crate::exercise::service::ExerciseService;
use crate::submission::service::SubmissionService;
use crate::db::submission_repo::SqliteSubmissionRepository;

/// Shared application state type matching the router state.
/// Extended from 4-tuple to 5-tuple to include SubmissionService.
pub type AppState = (
    Arc<ConfigService<SqliteConfigRepository>>,
    Arc<OllamaService>,
    Arc<MaterialService>,
    Arc<ExerciseService>,
    Arc<SubmissionService<SqliteSubmissionRepository>>,
);

pub use config::{get_config_handler, save_config_handler, check_configured_handler};
pub use ollama::{get_ollama_models_handler, OllamaService};
pub use materials::{
    get_materials_handler, fetch_materials_handler, refresh_material_handler,
    delete_material_handler,
};
pub use exercises::{
    get_exercises_handler, generate_exercises_handler, get_exercise_by_id_handler,
    delete_exercise_handler,
};
pub use submissions::{
    submit_code_handler, get_submission_handler, get_submissions_handler,
    get_solution_handler, get_daily_progress_handler, get_score_trend_handler,
};