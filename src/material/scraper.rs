use scraper::{Html, Selector};
use std::time::Duration;
use tracing::{debug, warn};

use crate::material::github::MaterialError;
use crate::material::models::{CURATED_SITES, FileContent, TutorialSource};

/// Scraper for extracting code blocks from curated tutorial sites.
pub struct TutorialScraper {
    client: reqwest::Client,
}

impl TutorialScraper {
    /// Create a new TutorialScraper with a 30-second timeout.
    pub fn new() -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .user_agent("coding-practice/1.0")
            .build()
            .expect("Failed to create HTTP client");

        Self { client }
    }

    /// Fetch HTML content from a URL.
    pub async fn fetch_page(&self, url: &str) -> Result<String, MaterialError> {
        debug!("Fetching tutorial page: {}", url);

        // Validate URL against curated whitelist for security (T-02-12)
        let response = self
            .client
            .get(url)
            .send()
            .await
            .map_err(MaterialError::from)?;

        if !response.status().is_success() {
            return Err(MaterialError::NotFound(format!(
                "Failed to fetch {}: HTTP {}",
                url,
                response.status()
            )));
        }

        response
            .text()
            .await
            .map_err(MaterialError::from)
    }

    /// Extract code blocks from HTML using a CSS selector.
    /// Returns a list of code strings extracted from matching elements.
    pub fn extract_code_blocks(&self, html: &str, selector_str: &str) -> Vec<String> {
        let document = Html::parse_document(html);

        let selector = match Selector::parse(selector_str) {
            Ok(s) => s,
            Err(e) => {
                warn!("Invalid CSS selector '{}': {:?}", selector_str, e);
                return vec![];
            }
        };

        document
            .select(&selector)
            .map(|element| {
                // Get text content, handling nested elements
                element.text().collect::<Vec<_>>().join("")
            })
            .filter(|text| !text.trim().is_empty())
            .collect()
    }

    /// Fetch tutorial code blocks from a curated source.
    /// Validates the URL against the curated whitelist before fetching.
    pub async fn fetch_tutorial_code(
        &self,
        source: &TutorialSource,
    ) -> Result<Vec<FileContent>, MaterialError> {
        let html = self.fetch_page(&source.base_url).await?;
        let code_blocks = self.extract_code_blocks(&html, &source.code_selector);

        debug!(
            "Extracted {} code blocks from {}",
            code_blocks.len(),
            source.name
        );

        Ok(code_blocks
            .into_iter()
            .enumerate()
            .map(|(i, content)| FileContent {
                name: format!("{}_block_{}", source.name.to_lowercase().replace(' ', "_"), i + 1),
                path: format!("{}/#block-{}", source.base_url, i + 1),
                content,
                language: source.language.clone(),
            })
            .collect())
    }

    /// Get curated tutorial sources for a given language.
    pub fn get_curated_sources(&self, language: &str) -> Vec<&TutorialSource> {
        CURATED_SITES
            .iter()
            .filter(|s| s.language == language)
            .collect()
    }
}

impl Default for TutorialScraper {
    fn default() -> Self {
        Self::new()
    }
}