use anyhow::{Context, Result};

#[derive(Debug, Clone, PartialEq)]
pub enum Provider {
    Claude,
    OpenAI,
    Ollama,
}

impl Provider {
    pub fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "claude" | "anthropic" => Ok(Provider::Claude),
            "openai"               => Ok(Provider::OpenAI),
            "ollama"               => Ok(Provider::Ollama),
            other => anyhow::bail!(
                "Unknown provider '{}'. Choose: claude, openai, ollama", other
            ),
        }
    }

    pub fn label(&self) -> &str {
        match self {
            Provider::Claude => "Claude (Anthropic)",
            Provider::OpenAI => "OpenAI",
            Provider::Ollama => "Ollama (local)",
        }
    }
}

#[derive(Clone)]
pub struct Config {
    pub provider: Provider,
    pub api_key: String,
    pub model: String,
    pub max_tokens: u32,
    pub system_prompt: String,
    pub base_url: String,
}

impl Config {
    pub fn load() -> Result<Self> {
        dotenv::dotenv().ok();

        // PROVIDER — defaults to "claude"
        let provider_str = std::env::var("PROVIDER").unwrap_or_else(|_| "claude".to_string());
        let provider = Provider::from_str(&provider_str)?;

        // API key (not required for Ollama)
        let api_key = match provider {
            Provider::Ollama => std::env::var("API_KEY").unwrap_or_default(),
            Provider::Claude => std::env::var("ANTHROPIC_API_KEY")
                .or_else(|_| std::env::var("API_KEY"))
                .context("Set ANTHROPIC_API_KEY in .env for Claude")?,
            Provider::OpenAI => std::env::var("OPENAI_API_KEY")
                .or_else(|_| std::env::var("API_KEY"))
                .context("Set OPENAI_API_KEY in .env for OpenAI")?,
        };

        // Model defaults per provider
        let default_model = match provider {
            Provider::Claude => "claude-sonnet-4-20250514",
            Provider::OpenAI => "gpt-4o",
            Provider::Ollama => "llama3.2",
        };
        let model = std::env::var("MODEL").unwrap_or_else(|_| default_model.to_string());

        // Base URL defaults per provider
        let default_url = match provider {
            Provider::Claude => "https://api.anthropic.com".to_string(),
            Provider::OpenAI => "https://api.openai.com".to_string(),
            Provider::Ollama => "http://localhost:11434".to_string(),
        };
        let base_url = std::env::var("BASE_URL").unwrap_or(default_url);

        let max_tokens: u32 = std::env::var("MAX_TOKENS")
            .unwrap_or_else(|_| "1024".to_string())
            .parse()
            .context("MAX_TOKENS must be a number")?;

        let system_prompt = std::env::var("SYSTEM_PROMPT")
            .unwrap_or_else(|_| "You are a helpful, concise assistant.".to_string());

        Ok(Config {
            provider,
            api_key,
            model,
            max_tokens,
            system_prompt,
            base_url,
        })
    }

    pub fn from_runtime(
        provider: Provider,
        api_key: String,
        model: String,
        max_tokens: u32,
        system_prompt: String,
        base_url: Option<String>,
    ) -> Self {
        let default_url = match provider {
            Provider::Claude => "https://api.anthropic.com",
            Provider::OpenAI => "https://api.openai.com",
            Provider::Ollama => "http://localhost:11434",
        };

        Config {
            provider,
            api_key,
            model,
            max_tokens,
            system_prompt,
            base_url: base_url.unwrap_or_else(|| default_url.to_string()),
        }
    }
}
