use std::path::{Path, PathBuf};
use std::sync::Arc;

use sha2::{Digest, Sha256};
use tracing::{debug, info, warn};

use crate::db::material_repo::MaterialRepository;
use crate::db::models::{Material, NewMaterial};
use crate::material::github::MaterialError;
use crate::material::models::language_from_extension;

/// Cache layer for materials: SQLite metadata + filesystem source code storage.
pub struct MaterialCache {
    material_repo: Arc<dyn MaterialRepository>,
    data_dir: PathBuf,
}

impl MaterialCache {
    /// Create a new MaterialCache with a repository backend and data directory.
    pub fn new(material_repo: Arc<dyn MaterialRepository>, data_dir: PathBuf) -> Self {
        Self {
            material_repo,
            data_dir,
        }
    }

    /// Store a material: write source code to filesystem, insert metadata into SQLite.
    /// The local_path is derived from language and content hash (T-02-11: no user-supplied filenames).
    pub async fn store_material(&self, material: NewMaterial, content: &str) -> Result<Material, MaterialError> {
        // Check if already cached (D-03: cache forever, skip if exists)
        if let Ok(Some(_)) = self.material_repo.get_by_source_url(&material.source_url).await {
            debug!("Material already cached: {}", material.source_url);
            return self
                .material_repo
                .get_by_source_url(&material.source_url)
                .await
                .map(|m| m.expect("just confirmed it exists"))
                .map_err(|e| MaterialError::CacheError(e.to_string()));
        }

        let hash = content_hash(content);
        let extension = language_from_extension(&material.local_path)
            .map(|lang| match lang {
                "python" => ".py",
                "rust" => ".rs",
                "go" => ".go",
                "cpp" => ".cpp",
                _ => ".txt",
            })
            .unwrap_or(".txt");

        // T-02-11: Use content hash for filenames, never user-supplied names
        let relative_path = format!("materials/{}/{}{}", material.language, hash, extension);
        let full_path = self.data_dir.join(&relative_path);

        // Ensure directory exists
        if let Some(parent) = full_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| MaterialError::CacheError(format!("Failed to create directory: {}", e)))?;
        }

        // Write source code to filesystem
        std::fs::write(&full_path, content)
            .map_err(|e| MaterialError::CacheError(format!("Failed to write file: {}", e)))?;

        info!("Stored material file: {:?}", full_path);

        // Insert metadata into SQLite with the relative path
        let material = NewMaterial {
            source_url: material.source_url,
            source_type: material.source_type,
            language: material.language,
            title: material.title,
            difficulty: material.difficulty,
            local_path: relative_path,
        };

        self.material_repo
            .insert(material)
            .await
            .map_err(|e| MaterialError::CacheError(e.to_string()))
    }

    /// Get a material by ID from the database.
    pub async fn get_material(&self, id: i64) -> Result<Option<Material>, MaterialError> {
        self.material_repo
            .get_by_id(id)
            .await
            .map_err(|e| MaterialError::CacheError(e.to_string()))
    }

    /// Read cached source code content from the filesystem.
    pub fn get_cached_content(&self, local_path: &str) -> Result<String, MaterialError> {
        let full_path = self.data_dir.join(local_path);
        std::fs::read_to_string(&full_path)
            .map_err(|e| MaterialError::CacheError(format!("Failed to read cached file {}: {}", local_path, e)))
    }

    /// List materials with optional filtering by language and/or difficulty.
    pub async fn list_materials(
        &self,
        language: Option<&str>,
        difficulty: Option<&str>,
    ) -> Result<Vec<Material>, MaterialError> {
        match (language, difficulty) {
            (Some(lang), Some(diff)) => self
                .material_repo
                .get_by_language_and_difficulty(lang, diff)
                .await
                .map_err(|e| MaterialError::CacheError(e.to_string())),
            (Some(lang), None) => self
                .material_repo
                .get_by_language(lang)
                .await
                .map_err(|e| MaterialError::CacheError(e.to_string())),
            _ => self
                .material_repo
                .get_all()
                .await
                .map_err(|e| MaterialError::CacheError(e.to_string())),
        }
    }

    /// Check if a material is already cached by its source URL.
    pub async fn is_cached(&self, source_url: &str) -> Result<bool, MaterialError> {
        self.material_repo
            .get_by_source_url(source_url)
            .await
            .map(|m| m.is_some())
            .map_err(|e| MaterialError::CacheError(e.to_string()))
    }

    /// Delete a material from cache (DB + filesystem).
    /// Signals that re-fetch is needed; actual re-fetch happens via MaterialService.
    pub async fn delete_material(&self, id: i64) -> Result<Material, MaterialError> {
        let material = self
            .material_repo
            .get_by_id(id)
            .await
            .map_err(|e| MaterialError::CacheError(e.to_string()))?
            .ok_or_else(|| MaterialError::NotFound(format!("Material {} not found", id)))?;

        // Delete file from filesystem
        let full_path = self.data_dir.join(&material.local_path);
        if full_path.exists() {
            if let Err(e) = std::fs::remove_file(&full_path) {
                warn!("Failed to delete cached file {:?}: {}", full_path, e);
            }
        }

        // Delete metadata from DB
        self.material_repo
            .delete(id)
            .await
            .map_err(|e| MaterialError::CacheError(e.to_string()))?;

        info!("Deleted material {} from cache", id);
        Ok(material)
    }

    /// Return the data directory path.
    pub fn get_data_dir(&self) -> &Path {
        &self.data_dir
    }
}

/// Generate a SHA-256 hash of content for unique filenames.
fn content_hash(content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    let result = hasher.finalize();
    // Use first 16 hex chars for a short but unique filename
    format!("{:x}", result)[..16].to_string()
}