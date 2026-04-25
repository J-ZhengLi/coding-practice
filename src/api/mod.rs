pub mod config;
pub mod ollama;
pub mod materials;
pub mod exercises;
pub mod submissions;
pub mod schedule;

use std::sync::Arc;

use crate::config::service::ConfigService;
use crate::db::repository::SqliteConfigRepository;
use crate::material::fetcher::MaterialService;
use crate::exercise::service::ExerciseService;
use crate::submission::service::SubmissionService;
use crate::db::submission_repo::SqliteSubmissionRepository;
use crate::schedule::ScheduleService;
use crate::db::review_schedule_repo::SqliteReviewScheduleRepository;

/// Shared application state type matching the router state.
/// Extended from 5-tuple to 6-tuple to include ScheduleService.
pub type AppState = (
    Arc<ConfigService<SqliteConfigRepository>>,
    Arc<OllamaService>,
    Arc<MaterialService>,
    Arc<ExerciseService>,
    Arc<SubmissionService<SqliteSubmissionRepository>>,
    Arc<ScheduleService<SqliteReviewScheduleRepository>>,
);

pub use config::{get_config_handler, save_config_handler, check_configured_handler};
pub use ollama::{get_llm_models_handler, OllamaService};
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
pub use schedule::{get_daily_plan_handler, get_schedule_status_handler};