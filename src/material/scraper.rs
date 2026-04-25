use scraper::{Html, Selector};
use std::time::Duration;
use tracing::{debug, warn};

use crate::material::github::MaterialError;
use crate::material::models::{CURATED_SITES, FileContent, TutorialSource, is_valid_code_block};

/// Maximum number of sub-pages to follow from a tutorial index page.
const MAX_SUBPAGES: usize = 20;

/// Minimum code block length to include (filters out trivial snippets).
const MIN_CODE_BLOCK_LEN: usize = 20;

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

    /// Extract internal links from an index page that point to sub-pages
    /// of the same tutorial site. Only returns links under the same base path.
    fn extract_subpage_links(&self, html: &str, base_url: &str) -> Vec<String> {
        let document = Html::parse_document(html);
        let link_selector = match Selector::parse("a[href]") {
            Ok(s) => s,
            Err(_) => return vec![],
        };

        // Normalize base URL to determine the path prefix
        let base_path = url::Url::parse(base_url)
            .ok()
            .and_then(|u| {
                let path = u.path();
                // Use parent directory as prefix for sub-pages
                if path.ends_with('/') {
                    Some(path.to_string())
                } else {
                    Some(format!("{}/", path.rfind('/').unwrap_or(0)))
                }
            })
            .unwrap_or_default();

        let mut links: Vec<String> = document
            .select(&link_selector)
            .filter_map(|el| el.value().attr("href"))
            .filter_map(|href| {
                // Only follow relative links or same-origin links
                if href.starts_with("http") || href.starts_with("//") || href.starts_with('#') {
                    return None;
                }
                // Remove anchor fragments
                let href = href.split('#').next().unwrap_or(href);
                if href.is_empty() {
                    return None;
                }
                // Resolve relative URL against base_url
                url::Url::parse(base_url)
                    .ok()
                    .and_then(|base| base.join(href).ok())
                    .map(|resolved| resolved.to_string())
            })
            .filter(|link| link != base_url && link.as_str() != base_url.trim_end_matches('/'))
            .filter(|link| {
                // Only follow links under the same base path
                link.starts_with(&base_url.trim_end_matches('/'))
            })
            .collect();

        links.sort();
        links.dedup();
        links.truncate(MAX_SUBPAGES);
        links
    }

    /// Fetch tutorial code blocks from a curated source.
    /// First tries the base URL; if no code blocks found, follows
    /// sub-page links from the index page and scrapes those.
    pub async fn fetch_tutorial_code(
        &self,
        source: &TutorialSource,
    ) -> Result<Vec<FileContent>, MaterialError> {
        let html = self.fetch_page(&source.base_url).await?;
        let mut code_blocks = self.extract_code_blocks(&html, &source.code_selector);

        debug!(
            "Extracted {} code blocks from {} (index page)",
            code_blocks.len(),
            source.name
        );

        // If index page has no code blocks, follow sub-page links
        if code_blocks.is_empty() {
            let subpage_links = self.extract_subpage_links(&html, &source.base_url);

            debug!(
                "Found {} sub-page links for {}",
                subpage_links.len(),
                source.name
            );

            for link in &subpage_links {
                match self.fetch_page(link).await {
                    Ok(sub_html) => {
                        let sub_blocks = self.extract_code_blocks(&sub_html, &source.code_selector);
                        debug!("Extracted {} code blocks from {}", sub_blocks.len(), link);
                        code_blocks.extend(sub_blocks);
                    }
                    Err(e) => {
                        warn!("Failed to fetch sub-page {}: {}", link, e);
                    }
                }

                // Stop early if we have enough material
                if code_blocks.len() >= MAX_SUBPAGES * 2 {
                    break;
                }
            }
        }

        // Filter out trivially short code blocks and low-quality material
        code_blocks.retain(|block| {
            block.trim().len() >= MIN_CODE_BLOCK_LEN
                && is_valid_code_block(block, &source.language)
        });

        debug!(
            "Total {} code blocks from {} (after filtering)",
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