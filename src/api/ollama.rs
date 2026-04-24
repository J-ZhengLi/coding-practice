use axum::{extract::State, Json};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use crate::error::{AppError, Result};

use super::AppState;

#[derive(Debug, Deserialize)]
pub struct OllamaModel {
    pub name: String,
    pub modified_at: String,
    pub size: u64,
}

#[derive(Debug, Deserialize)]
struct OllamaModelsResponse {
    models: Vec<OllamaModel>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ModelInfo {
    pub name: String,
    pub model_type: String, // "local" or "api"
}

#[derive(Clone)]
pub struct OllamaService {
    client: Client,
    base_url: String,
    cache: Arc<Mutex<Option<(Vec<ModelInfo>, Instant)>>>,
    cache_ttl: Duration,
}

impl OllamaService {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            base_url: "http://localhost:11434".to_string(),
            cache: Arc::new(Mutex::new(None)),
            cache_ttl: Duration::from_secs(300), // 5 minutes
        }
    }

    pub async fn list_models(&self) -> Result<Vec<ModelInfo>> {
        // Check cache
        {
            let cache = self.cache.lock().unwrap_or_else(|e| e.into_inner());
            if let Some((models, timestamp)) = cache.as_ref() {
                if timestamp.elapsed() < self.cache_ttl {
                    return Ok(models.clone());
                }
            }
        }

        // Fetch from Ollama API
        let url = format!("{}/api/tags", self.base_url);
        let response = self
            .client
            .get(&url)
            .timeout(Duration::from_secs(5))
            .send()
            .await
            .map_err(|e| AppError::Ollama(format!("Failed to connect to Ollama: {}", e)))?;

        if !response.status().is_success() {
            return Err(AppError::Ollama(format!(
                "Ollama API returned error: {}",
                response.status()
            )));
        }

        let ollama_response: OllamaModelsResponse = response.json().await
            .map_err(|e| AppError::Ollama(format!("Failed to parse Ollama response: {}", e)))?;

        // Convert to ModelInfo with "(local)" suffix per D-04
        let models: Vec<ModelInfo> = ollama_response.models
            .into_iter()
            .map(|m| ModelInfo {
                name: format!("{} (local)", m.name),
                model_type: "local".to_string(),
            })
            .collect();

        // Update cache
        {
            let mut cache = self.cache.lock().unwrap_or_else(|e| e.into_inner());
            *cache = Some((models.clone(), Instant::now()));
        }

        Ok(models)
    }
}

pub async fn get_ollama_models_handler(
    State((_, ollama_service, _, _, _)): State<AppState>,
) -> Result<Json<Vec<ModelInfo>>> {
    let models = ollama_service.list_models().await?;
    Ok(Json(models))
}