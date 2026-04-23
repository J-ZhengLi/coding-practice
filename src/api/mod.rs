pub mod config;
pub mod ollama;
pub mod materials;
pub mod exercises;

pub use config::{get_config_handler, save_config_handler, check_configured_handler};
pub use ollama::{get_ollama_models_handler, OllamaService};
pub use materials::{
    get_materials_handler, fetch_materials_handler, refresh_material_handler,
    delete_material_handler,
};
pub use exercises::{
    get_exercises_handler, generate_exercises_handler, get_exercise_by_id_handler,
    delete_exercise_handler, AppState as ExerciseAppState,
};