use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub openrouter_api_key: String,
    pub openrouter_model: String,
    pub database_url: String,
    pub backend_host: String,
    pub backend_port: u16,
    pub frontend_origin: String,
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("missing required env var: {0}")]
    Missing(&'static str),
    #[error("invalid env var {0}: {1}")]
    Invalid(&'static str, String),
}

impl Config {
    pub fn from_env() -> Result<Self, ConfigError> {
        let openrouter_api_key = match env::var("OPENROUTER_API_KEY") {
            Ok(v) if !v.trim().is_empty() => v,
            _ => return Err(ConfigError::Missing("OPENROUTER_API_KEY")),
        };

        let openrouter_model = env::var("OPENROUTER_MODEL")
            .unwrap_or_else(|_| "nvidia/nemotron-3-super-120b-a12b:free".to_string());

        let database_url = env::var("DATABASE_URL")
            .unwrap_or_else(|_| "sqlite://gpt-copy-v8.sqlite?mode=rwc".to_string());

        let backend_host = env::var("BACKEND_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());

        let backend_port = match env::var("BACKEND_PORT") {
            Ok(v) => v
                .parse::<u16>()
                .map_err(|e| ConfigError::Invalid("BACKEND_PORT", e.to_string()))?,
            Err(_) => 8080,
        };

        let frontend_origin =
            env::var("FRONTEND_ORIGIN").unwrap_or_else(|_| "http://localhost:3000".to_string());

        Ok(Self {
            openrouter_api_key,
            openrouter_model,
            database_url,
            backend_host,
            backend_port,
            frontend_origin,
        })
    }
}
