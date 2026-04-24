use serde::{Deserialize, Serialize};

use super::models::{AiError, AnalysisResult, ExerciseResult, ExerciseSection, EvaluationResult};
use super::provider::AiProvider;
use super::prompts;

const MAX_RETRIES: u32 = 2;
const ANTHROPIC_API_URL: &str = "https://api.anthropic.com/v1/messages";
const ANTHROPIC_VERSION: &str = "2023-06-01";

/// AI provider implementation for Anthropic Claude using direct reqwest calls
/// to the Messages API.
///
/// Anthropic has no official Rust SDK, so we use reqwest directly with the
/// required `x-api-key` and `anthropic-version` headers.
pub struct AnthropicProvider {
    client: reqwest::Client,
    api_key: String,
    model: String,
}

impl AnthropicProvider {
    /// Creates a new AnthropicProvider with an API key and model name.
    ///
    /// Common model names: "claude-sonnet-4-20250514", "claude-3-5-sonnet-20241022",
    /// "claude-3-haiku-20240307".
    pub fn new(api_key: &str, model: &str) -> Self {
        Self {
            client: reqwest::Client::new(),
            api_key: api_key.to_string(),
            model: model.to_string(),
        }
    }
}

/// Anthropic Messages API request body.
#[derive(Debug, Serialize)]
struct AnthropicRequest {
    model: String,
    max_tokens: u32,
    messages: Vec<AnthropicMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    system: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
}

#[derive(Debug, Serialize)]
struct AnthropicMessage {
    role: String,
    content: String,
}

/// Anthropic Messages API response body.
#[derive(Debug, Deserialize)]
struct AnthropicResponse {
    content: Vec<AnthropicContentBlock>,
}

#[derive(Debug, Deserialize)]
struct AnthropicContentBlock {
    #[serde(rename = "type")]
    content_type: String,
    text: Option<String>,
}

/// Send a request to the Anthropic Messages API.
async fn send_anthropic_request(
    client: &reqwest::Client,
    api_key: &str,
    model: &str,
    system_prompt: &str,
    user_prompt: &str,
    temperature: f32,
    max_tokens: u32,
) -> Result<String, AiError> {
    let request = AnthropicRequest {
        model: model.to_string(),
        max_tokens,
        messages: vec![AnthropicMessage {
            role: "user".to_string(),
            content: user_prompt.to_string(),
        }],
        system: Some(system_prompt.to_string()),
        temperature: Some(temperature),
    };

    let response = client
        .post(ANTHROPIC_API_URL)
        .header("x-api-key", api_key)
        .header("anthropic-version", ANTHROPIC_VERSION)
        .header("content-type", "application/json")
        .json(&request)
        .send()
        .await
        .map_err(|e| {
            let err_str = e.to_string();
            if err_str.contains("error sending request")
                || err_str.contains("connection refused")
                || err_str.contains("connect error")
                || err_str.contains("dns")
            {
                AiError::ProviderUnavailable(err_str)
            } else {
                AiError::ProviderError(err_str)
            }
        })?;

    let status = response.status();
    if status.as_u16() == 429 {
        let body = response.text().await.unwrap_or_default();
        return Err(AiError::RateLimited(body));
    }
    if status.as_u16() == 401 {
        let body = response.text().await.unwrap_or_default();
        return Err(AiError::ProviderError(format!("Authentication failed: {}", body)));
    }
    if !status.is_success() {
        let body = response.text().await.unwrap_or_default();
        return Err(AiError::ProviderError(format!(
            "Anthropic API error ({}): {}",
            status, body
        )));
    }

    let api_response: AnthropicResponse = response
        .json()
        .await
        .map_err(|e| AiError::InvalidResponse(format!("Failed to parse Anthropic response: {}", e)))?;

    // Extract text from content blocks
    let text = api_response
        .content
        .iter()
        .find(|block| block.content_type == "text")
        .and_then(|block| block.text.clone())
        .ok_or_else(|| AiError::InvalidResponse("No text content in Anthropic response".to_string()))?;

    Ok(text)
}

/// Retry a request up to MAX_RETRIES times on InvalidResponse errors.
async fn retry_anthropic_request(
    client: &reqwest::Client,
    api_key: &str,
    model: &str,
    system_prompt: &str,
    user_prompt: &str,
    temperature: f32,
    max_tokens: u32,
) -> Result<String, AiError> {
    let mut last_err = None;
    for _ in 0..=MAX_RETRIES {
        match send_anthropic_request(client, api_key, model, system_prompt, user_prompt, temperature, max_tokens).await {
            Ok(content) => return Ok(content),
            Err(e @ AiError::InvalidResponse(_)) => {
                last_err = Some(e);
                continue;
            }
            Err(e) => return Err(e),
        }
    }
    Err(last_err.unwrap_or(AiError::MaxRetriesExceeded))
}

#[async_trait::async_trait]
impl AiProvider for AnthropicProvider {
    async fn analyze(
        &self,
        _prompt: &str,
        code: &str,
        language: &str,
    ) -> Result<AnalysisResult, AiError> {
        let system_prompt = prompts::format_analyze_system_prompt(language, "mixed");
        let user_prompt = prompts::format_analyze_user_prompt(language, 5, "mixed", code);

        let content = retry_anthropic_request(
            &self.client,
            &self.api_key,
            &self.model,
            &system_prompt,
            &user_prompt,
            0.2,
            2048,
        )
        .await?;

        serde_json::from_str::<AnalysisResult>(&content)
            .map_err(|e| AiError::InvalidResponse(format!("Failed to parse analysis result: {}", e)))
    }

    async fn generate(
        &self,
        _prompt: &str,
        analysis: &ExerciseSection,
        code: &str,
        language: &str,
        difficulty: &str,
    ) -> Result<ExerciseResult, AiError> {
        let system_prompt = prompts::format_generate_system_prompt(language);
        let user_prompt = prompts::format_generate_user_prompt(
            difficulty,
            language,
            &analysis.concept,
            &analysis.reason,
            analysis.start_line,
            analysis.end_line,
            &extract_lines(code, analysis.start_line, analysis.end_line),
            code,
        );

        let content = retry_anthropic_request(
            &self.client,
            &self.api_key,
            &self.model,
            &system_prompt,
            &user_prompt,
            0.5,
            4096,
        )
        .await?;

        serde_json::from_str::<ExerciseResult>(&content)
            .map_err(|e| AiError::InvalidResponse(format!("Failed to parse exercise result: {}", e)))
    }

    async fn evaluate(
        &self,
        language: &str,
        title: &str,
        description: &str,
        original_code: &str,
        user_code: &str,
        todo_comment: &str,
    ) -> Result<EvaluationResult, AiError> {
        let system_prompt = prompts::format_evaluate_system_prompt();
        let user_prompt = prompts::format_evaluate_user_prompt(
            language, title, description, original_code, user_code, todo_comment,
        );

        let content = retry_anthropic_request(
            &self.client,
            &self.api_key,
            &self.model,
            &system_prompt,
            &user_prompt,
            0.3,
            4096,
        )
        .await?;

        serde_json::from_str::<EvaluationResult>(&content)
            .map_err(|e| AiError::InvalidResponse(format!("Failed to parse evaluation result: {}", e)))
    }
}

/// Extracts lines from source code within a given range (1-indexed).
fn extract_lines(code: &str, start_line: usize, end_line: usize) -> String {
    code.lines()
        .enumerate()
        .filter(|(i, _)| *i >= start_line.saturating_sub(1) && *i < end_line)
        .map(|(_, line)| line)
        .collect::<Vec<_>>()
        .join("\n")
}