use anyhow::{Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::config::{Config, Provider};
use crate::conversation::Message;

// ── Anthropic (Claude) types ───────────────────────────────────────────────────

#[derive(Serialize)]
struct AnthropicRequest<'a> {
    model: &'a str,
    max_tokens: u32,
    system: &'a str,
    messages: &'a Vec<Message>,
}

#[derive(Deserialize)]
struct AnthropicResponse {
    content: Vec<AnthropicBlock>,
}

#[derive(Deserialize)]
struct AnthropicBlock {
    #[serde(rename = "type")]
    block_type: String,
    text: Option<String>,
}

// ── OpenAI-compatible types (OpenAI + Ollama) ─────────────────────────────────

#[derive(Serialize)]
struct OpenAIRequest<'a> {
    model: &'a str,
    max_tokens: u32,
    messages: Vec<OpenAIMessage<'a>>,
}

#[derive(Serialize)]
struct OpenAIMessage<'a> {
    role: &'a str,
    content: &'a str,
}

#[derive(Deserialize)]
struct OpenAIResponse {
    choices: Vec<OpenAIChoice>,
}

#[derive(Deserialize)]
struct OpenAIChoice {
    message: OpenAIChoiceMessage,
}

#[derive(Deserialize)]
struct OpenAIChoiceMessage {
    content: String,
}

// ── Shared error type ──────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct ErrorResponse {
    error: ApiError,
}

#[derive(Deserialize)]
struct ApiError {
    message: String,
}

// ── Main client ────────────────────────────────────────────────────────────────

#[derive(Clone)]
pub struct ChatClient {
    http: Client,
    config: Config,
}

impl ChatClient {
    pub fn new(config: Config) -> Result<Self> {
        let http = Client::builder()
            .build()
            .context("Failed to build HTTP client")?;
        Ok(ChatClient { http, config })
    }

    pub async fn send(&self, messages: &Vec<Message>) -> Result<String> {
        match self.config.provider {
            Provider::Claude => self.send_anthropic(messages).await,
            Provider::OpenAI | Provider::Ollama => self.send_openai(messages).await,
        }
    }

    // ── Anthropic native API (/v1/messages) ────────────────────────────────────

    async fn send_anthropic(&self, messages: &Vec<Message>) -> Result<String> {
        let url = format!("{}/v1/messages", self.config.base_url);

        let body = AnthropicRequest {
            model: &self.config.model,
            max_tokens: self.config.max_tokens,
            system: &self.config.system_prompt,
            messages,
        };

        let response = self
            .http
            .post(&url)
            .header("x-api-key", &self.config.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&body)
            .send()
            .await
            .context("Network request failed")?;

        let status = response.status();
        let text = response.text().await.context("Failed to read response")?;

        if !status.is_success() {
            if let Ok(e) = serde_json::from_str::<ErrorResponse>(&text) {
                anyhow::bail!("API error {}: {}", status, e.error.message);
            }
            anyhow::bail!("API error {}: {}", status, text);
        }

        let parsed: AnthropicResponse =
            serde_json::from_str(&text).context("Failed to parse Anthropic response")?;

        parsed
            .content
            .into_iter()
            .find(|b| b.block_type == "text")
            .and_then(|b| b.text)
            .context("No text content in response")
    }

    // ── OpenAI-compatible API (/v1/chat/completions) ───────────────────────────
    // Used for both OpenAI and Ollama

    async fn send_openai(&self, messages: &Vec<Message>) -> Result<String> {
        let url = format!("{}/v1/chat/completions", self.config.base_url);

        // Prepend system prompt as a system message
        let mut oai_messages: Vec<OpenAIMessage> = vec![OpenAIMessage {
            role: "system",
            content: &self.config.system_prompt,
        }];
        for m in messages {
            oai_messages.push(OpenAIMessage {
                role: &m.role,
                content: &m.content,
            });
        }

        let body = OpenAIRequest {
            model: &self.config.model,
            max_tokens: self.config.max_tokens,
            messages: oai_messages,
        };

        let mut req = self
            .http
            .post(&url)
            .header("content-type", "application/json");

        // Ollama doesn't need an auth header; OpenAI does
        if !self.config.api_key.is_empty() {
            req = req.header("Authorization", format!("Bearer {}", self.config.api_key));
        }

        let response = req
            .json(&body)
            .send()
            .await
            .context("Network request failed")?;

        let status = response.status();
        let text = response.text().await.context("Failed to read response")?;

        if !status.is_success() {
            if let Ok(e) = serde_json::from_str::<ErrorResponse>(&text) {
                anyhow::bail!("API error {}: {}", status, e.error.message);
            }
            anyhow::bail!("API error {}: {}", status, text);
        }

        let parsed: OpenAIResponse =
            serde_json::from_str(&text).context("Failed to parse OpenAI response")?;

        parsed
            .choices
            .into_iter()
            .next()
            .map(|c| c.message.content)
            .context("No choices in response")
    }
}
