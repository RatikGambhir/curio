use std::{env, fmt};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ServiceConfig {
    pub openai_api_key: String,
    pub openai_model: String,
    pub openai_base_url: String,
    pub database_url: String,
    pub cors_allowed_origins: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConfigError {
    variable: &'static str,
}

impl ConfigError {
    fn missing(variable: &'static str) -> Self {
        Self { variable }
    }
}

impl fmt::Display for ConfigError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "required environment variable {} is not set",
            self.variable
        )
    }
}

impl std::error::Error for ConfigError {}

impl ServiceConfig {
    pub fn from_env() -> Result<Self, ConfigError> {
        Ok(Self {
            openai_api_key: required("OPENAI_API_KEY")?,
            openai_model: required("OPENAI_MODEL")?,
            openai_base_url: env::var("OPENAI_BASE_URL")
                .unwrap_or_else(|_| "https://api.openai.com".to_owned()),
            database_url: env::var("CURIO_DATABASE_URL")
                .unwrap_or_else(|_| "sqlite://curio.db".to_owned()),
            cors_allowed_origins: env::var("CURIO_CORS_ALLOWED_ORIGINS")
                .unwrap_or_else(|_| "http://localhost:5173,http://127.0.0.1:5173".to_owned())
                .split(',')
                .map(str::trim)
                .filter(|origin| !origin.is_empty())
                .map(str::to_owned)
                .collect(),
        })
    }
}

fn required(variable: &'static str) -> Result<String, ConfigError> {
    env::var(variable)
        .ok()
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| ConfigError::missing(variable))
}
