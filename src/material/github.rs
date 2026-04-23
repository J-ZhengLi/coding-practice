use reqwest::Client;
use serde::Deserialize;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Mutex;
use tracing::warn;

use crate::material::models::{RepoFileInfo, RepoInfo};

/// Errors that can occur during GitHub API operations.
#[derive(Debug, thiserror::Error)]
pub enum MaterialError {
    #[error("HTTP error: {0}")]
    HttpError(#[from] reqwest::Error),
    #[error("Rate limited. Reset at: {reset_at}")]
    RateLimited { reset_at: i64 },
    #[error("Not found: {0}")]
    NotFound(String),
    #[error("Parse error: {0}")]
    ParseError(String),
    #[error("Cache error: {0}")]
    CacheError(String),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

/// Client for interacting with the GitHub REST API.
/// Tracks rate limits and uses PAT when available.
pub struct GitHubClient {
    client: Client,
    pat: Option<String>,
    rate_limit_remaining: AtomicU32,
    rate_limit_reset: Mutex<Option<i64>>,
}

#[derive(Debug, Deserialize)]
struct SearchResponse {
    items: Vec<RepoItem>,
}

#[derive(Debug, Deserialize)]
struct RepoItem {
    full_name: String,
    html_url: String,
    stargazers_count: u32,
    description: Option<String>,
    language: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ContentEntry {
    name: String,
    path: String,
    #[serde(rename = "type")]
    entry_type: String,
}

impl GitHubClient {
    /// Create a new GitHubClient with an optional Personal Access Token.
    pub fn new(pat: Option<String>) -> Self {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            reqwest::header::ACCEPT,
            "application/vnd.github+json".parse().unwrap(),
        );
        headers.insert(
            reqwest::header::USER_AGENT,
            "coding-practice/1.0".parse().unwrap(),
        );
        if let Some(ref token) = pat {
            headers.insert(
                reqwest::header::AUTHORIZATION,
                format!("Bearer {}", token).parse().unwrap(),
            );
        }

        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .default_headers(headers)
            .build()
            .expect("Failed to create HTTP client");

        let has_pat = pat.is_some();
        Self {
            client,
            pat,
            rate_limit_remaining: AtomicU32::new(if has_pat { 5000 } else { 60 }),
            rate_limit_reset: Mutex::new(None),
        }
    }

    /// Search GitHub repositories by query and language.
    /// Returns top 10 repos sorted by stars.
    pub async fn search_repositories(
        &self,
        query: &str,
        language: &str,
    ) -> Result<Vec<RepoInfo>, MaterialError> {
        if !self.check_rate_limit()? {
            let reset = self.rate_limit_reset.lock().unwrap();
            return Err(MaterialError::RateLimited {
                reset_at: reset.unwrap_or(0),
            });
        }

        let url = format!(
            "https://api.github.com/search/repositories?q={}+language:{}&sort=stars&order=desc&per_page=10",
            urlencoding::encode(query),
            language
        );

        let response = self.client.get(&url).send().await?;

        self.update_rate_limits(&response);

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(MaterialError::ParseError(format!(
                "GitHub search API error {}: {}",
                status, body
            )));
        }

        let search: SearchResponse = response.json().await?;
        Ok(search
            .items
            .into_iter()
            .map(|r| RepoInfo {
                full_name: r.full_name,
                html_url: r.html_url,
                stargazers_count: r.stargazers_count,
                description: r.description,
                language: r.language.unwrap_or_else(|| language.to_string()),
            })
            .collect())
    }

    /// Get file content from a GitHub repository.
    /// Uses the raw content API endpoint.
    pub async fn get_file_content(
        &self,
        owner: &str,
        repo: &str,
        path: &str,
    ) -> Result<String, MaterialError> {
        if !self.check_rate_limit()? {
            let reset = self.rate_limit_reset.lock().unwrap();
            return Err(MaterialError::RateLimited {
                reset_at: reset.unwrap_or(0),
            });
        }

        let url = format!(
            "https://api.github.com/repos/{}/{}/contents/{}",
            owner, repo, path
        );

        let response = self
            .client
            .get(&url)
            .header("Accept", "application/vnd.github.raw+json")
            .send()
            .await?;

        self.update_rate_limits(&response);

        if response.status() == reqwest::StatusCode::NOT_FOUND {
            return Err(MaterialError::NotFound(format!(
                "File not found: {}/{}/{}",
                owner, repo, path
            )));
        }

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(MaterialError::ParseError(format!(
                "GitHub contents API error {}: {}",
                status, body
            )));
        }

        Ok(response.text().await?)
    }

    /// List files in a GitHub repository directory, filtered by language extensions.
    pub async fn list_repository_files(
        &self,
        owner: &str,
        repo: &str,
        path: &str,
        language: &str,
    ) -> Result<Vec<RepoFileInfo>, MaterialError> {
        if !self.check_rate_limit()? {
            let reset = self.rate_limit_reset.lock().unwrap();
            return Err(MaterialError::RateLimited {
                reset_at: reset.unwrap_or(0),
            });
        }

        let url = format!(
            "https://api.github.com/repos/{}/{}/contents/{}",
            owner, repo, path
        );

        let response = self
            .client
            .get(&url)
            .header("Accept", "application/vnd.github+json")
            .send()
            .await?;

        self.update_rate_limits(&response);

        if response.status() == reqwest::StatusCode::NOT_FOUND {
            return Ok(vec![]);
        }

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(MaterialError::ParseError(format!(
                "GitHub contents API error {}: {}",
                status, body
            )));
        }

        let entries: Vec<ContentEntry> = response.json().await?;
        let extensions = crate::material::models::language_extensions(language);

        Ok(entries
            .into_iter()
            .filter(|e| {
                if e.entry_type == "file" {
                    extensions.iter().any(|ext| e.name.ends_with(ext))
                } else {
                    false
                }
            })
            .map(|e| RepoFileInfo {
                name: e.name,
                path: e.path,
                file_type: e.entry_type,
            })
            .collect())
    }

    /// Check if we have sufficient rate limit remaining.
    /// Returns false if remaining < 5, logs a warning.
    pub fn check_rate_limit(&self) -> Result<bool, MaterialError> {
        let remaining = self.rate_limit_remaining.load(Ordering::SeqCst);
        if remaining < 5 {
            warn!(
                "GitHub API rate limit low: {} requests remaining",
                remaining
            );
            return Ok(false);
        }
        Ok(true)
    }

    /// Update rate limit counters from a GitHub API response.
    fn update_rate_limits(&self, response: &reqwest::Response) {
        if let Some(remaining) = response.headers().get("x-ratelimit-remaining") {
            if let Ok(val) = remaining.to_str() {
                if let Ok(num) = val.parse::<u32>() {
                    self.rate_limit_remaining.store(num, Ordering::SeqCst);
                }
            }
        }
        if let Some(reset) = response.headers().get("x-ratelimit-reset") {
            if let Ok(val) = reset.to_str() {
                if let Ok(timestamp) = val.parse::<i64>() {
                    if let Ok(mut guard) = self.rate_limit_reset.lock() {
                        *guard = Some(timestamp);
                    }
                }
            }
        }
    }
}