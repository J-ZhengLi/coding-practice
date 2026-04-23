// Placeholder - will be implemented in Task 2

use crate::material::github::GitHubClient;
use crate::material::scraper::TutorialScraper;
use crate::material::cache::MaterialCache;
use crate::config::service::ConfigService;
use crate::db::repository::SqliteConfigRepository;

pub struct MaterialService {
    _github_client: GitHubClient,
    _scraper: TutorialScraper,
    _cache: MaterialCache,
    _config_service: std::sync::Arc<ConfigService<SqliteConfigRepository>>,
}

impl MaterialService {
    pub fn new(
        github_client: GitHubClient,
        scraper: TutorialScraper,
        cache: MaterialCache,
        config_service: std::sync::Arc<ConfigService<SqliteConfigRepository>>,
    ) -> Self {
        Self {
            _github_client: github_client,
            _scraper: scraper,
            _cache: cache,
            _config_service: config_service,
        }
    }
}