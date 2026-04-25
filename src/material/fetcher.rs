use std::sync::Arc;
use tracing::{debug, info, warn};

use crate::config::service::ConfigService;
use crate::db::models::{Material, NewMaterial};
use crate::db::repository::SqliteConfigRepository;
use crate::material::cache::MaterialCache;
use crate::material::github::{GitHubClient, MaterialError};
use crate::material::models::{estimate_difficulty, is_valid_code_block};
use crate::material::scraper::TutorialScraper;

/// Service orchestrating GitHub fetching, tutorial scraping, and caching.
pub struct MaterialService {
    github_client: GitHubClient,
    scraper: TutorialScraper,
    pub cache: MaterialCache,
    config_service: Arc<ConfigService<SqliteConfigRepository>>,
}

/// Allowed language values for validation (T-02-11).
const ALLOWED_LANGUAGES: &[&str] = &["python", "rust", "go", "cpp"];

impl MaterialService {
    /// Create a new MaterialService with all dependencies.
    pub fn new(
        github_client: GitHubClient,
        scraper: TutorialScraper,
        cache: MaterialCache,
        config_service: Arc<ConfigService<SqliteConfigRepository>>,
    ) -> Self {
        Self {
            github_client,
            scraper,
            cache,
            config_service,
        }
    }

    /// Validate that a language is supported.
    fn validate_language(language: &str) -> Result<(), MaterialError> {
        if !ALLOWED_LANGUAGES.contains(&language) {
            return Err(MaterialError::ParseError(format!(
                "Unsupported language: {}. Must be one of: {}",
                language,
                ALLOWED_LANGUAGES.join(", ")
            )));
        }
        Ok(())
    }

    /// Fetch source code materials from GitHub.
    /// Searches repos by language and difficulty, fetches top files, caches them.
    pub async fn fetch_from_github(
        &self,
        language: &str,
        difficulty: &str,
        max_repos: usize,
    ) -> Result<Vec<Material>, MaterialError> {
        Self::validate_language(language)?;

        // Check rate limit before starting (D-07, T-02-10)
        if !self.github_client.check_rate_limit()? {
            warn!("GitHub API rate limit too low, skipping fetch");
            return Ok(vec![]);
        }

        // Construct search query based on difficulty
        let query = match difficulty {
            "advanced" => format!("{} algorithms advanced", language),
            _ => format!("{} tutorial {}", language, difficulty),
        };

        let repos = self
            .github_client
            .search_repositories(&query, language)
            .await?;

        info!(
            "Found {} repos for language={}, difficulty={}",
            repos.len(),
            language,
            difficulty
        );

        let mut materials = Vec::new();

        for repo in repos.iter().take(max_repos) {
            // Parse owner/repo from full_name
            let parts: Vec<&str> = repo.full_name.split('/').collect();
            if parts.len() != 2 {
                continue;
            }
            let owner = parts[0];
            let repo_name = parts[1];

            // List source files in the repo root
            let files = match self
                .github_client
                .list_repository_files(owner, repo_name, "", language)
                .await
            {
                Ok(f) => f,
                Err(e) => {
                    warn!("Failed to list files for {}: {}", repo.full_name, e);
                    continue;
                }
            };

            // Fetch top files (limit to 5 per repo for efficiency)
            for file in files.iter().take(5) {
                // Check cache first (D-03)
                let source_url =
                    format!("https://github.com/{}/{}", repo.full_name, file.path);
                if self.cache.is_cached(&source_url).await.unwrap_or(false) {
                    debug!("Already cached: {}", source_url);
                    continue;
                }

                match self
                    .github_client
                    .get_file_content(owner, repo_name, &file.path)
                    .await
                {
                    Ok(content) => {
                        // Skip low-quality material (one-liners, trees, wrong language)
                        if !is_valid_code_block(&content, language) {
                            debug!("Skipping low-quality material: {}/{}", repo.full_name, file.path);
                            continue;
                        }

                        let estimated_diff =
                            estimate_difficulty(&file.name, language);

                        let new_material = NewMaterial {
                            source_url: source_url.clone(),
                            source_type: "github".to_string(),
                            language: language.to_string(),
                            title: format!(
                                "{} - {}",
                                repo.full_name, file.name
                            ),
                            difficulty: estimated_diff.to_string(),
                            local_path: file.path.clone(),
                        };

                        match self.cache.store_material(new_material, &content).await {
                            Ok(m) => {
                                info!("Cached GitHub material: {}", m.title);
                                materials.push(m);
                            }
                            Err(e) => {
                                warn!("Failed to cache material {}: {}", source_url, e);
                            }
                        }
                    }
                    Err(e) => {
                        warn!(
                            "Failed to fetch file {}/{}: {}",
                            repo.full_name, file.path, e
                        );
                    }
                }
            }
        }

        Ok(materials)
    }

    /// Fetch source code from curated tutorial sites for a language.
    pub async fn fetch_from_tutorial(
        &self,
        language: &str,
    ) -> Result<Vec<Material>, MaterialError> {
        Self::validate_language(language)?;

        let sources = self.scraper.get_curated_sources(language);

        if sources.is_empty() {
            debug!("No curated sources for language: {}", language);
            return Ok(vec![]);
        }

        let mut materials = Vec::new();

        for source in sources {
            // Check cache first (D-03)
            if self.cache.is_cached(&source.base_url).await.unwrap_or(false) {
                debug!("Already cached tutorial: {}", source.name);
                continue;
            }

            match self.scraper.fetch_tutorial_code(source).await {
                Ok(file_contents) => {
                    info!(
                        "Fetched {} code blocks from {}",
                        file_contents.len(),
                        source.name
                    );

                    for fc in file_contents {
                        let estimated_diff =
                            estimate_difficulty(&fc.name, language);

                        let new_material = NewMaterial {
                            source_url: fc.path.clone(),
                            source_type: "tutorial".to_string(),
                            language: fc.language.clone(),
                            title: format!("{} - {}", source.name, fc.name),
                            difficulty: estimated_diff.to_string(),
                            local_path: fc.name.clone(),
                        };

                        match self.cache.store_material(new_material, &fc.content).await {
                            Ok(m) => {
                                info!("Cached tutorial material: {}", m.title);
                                materials.push(m);
                            }
                            Err(e) => {
                                warn!("Failed to cache tutorial material: {}", e);
                            }
                        }
                    }
                }
                Err(e) => {
                    warn!(
                        "Failed to fetch tutorial from {}: {}",
                        source.name, e
                    );
                }
            }
        }

        Ok(materials)
    }

    /// Fetch materials from both GitHub and tutorial sources.
    /// Checks cache first; fetches on cache miss (D-09, D-11).
    pub async fn fetch_materials(
        &self,
        language: &str,
        difficulty: &str,
    ) -> Result<Vec<Material>, MaterialError> {
        Self::validate_language(language)?;

        // Check cache first for language+difficulty (D-09: batch generation, D-11: cache forever)
        let cached = self
            .cache
            .list_materials(Some(language), Some(difficulty))
            .await?;

        if !cached.is_empty() {
            info!(
                "Found {} cached materials for {} / {}",
                cached.len(),
                language,
                difficulty
            );
            return Ok(cached);
        }

        // Cache miss: fetch from GitHub and tutorial sources
        info!(
            "No cached materials for {} / {}, fetching...",
            language, difficulty
        );

        let mut materials = Vec::new();

        // Fetch from GitHub
        match self.fetch_from_github(language, difficulty, 10).await {
            Ok(github_materials) => {
                materials.extend(github_materials);
            }
            Err(e) => {
                warn!("GitHub fetch failed: {}", e);
            }
        }

        // Fetch from tutorial sources
        match self.fetch_from_tutorial(language).await {
            Ok(tutorial_materials) => {
                materials.extend(tutorial_materials);
            }
            Err(e) => {
                warn!("Tutorial fetch failed: {}", e);
            }
        }

        // Return all materials for this language+difficulty (including newly cached)
        self.cache
            .list_materials(Some(language), Some(difficulty))
            .await
            .map_err(|e| MaterialError::CacheError(e.to_string()))
    }

    /// Refresh a specific material by deleting the cached version and re-fetching.
    pub async fn refresh_material(&self, id: i64) -> Result<Material, MaterialError> {
        // Get the original material info
        let original = self.cache.delete_material(id).await?;

        // Re-fetch based on source type
        let content = match original.source_type.as_str() {
            "github" => {
                // Parse the GitHub URL to extract owner, repo, and path
                // URL format: https://github.com/owner/repo/path/to/file
                let url_path = original
                    .source_url
                    .strip_prefix("https://github.com/")
                    .ok_or_else(|| {
                        MaterialError::ParseError(format!(
                            "Invalid GitHub URL: {}",
                            original.source_url
                        ))
                    })?;
                let parts: Vec<&str> = url_path.splitn(3, '/').collect();
                if parts.len() < 3 {
                    return Err(MaterialError::ParseError(format!(
                        "Cannot parse GitHub URL: {}",
                        original.source_url
                    )));
                }
                let owner = parts[0];
                let repo = parts[1];
                let path = parts[2];

                self.github_client
                    .get_file_content(owner, repo, path)
                    .await?
            }
            "tutorial" => {
                // For tutorials, re-scrape the page
                let sources = self.scraper.get_curated_sources(&original.language);
                let source = sources
                    .into_iter()
                    .find(|s| original.source_url.starts_with(&s.base_url))
                    .ok_or_else(|| {
                        MaterialError::NotFound(format!(
                            "Tutorial source not found for: {}",
                            original.source_url
                        ))
                    })?;

                let file_contents = self.scraper.fetch_tutorial_code(source).await?;
                // Find the matching code block
                file_contents
                    .into_iter()
                    .find(|fc| fc.path == original.source_url)
                    .map(|fc| fc.content)
                    .unwrap_or_default()
            }
            _ => {
                return Err(MaterialError::ParseError(format!(
                    "Unknown source type: {}",
                    original.source_type
                )));
            }
        };

        // Store the refreshed material
        let new_material = NewMaterial {
            source_url: original.source_url.clone(),
            source_type: original.source_type.clone(),
            language: original.language.clone(),
            title: original.title.clone(),
            difficulty: original.difficulty.clone(),
            local_path: original.local_path.clone(),
        };

        self.cache
            .store_material(new_material, &content)
            .await
    }
}