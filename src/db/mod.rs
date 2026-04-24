pub mod models;
pub mod pool;
pub mod repository;
pub mod material_repo;
pub mod exercise_repo;
pub mod submission_repo;

pub use models::{Config, Material, NewMaterial, Exercise, NewExercise, Submission, NewSubmission, DailyScore};
pub use pool::{create_pool, get_database_path};
pub use repository::{ConfigRepository, SqliteConfigRepository};
pub use material_repo::{MaterialRepository, SqliteMaterialRepository};
pub use exercise_repo::{ExerciseRepository, SqliteExerciseRepository};
pub use submission_repo::{SubmissionRepository, SqliteSubmissionRepository};