use serde::Deserialize;
use crate::error::ConfigError;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub server: ServerConfig,
    pub storage: StorageConfig,
    pub processing: ProcessingConfig,
    pub retry: RetryConfig,
    pub metrics: MetricsConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Deserialize, Clone)]
pub struct StorageConfig {
    pub bucket: String,
    pub prefix: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ProcessingConfig {
    pub batch_size: usize,
    pub batch_timeout_ms: u64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct RetryConfig {
    pub max_retries: u32,
    pub initial_backoff_ms: u64,
    pub max_backoff_ms: u64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct MetricsConfig {
    pub enabled: bool,
    pub push_interval_ms: u64,
}

impl Config {
    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.processing.batch_size == 0 {
            return Err(ConfigError::InvalidValue("batch_size must be > 0".into()));
        }
        if self.retry.max_retries == 0 {
            return Err(ConfigError::InvalidValue("max_retries must be > 0".into()));
        }
        if self.retry.max_backoff_ms < self.retry.initial_backoff_ms {
            return Err(ConfigError::InvalidValue(
                "max_backoff_ms must be >= initial_backoff_ms".into()
            ));
        }
        Ok(())
    }

    pub fn from_env() -> Result<Self, ConfigError> {
        // Load from environment or config file
        todo!()
    }
}

// Add Default implementations
impl Default for ProcessingConfig {
    fn default() -> Self {
        Self {
            batch_size: 100,
            batch_timeout_ms: 5000,
        }
    }
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            initial_backoff_ms: 100,
            max_backoff_ms: 1000,
        }
    }
}
