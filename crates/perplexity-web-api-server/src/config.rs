use perplexity_web_client::{ReasonModel, SearchModel};
use std::time::Duration;

pub struct Config {
    pub session_token: String,
    pub csrf_token: String,
    pub api_key: Option<String>,
    pub search_model: Option<SearchModel>,
    pub reason_model: Option<ReasonModel>,
    pub host: String,
    pub port: u16,
    pub search_timeout: Duration,
    pub reason_timeout: Duration,
    pub research_timeout: Duration,
}

impl Config {
    pub fn from_env() -> Result<Self, ConfigError> {
        let session_token = require_env("PERPLEXITY_SESSION_TOKEN")?;
        let csrf_token = require_env("PERPLEXITY_CSRF_TOKEN")?;
        let api_key = optional_env("PERPLEXITY_API_KEY")?;
        let search_model = optional_model_env::<SearchModel>("PERPLEXITY_SEARCH_MODEL")?;
        let reason_model = optional_model_env::<ReasonModel>("PERPLEXITY_REASON_MODEL")?;
        let host = optional_env("HOST")?.unwrap_or_else(|| "127.0.0.1".to_string());
        let port = optional_env("PORT")?
            .map(|s| {
                s.parse::<u16>().map_err(|_| {
                    ConfigError::Invalid("PORT".into(), "must be a valid port number".into())
                })
            })
            .transpose()?
            .unwrap_or(3000);
        let search_timeout = parse_timeout_env("SEARCH_TIMEOUT_SECS", 30)?;
        let reason_timeout = parse_timeout_env("REASON_TIMEOUT_SECS", 120)?;
        let research_timeout = parse_timeout_env("RESEARCH_TIMEOUT_SECS", 300)?;

        Ok(Self {
            session_token,
            csrf_token,
            api_key,
            search_model,
            reason_model,
            host,
            port,
            search_timeout,
            reason_timeout,
            research_timeout,
        })
    }

    pub fn bind_addr(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("{0} is missing")]
    Missing(String),
    #[error("{0} is invalid: {1}")]
    Invalid(String, String),
}

fn require_env(name: &str) -> Result<String, ConfigError> {
    std::env::var(name)
        .map_err(|_| ConfigError::Missing(name.into()))
        .and_then(|v| {
            if v.trim().is_empty() {
                Err(ConfigError::Missing(name.into()))
            } else {
                Ok(v)
            }
        })
}

fn optional_env(name: &str) -> Result<Option<String>, ConfigError> {
    match std::env::var(name) {
        Ok(v) if v.trim().is_empty() => Ok(None),
        Ok(v) => Ok(Some(v)),
        Err(std::env::VarError::NotPresent) => Ok(None),
        Err(std::env::VarError::NotUnicode(_)) => Err(ConfigError::Invalid(
            name.into(),
            "must be valid UTF-8".into(),
        )),
    }
}

fn optional_model_env<T>(name: &str) -> Result<Option<T>, ConfigError>
where
    T: std::str::FromStr,
    T::Err: std::fmt::Display,
{
    let Some(value) = optional_env(name)? else {
        return Ok(None);
    };
    value
        .parse::<T>()
        .map(Some)
        .map_err(|e| ConfigError::Invalid(name.into(), e.to_string()))
}

fn parse_timeout_env(name: &str, default_secs: u64) -> Result<Duration, ConfigError> {
    let secs = optional_env(name)?
        .map(|s| {
            s.parse::<u64>()
                .map_err(|_| ConfigError::Invalid(name.into(), "must be a positive integer".into()))
        })
        .transpose()?
        .unwrap_or(default_secs);
    Ok(Duration::from_secs(secs))
}
