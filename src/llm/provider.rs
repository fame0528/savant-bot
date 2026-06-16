//! LLM provider abstraction.
//!
//! Per FID-2026-0616-008, this module defines a `Provider` trait that
//! abstracts over LLM providers, with `OpenRouterProvider` as the v1
//! backing implementation. A `MockProvider` is included for unit tests.
//!
//! OpenRouter's native model-array failover (sending a `models: []` array)
//! implements the pattern 8 (provider failover) without our own multi-
//! provider complexity. See the architecture doc §3.2 line 104.

use std::sync::Mutex;
use std::time::Duration;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// OpenRouter chat-completions endpoint (per their API spec).
const OPENROUTER_API_URL: &str = "https://openrouter.ai/api/v1/chat/completions";

/// Default HTTP timeout for LLM requests. 60s accommodates the 5-30s
/// typical LLM response time (per doc §3.1) plus network overhead.
const HTTP_TIMEOUT: Duration = Duration::from_secs(60);

/// A single message in a chat conversation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

impl ChatMessage {
    /// Convenience constructor for a user message.
    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: "user".to_string(),
            content: content.into(),
        }
    }

    /// Convenience constructor for a system message.
    pub fn system(content: impl Into<String>) -> Self {
        Self {
            role: "system".to_string(),
            content: content.into(),
        }
    }

    /// Convenience constructor for an assistant message (for context).
    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            role: "assistant".to_string(),
            content: content.into(),
        }
    }
}

/// A chat request to be sent to the provider.
#[derive(Debug, Clone)]
pub struct ChatRequest {
    pub messages: Vec<ChatMessage>,
    pub temperature: f32,
    pub max_tokens: u32,
    /// Fallback models for OpenRouter's `models: []` array (per doc §3.2 line 104).
    /// If empty, the request uses only the primary model.
    pub models: Vec<String>,
}

impl ChatRequest {
    /// Build a request with sensible defaults.
    pub fn new(messages: Vec<ChatMessage>) -> Self {
        Self {
            messages,
            temperature: 0.7,
            max_tokens: 1024,
            models: Vec::new(),
        }
    }

    pub fn with_temperature(mut self, t: f32) -> Self {
        self.temperature = t;
        self
    }

    pub fn with_max_tokens(mut self, n: u32) -> Self {
        self.max_tokens = n;
        self
    }

    pub fn with_fallback_models(mut self, models: Vec<String>) -> Self {
        self.models = models;
        self
    }
}

/// A chat response from the provider.
#[derive(Debug, Clone)]
pub struct ChatResponse {
    pub content: String,
    pub model: String,
    pub usage: Option<Usage>,
}

/// Token-usage accounting returned by the provider, if available.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Usage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

/// Errors that can occur when calling an LLM provider.
#[derive(Debug, thiserror::Error)]
pub enum ProviderError {
    #[error("HTTP transport error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("rate limited: retry after {0}s")]
    RateLimited(u64),

    #[error("API error: {0}")]
    Api(String),

    #[error("provider unavailable: {0}")]
    Unavailable(String),

    #[error("invalid response from provider: {0}")]
    InvalidResponse(String),

    #[error("provider not configured: {0}")]
    NotConfigured(String),
}

impl ProviderError {
    /// Returns true if this error indicates the provider asked us to back off.
    pub fn is_rate_limited(&self) -> bool {
        matches!(self, ProviderError::RateLimited(_))
    }
}

/// Trait for any LLM provider. Implementations: `OpenRouterProvider`,
/// `MockProvider` (for tests), future v2 providers.
#[async_trait]
pub trait Provider: Send + Sync {
    /// Stable identifier for this provider (e.g., `"openrouter"`, `"mock"`).
    fn name(&self) -> &str;

    /// Send a chat request and return the response.
    async fn chat(&self, req: ChatRequest) -> Result<ChatResponse, ProviderError>;

    /// Models the provider supports, for help-text and validation.
    fn available_models(&self) -> Vec<String>;
}

/// OpenRouter-backed provider.
///
/// OpenRouter (https://openrouter.ai) is an LLM aggregator that exposes a
/// unified OpenAI-compatible API. It supports native model-array failover
/// via the `models: []` payload field — when the primary model is saturated
/// or errors, OpenRouter routes to a fallback automatically (per doc §3.2
/// line 104).
pub struct OpenRouterProvider {
    api_key: String,
    base_url: String,
    http: reqwest::Client,
    fallback_models: Vec<String>,
}

impl OpenRouterProvider {
    /// Build a new OpenRouter provider.
    ///
    /// `api_key` is the OpenRouter API key. If empty, `chat()` returns
    /// `ProviderError::NotConfigured` (the bot can still start, but LLM
    /// commands gracefully report the missing config).
    ///
    /// `fallback_models` is the default `models: []` array for the
    /// primary-model-failover behavior. Per-request `models` in
    /// `ChatRequest` overrides this.
    pub fn new(api_key: String, fallback_models: Vec<String>) -> Self {
        let http = reqwest::Client::builder()
            .timeout(HTTP_TIMEOUT)
            .build()
            .expect("reqwest Client::builder should never fail with default config");
        Self {
            api_key,
            base_url: OPENROUTER_API_URL.to_string(),
            http,
            fallback_models,
        }
    }

    /// Override the API base URL (for testing against a mock server).
    pub fn with_base_url(mut self, url: String) -> Self {
        self.base_url = url;
        self
    }
}

#[async_trait]
impl Provider for OpenRouterProvider {
    fn name(&self) -> &str {
        "openrouter"
    }

    async fn chat(&self, req: ChatRequest) -> Result<ChatResponse, ProviderError> {
        if self.api_key.is_empty() {
            return Err(ProviderError::NotConfigured(
                "OPENROUTER_API_KEY is empty".to_string(),
            ));
        }

        // Build the OpenRouter request body.
        // Per OpenRouter API: messages, temperature, max_tokens, models (optional).
        let messages_json: Vec<serde_json::Value> = req
            .messages
            .iter()
            .map(|m| serde_json::json!({ "role": m.role, "content": m.content }))
            .collect();

        let mut body = serde_json::json!({
            "messages": messages_json,
            "temperature": req.temperature,
            "max_tokens": req.max_tokens,
        });

        // Per-request models take priority over the provider-level fallback list.
        let effective_models: &[String] = if !req.models.is_empty() {
            &req.models
        } else {
            &self.fallback_models
        };
        if !effective_models.is_empty() {
            body["models"] = serde_json::json!(effective_models);
        }

        let response = self
            .http
            .post(&self.base_url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .header("HTTP-Referer", "https://github.com/fame0528/savant-bot")
            .header("X-OpenRouter-Title", "savant-bot")
            .json(&body)
            .send()
            .await
            .map_err(ProviderError::Http)?;

        let status = response.status();
        if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
            let retry_after = response
                .headers()
                .get("retry-after")
                .and_then(|v| v.to_str().ok())
                .and_then(|v| v.parse::<u64>().ok())
                .unwrap_or(60);
            return Err(ProviderError::RateLimited(retry_after));
        }

        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(ProviderError::Api(format!("status {}: {}", status, body)));
        }

        let raw: serde_json::Value = response.json().await.map_err(ProviderError::Http)?;

        // Extract the first choice's content.
        let content = raw["choices"][0]["message"]["content"]
            .as_str()
            .ok_or_else(|| {
                ProviderError::InvalidResponse(
                    "missing or non-string choices[0].message.content".to_string(),
                )
            })?
            .to_string();

        let model = raw["model"].as_str().unwrap_or("unknown").to_string();

        let usage = raw.get("usage").map(|u| Usage {
            prompt_tokens: u["prompt_tokens"].as_u64().unwrap_or(0) as u32,
            completion_tokens: u["completion_tokens"].as_u64().unwrap_or(0) as u32,
            total_tokens: u["total_tokens"].as_u64().unwrap_or(0) as u32,
        });

        Ok(ChatResponse {
            content,
            model,
            usage,
        })
    }

    fn available_models(&self) -> Vec<String> {
        self.fallback_models.clone()
    }
}

/// Mock provider for unit tests. Captures all `chat()` calls and returns
/// queued responses. Can be configured to return errors.
pub struct MockProvider {
    responses: Mutex<Vec<ChatResponse>>,
    calls: Mutex<Vec<ChatRequest>>,
    error_to_return: Mutex<Option<ProviderError>>,
}

impl MockProvider {
    pub fn new() -> Self {
        Self {
            responses: Mutex::new(Vec::new()),
            calls: Mutex::new(Vec::new()),
            error_to_return: Mutex::new(None),
        }
    }

    /// Queue a response to be returned by the next `chat()` call.
    pub fn queue_response(&self, resp: ChatResponse) {
        self.responses
            .lock()
            .expect("MockProvider mutex poisoned")
            .push(resp);
    }

    /// Set an error to be returned by the next `chat()` call (one-shot;
    /// the error is consumed on read so subsequent calls can return other
    /// responses or errors).
    pub fn set_error(&self, err: ProviderError) {
        *self
            .error_to_return
            .lock()
            .expect("MockProvider mutex poisoned") = Some(err);
    }

    /// Snapshot of all `chat()` calls made so far.
    pub fn calls(&self) -> Vec<ChatRequest> {
        self.calls
            .lock()
            .expect("MockProvider mutex poisoned")
            .clone()
    }
}

impl Default for MockProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Provider for MockProvider {
    fn name(&self) -> &str {
        "mock"
    }

    async fn chat(&self, req: ChatRequest) -> Result<ChatResponse, ProviderError> {
        self.calls
            .lock()
            .expect("MockProvider mutex poisoned")
            .push(req);

        if let Some(err) = self
            .error_to_return
            .lock()
            .expect("MockProvider mutex poisoned")
            .take()
        {
            return Err(err);
        }

        let mut responses = self.responses.lock().expect("MockProvider mutex poisoned");
        if responses.is_empty() {
            return Err(ProviderError::Unavailable(
                "no mock responses queued".to_string(),
            ));
        }
        Ok(responses.remove(0))
    }

    fn available_models(&self) -> Vec<String> {
        vec!["mock-model".to_string()]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn mock_returns_queued_responses_in_order() {
        let mock = MockProvider::new();
        mock.queue_response(ChatResponse {
            content: "first".to_string(),
            model: "mock".to_string(),
            usage: None,
        });
        mock.queue_response(ChatResponse {
            content: "second".to_string(),
            model: "mock".to_string(),
            usage: None,
        });

        let r1 = mock
            .chat(ChatRequest::new(vec![ChatMessage::user("a")]))
            .await
            .unwrap();
        let r2 = mock
            .chat(ChatRequest::new(vec![ChatMessage::user("b")]))
            .await
            .unwrap();
        assert_eq!(r1.content, "first");
        assert_eq!(r2.content, "second");
        assert_eq!(mock.calls().len(), 2);
    }

    #[tokio::test]
    async fn mock_returns_set_error() {
        let mock = MockProvider::new();
        mock.set_error(ProviderError::RateLimited(42));
        let err = mock.chat(ChatRequest::new(vec![])).await.unwrap_err();
        assert!(err.is_rate_limited());
        assert!(matches!(err, ProviderError::RateLimited(42)));
    }

    #[tokio::test]
    async fn mock_returns_unavailable_when_no_response_queued() {
        let mock = MockProvider::new();
        let err = mock.chat(ChatRequest::new(vec![])).await.unwrap_err();
        assert!(matches!(err, ProviderError::Unavailable(_)));
    }

    #[tokio::test]
    async fn mock_captures_request_payload() {
        let mock = MockProvider::new();
        mock.queue_response(ChatResponse {
            content: "ok".to_string(),
            model: "mock".to_string(),
            usage: None,
        });
        let req = ChatRequest::new(vec![
            ChatMessage::system("you are helpful"),
            ChatMessage::user("hi"),
        ])
        .with_temperature(0.3)
        .with_max_tokens(64)
        .with_fallback_models(vec!["openrouter/auto".to_string()]);
        mock.chat(req).await.unwrap();
        let calls = mock.calls();
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].messages.len(), 2);
        assert_eq!(calls[0].messages[0].role, "system");
        assert_eq!(calls[0].messages[1].role, "user");
        assert!((calls[0].temperature - 0.3).abs() < f32::EPSILON);
        assert_eq!(calls[0].max_tokens, 64);
        assert_eq!(calls[0].models, vec!["openrouter/auto".to_string()]);
    }

    #[test]
    fn chat_message_constructors() {
        assert_eq!(ChatMessage::user("x").role, "user");
        assert_eq!(ChatMessage::system("x").role, "system");
        assert_eq!(ChatMessage::assistant("x").role, "assistant");
    }

    #[test]
    fn chat_request_defaults_and_setters() {
        let r = ChatRequest::new(vec![ChatMessage::user("x")]);
        assert!((r.temperature - 0.7).abs() < f32::EPSILON);
        assert_eq!(r.max_tokens, 1024);
        assert!(r.models.is_empty());

        let r2 = r
            .with_temperature(0.1)
            .with_max_tokens(32)
            .with_fallback_models(vec!["m1".to_string()]);
        assert!((r2.temperature - 0.1).abs() < f32::EPSILON);
        assert_eq!(r2.max_tokens, 32);
        assert_eq!(r2.models, vec!["m1".to_string()]);
    }

    #[test]
    fn provider_error_is_rate_limited() {
        assert!(ProviderError::RateLimited(60).is_rate_limited());
        assert!(!ProviderError::Unavailable("x".to_string()).is_rate_limited());
        assert!(!ProviderError::NotConfigured("x".to_string()).is_rate_limited());
    }

    #[test]
    fn openrouter_provider_reports_not_configured_when_key_empty() {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        let provider = OpenRouterProvider::new(String::new(), vec![]);
        let err = rt
            .block_on(provider.chat(ChatRequest::new(vec![])))
            .unwrap_err();
        assert!(matches!(err, ProviderError::NotConfigured(_)));
    }

    #[test]
    fn openrouter_provider_name() {
        let provider = OpenRouterProvider::new("key".to_string(), vec![]);
        assert_eq!(provider.name(), "openrouter");
    }
}
