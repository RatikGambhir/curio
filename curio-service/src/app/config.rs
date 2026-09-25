use std::{env, fmt, path::PathBuf};

use crate::adapters::postgres::client::validate_schema_name;

const DEFAULT_MAX_CONNECTIONS: u32 = 10;
const DEFAULT_ACQUIRE_TIMEOUT_SECONDS: u64 = 10;
const DEFAULT_EMBEDDING_MODEL: &str = "text-embedding-3-small";
const DEFAULT_IMAGE_DESCRIPTION_MODEL: &str = "gpt-5.5";
const DEFAULT_DOCUMENT_CONCURRENCY: u32 = 8;
const DEFAULT_COMPLETED_JOB_RETENTION_SECONDS: u64 = 10 * 60;
const OFFICE_EXECUTABLE_CANDIDATES: [&str; 6] = [
    "/Applications/LibreOffice.app/Contents/MacOS/soffice",
    "/Applications/LibreOfficeDev.app/Contents/MacOS/soffice",
    "/opt/homebrew/bin/soffice",
    "/usr/local/bin/soffice",
    "/usr/bin/soffice",
    "/usr/bin/libreoffice",
];

/// Documents ingestion, embedding, and preview settings.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DocumentsConfig {
    pub embedding_model: String,
    pub image_description_model: String,
    pub max_concurrent_documents: usize,
    pub completed_job_retention_seconds: u64,
    /// LibreOffice executable used for Office previews; DOCX falls back to a
    /// text-rendered PDF when it is absent.
    pub office_executable: Option<PathBuf>,
}

impl Default for DocumentsConfig {
    fn default() -> Self {
        Self {
            embedding_model: DEFAULT_EMBEDDING_MODEL.to_owned(),
            image_description_model: DEFAULT_IMAGE_DESCRIPTION_MODEL.to_owned(),
            max_concurrent_documents: DEFAULT_DOCUMENT_CONCURRENCY as usize,
            completed_job_retention_seconds: DEFAULT_COMPLETED_JOB_RETENTION_SECONDS,
            office_executable: None,
        }
    }
}

impl DocumentsConfig {
    pub fn from_env() -> Result<Self, ConfigError> {
        Ok(Self {
            embedding_model: optional("OPENAI_EMBEDDING_MODEL")
                .unwrap_or_else(|| DEFAULT_EMBEDDING_MODEL.to_owned()),
            image_description_model: optional("OPENAI_IMAGE_DESCRIPTION_MODEL")
                .unwrap_or_else(|| DEFAULT_IMAGE_DESCRIPTION_MODEL.to_owned()),
            max_concurrent_documents: bounded_u32(
                "CURIO_DOCUMENT_CONCURRENCY",
                DEFAULT_DOCUMENT_CONCURRENCY,
                1,
                32,
            )? as usize,
            completed_job_retention_seconds: bounded_u64(
                "CURIO_DOCUMENT_JOB_RETENTION_SECONDS",
                DEFAULT_COMPLETED_JOB_RETENTION_SECONDS,
                0,
                24 * 60 * 60,
            )?,
            office_executable: resolve_office_executable(),
        })
    }
}

fn resolve_office_executable() -> Option<PathBuf> {
    if let Some(configured) = optional("CURIO_SOFFICE") {
        return Some(PathBuf::from(configured));
    }
    env::var_os("PATH")
        .and_then(|path| {
            env::split_paths(&path)
                .flat_map(|directory| [directory.join("soffice"), directory.join("libreoffice")])
                .find(|candidate| candidate.is_file())
        })
        .or_else(|| {
            OFFICE_EXECUTABLE_CANDIDATES
                .into_iter()
                .map(PathBuf::from)
                .find(|candidate| candidate.is_file())
        })
}

fn optional(variable: &'static str) -> Option<String> {
    env::var(variable)
        .ok()
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty())
}

#[derive(Clone, PartialEq, Eq)]
pub struct ServiceConfig {
    pub openai_api_key: String,
    pub openai_model: String,
    pub openai_base_url: String,
    pub database_url: String,
    pub database_schema: String,
    pub database_max_connections: u32,
    pub database_acquire_timeout_seconds: u64,
    pub cors_allowed_origins: Vec<String>,
    pub documents: DocumentsConfig,
}

impl fmt::Debug for ServiceConfig {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ServiceConfig")
            .field("openai_api_key", &"[REDACTED]")
            .field("openai_model", &self.openai_model)
            .field("openai_base_url", &self.openai_base_url)
            .field("database_url", &"[REDACTED]")
            .field("database_schema", &self.database_schema)
            .field("database_max_connections", &self.database_max_connections)
            .field(
                "database_acquire_timeout_seconds",
                &self.database_acquire_timeout_seconds,
            )
            .field("cors_allowed_origins", &self.cors_allowed_origins)
            .field("documents", &self.documents)
            .finish()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConfigError {
    variable: &'static str,
    reason: &'static str,
}

impl ConfigError {
    fn missing(variable: &'static str) -> Self {
        Self {
            variable,
            reason: "is not set",
        }
    }

    fn invalid(variable: &'static str, reason: &'static str) -> Self {
        Self { variable, reason }
    }
}

impl fmt::Display for ConfigError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "environment variable {} {}",
            self.variable, self.reason
        )
    }
}

impl std::error::Error for ConfigError {}

impl ServiceConfig {
    pub fn from_env() -> Result<Self, ConfigError> {
        let database_schema = required("CURIO_DB_SCHEMA")?;
        if !validate_schema_name(&database_schema) {
            return Err(ConfigError::invalid(
                "CURIO_DB_SCHEMA",
                "must start with a lowercase letter, contain only lowercase letters, numbers, or underscores, and be at most 63 characters",
            ));
        }

        Ok(Self {
            openai_api_key: required("OPENAI_API_KEY")?,
            openai_model: required("OPENAI_MODEL")?,
            openai_base_url: env::var("OPENAI_BASE_URL")
                .unwrap_or_else(|_| "https://api.openai.com".to_owned()),
            database_url: required("DATABASE_URL")?,
            database_schema,
            database_max_connections: bounded_u32(
                "CURIO_DB_MAX_CONNECTIONS",
                DEFAULT_MAX_CONNECTIONS,
                1,
                50,
            )?,
            database_acquire_timeout_seconds: bounded_u64(
                "CURIO_DB_ACQUIRE_TIMEOUT_SECONDS",
                DEFAULT_ACQUIRE_TIMEOUT_SECONDS,
                1,
                60,
            )?,
            cors_allowed_origins: env::var("CURIO_CORS_ALLOWED_ORIGINS")
                .unwrap_or_else(|_| {
                    "http://localhost:5173,http://127.0.0.1:5173,http://localhost:1420,http://127.0.0.1:1420"
                        .to_owned()
                })
                .split(',')
                .map(str::trim)
                .filter(|origin| !origin.is_empty())
                .map(str::to_owned)
                .collect(),
            documents: DocumentsConfig::from_env()?,
        })
    }
}

fn required(variable: &'static str) -> Result<String, ConfigError> {
    env::var(variable)
        .ok()
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| ConfigError::missing(variable))
}

fn bounded_u32(
    variable: &'static str,
    default: u32,
    minimum: u32,
    maximum: u32,
) -> Result<u32, ConfigError> {
    env::var(variable)
        .ok()
        .map(|value| value.parse::<u32>().ok())
        .unwrap_or(Some(default))
        .filter(|value| (minimum..=maximum).contains(value))
        .ok_or_else(|| ConfigError::invalid(variable, "is not a valid bounded integer"))
}

fn bounded_u64(
    variable: &'static str,
    default: u64,
    minimum: u64,
    maximum: u64,
) -> Result<u64, ConfigError> {
    env::var(variable)
        .ok()
        .map(|value| value.parse::<u64>().ok())
        .unwrap_or(Some(default))
        .filter(|value| (minimum..=maximum).contains(value))
        .ok_or_else(|| ConfigError::invalid(variable, "is not a valid bounded integer"))
}

#[cfg(test)]
mod tests {
    use super::{ServiceConfig, validate_schema_name};

    #[test]
    fn schema_names_are_conservative_identifiers() {
        for valid in ["curio_dev", "curio_prod", "curio_test_123", "public"] {
            assert!(validate_schema_name(valid));
        }

        for invalid in ["", "Curio", "1curio", "curio-test", "curio dev"] {
            assert!(!validate_schema_name(invalid));
        }
    }

    #[test]
    fn debug_output_redacts_credentials() {
        let config = ServiceConfig {
            openai_api_key: "openai-secret-value".to_owned(),
            openai_model: "test-model".to_owned(),
            openai_base_url: "https://api.openai.com".to_owned(),
            database_url: "postgresql://user:database-secret@db/curio".to_owned(),
            database_schema: "curio_dev".to_owned(),
            database_max_connections: 5,
            database_acquire_timeout_seconds: 10,
            cors_allowed_origins: vec!["http://localhost:5173".to_owned()],
            documents: super::DocumentsConfig::default(),
        };

        let output = format!("{config:?}");
        assert!(!output.contains("openai-secret-value"));
        assert!(!output.contains("database-secret"));
        assert!(output.contains("[REDACTED]"));
        assert!(output.contains("curio_dev"));
    }
}
