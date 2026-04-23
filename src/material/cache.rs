// Placeholder - will be implemented in Task 2

use crate::db::material_repo::MaterialRepository;
use crate::db::models::{Material, NewMaterial};
use crate::material::github::MaterialError;
use std::path::Path;
use std::sync::Arc;

pub struct MaterialCache {
    _material_repo: Arc<dyn MaterialRepository>,
    _data_dir: std::path::PathBuf,
}

impl MaterialCache {
    pub fn new(material_repo: Arc<dyn MaterialRepository>, data_dir: std::path::PathBuf) -> Self {
        Self {
            _material_repo: material_repo,
            _data_dir: data_dir,
        }
    }
}