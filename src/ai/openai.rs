use async_openai::{
    Client,
    config::OpenAIConfig,
    types::{
        ChatCompletionRequestMessage,
        ChatCompletionRequestSystemMessage,
        ChatCompletionRequestSystemMessageContent,
        ChatCompletionRequestUserMessage,
        ChatCompletionRequestUserMessageContent,
        CreateChatCompletionRequest,
        ResponseFormat,
    },
};

use super::models::{AiError, AnalysisResult, ExerciseResult, ExerciseSection, EvaluationResult};
use super::provider::AiProvider;
use super::prompts;

const MAX_RETRIES: u32 = 2;

/// AI provider implementation for Ollama (local LLM) using OpenAI-compatible API.
///
/// Ollama exposes an OpenAI-compatible endpoint at `http://localhost:11434/v1`,
/// so we reuse the async-openai client with a custom base URL.
pub struct OllamaProvider {
    client: Client<OpenAIConfig>,
    model: String,
}

impl OllamaProvider {
    /// Creates a new OllamaProvider with default settings.
    ///
    /// Uses `http://localhost:11434/v1` as the base URL and `llama3.2` as the default model.
    pub fn new() -> Self {
        let config = OpenAIConfig::new()
            .with_api_base("http://localhost:11434/v1")
            .with_api_key("ollama");
        Self {
            client: Client::with_config(config),
            model: "llama3.2".to_string(),
        }
    }

    /// Creates an OllamaProvider with a custom model name.
    pub fn new_with_config(model: &str) -> Self {
        let config = OpenAIConfig::new()
            .with_api_base("http://localhost:11434/v1")
            .with_api_key("ollama");
        Self {
            client: Client::with_config(config),
            model: model.to_string(),
        }
    }
}

impl Default for OllamaProvider {
    fn default() -> Self {
        Self::new()
    }
}

/// AI provider implementation for OpenAI API.
///
/// Uses the standard OpenAI endpoint at `https://api.openai.com/v1`.
pub struct OpenAiProvider {
    client: Client<OpenAIConfig>,
    model: String,
}

impl OpenAiProvider {
    /// Creates a new OpenAiProvider with an API key and model name.
    ///
    /// The API key should be a valid OpenAI API key.
    /// Common model names: "gpt-4o", "gpt-4o-mini", "gpt-4-turbo".
    pub fn new(api_key: &str, model: &str) -> Self {
        let config = OpenAIConfig::new()
            .with_api_base("https://api.openai.com/v1")
            .with_api_key(api_key);
        Self {
            client: Client::with_config(config),
            model: model.to_string(),
        }
    }
}

/// Helper: send a chat completion request via async-openai and parse the response content.
async fn send_chat_request(
    client: &Client<OpenAIConfig>,
    model: &str,
    system_prompt: &str,
    user_prompt: &str,
    temperature: f32,
    max_tokens: u32,
) -> Result<String, AiError> {
    let request = CreateChatCompletionRequest {
        model: model.to_string(),
        messages: vec![
            ChatCompletionRequestMessage::System(ChatCompletionRequestSystemMessage {
                content: ChatCompletionRequestSystemMessageContent::Text(system_prompt.to_string()),
                name: None,
            }),
            ChatCompletionRequestMessage::User(ChatCompletionRequestUserMessage {
                content: ChatCompletionRequestUserMessageContent::Text(user_prompt.to_string()),
                name: None,
            }),
        ],
        temperature: Some(temperature),
        max_completion_tokens: Some(max_tokens),
        response_format: Some(ResponseFormat::JsonObject),
        ..Default::default()
    };

    let response = client.chat().create(request).await.map_err(|e| {
        // Check for connection errors (provider unavailable) vs API errors
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

    let content = response
        .choices
        .first()
        .and_then(|choice| choice.message.content.clone())
        .ok_or_else(|| AiError::InvalidResponse("No content in response".to_string()))?;

    Ok(content)
}

/// Helper: retry a request up to MAX_RETRIES times on InvalidResponse errors.
async fn retry_chat_request(
    client: &Client<OpenAIConfig>,
    model: &str,
    system_prompt: &str,
    user_prompt: &str,
    temperature: f32,
    max_tokens: u32,
) -> Result<String, AiError> {
    let mut last_err = None;
    for _ in 0..=MAX_RETRIES {
        match send_chat_request(client, model, system_prompt, user_prompt, temperature, max_tokens).await {
            Ok(content) => return Ok(content),
            Err(e @ AiError::InvalidResponse(_)) => {
                last_err = Some(e);
                // Retry on invalid response
                continue;
            }
            Err(e) => return Err(e),
        }
    }
    // If all retries exhausted on InvalidResponse, try parsing what we have
    // or return MaxRetriesExceeded
    Err(last_err.unwrap_or(AiError::MaxRetriesExceeded))
}

#[async_trait::async_trait]
impl AiProvider for OllamaProvider {
    async fn analyze(
        &self,
        _prompt: &str,
        code: &str,
        language: &str,
    ) -> Result<AnalysisResult, AiError> {
        let system_prompt = prompts::format_analyze_system_prompt(language, "mixed");
        let user_prompt = prompts::format_analyze_user_prompt(language, 5, "mixed", code);

        let content = retry_chat_request(
            &self.client,
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

        let content = retry_chat_request(
            &self.client,
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

        let content = retry_chat_request(
            &self.client,
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

#[async_trait::async_trait]
impl AiProvider for OpenAiProvider {
    async fn analyze(
        &self,
        _prompt: &str,
        code: &str,
        language: &str,
    ) -> Result<AnalysisResult, AiError> {
        let system_prompt = prompts::format_analyze_system_prompt(language, "mixed");
        let user_prompt = prompts::format_analyze_user_prompt(language, 5, "mixed", code);

        let content = retry_chat_request(
            &self.client,
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

        let content = retry_chat_request(
            &self.client,
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

        let content = retry_chat_request(
            &self.client,
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