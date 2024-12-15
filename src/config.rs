use serde::Deserialize;
use std::env;
use std::fs;
use std::path::Path;
use tracing::{info, warn};
use crate::error::ConfigError;

/// Main configuration structure for the storage engine
#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    /// Server-related configuration
    pub server: ServerConfig,
    /// Storage backend configuration
    pub storage: StorageConfig,
    /// Message processing configuration
    pub processing: ProcessingConfig,
    /// Retry policy configuration
    pub retry: RetryConfig,
    /// Metrics collection configuration
    pub metrics: MetricsConfig,
}

/// Server configuration options
#[derive(Debug, Deserialize, Clone)]
pub struct ServerConfig {
    /// Server host address
    pub host: String,
    /// Server port number
    pub port: u16,
    /// Maximum concurrent connections
    #[serde(default = "default_max_connections")]
    pub max_connections: usize,
}

/// Storage backend configuration
#[derive(Debug, Deserialize, Clone)]
pub struct StorageConfig {
    /// Storage bucket name
    pub bucket: String,
    /// Key prefix for stored objects
    pub prefix: String,
    /// Storage region (for cloud storage)
    #[serde(default = "default_region")]
    pub region: String,
}

/// Message processing configuration
#[derive(Debug, Deserialize, Clone)]
pub struct ProcessingConfig {
    /// Number of messages to process in a batch
    pub batch_size: usize,
    /// Maximum time to wait before processing a partial batch
    pub batch_timeout_ms: u64,
}

/// Retry policy configuration
#[derive(Debug, Deserialize, Clone)]
pub struct RetryConfig {
    /// Maximum number of retry attempts
    pub max_retries: u32,
    /// Initial backoff duration in milliseconds
    pub initial_backoff_ms: u64,
    /// Maximum backoff duration in milliseconds
    pub max_backoff_ms: u64,
}

/// Metrics collection configuration
#[derive(Debug, Deserialize, Clone)]
pub struct MetricsConfig {
    /// Whether metrics collection is enabled
    pub enabled: bool,
    /// Interval for pushing metrics in milliseconds
    pub push_interval_ms: u64,
}

impl Config {
    /// Loads configuration from environment or file
    pub fn from_env() -> Result<Self, ConfigError> {
        // Try loading from config file first
        if let Ok(config_path) = env::var("CONFIG_FILE") {
            info!("Loading configuration from file: {}", config_path);
            return Self::from_file(&config_path);
        }

        // Fall back to environment variables
        warn!("No config file specified, using environment variables");
        Self::from_env_vars()
    }

    /// Loads configuration from a YAML file
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, ConfigError> {
        let contents = fs::read_to_string(path)
            .map_err(|e| ConfigError::InvalidFormat(format!("Failed to read config file: {}", e)))?;

        let config: Config = serde_yaml::from_str(&contents)
            .map_err(|e| ConfigError::InvalidFormat(format!("Invalid YAML format: {}", e)))?;

        config.validate()?;
        Ok(config)
    }

    /// Loads configuration from environment variables
    fn from_env_vars() -> Result<Self, ConfigError> {
        let config = Config {
            server: ServerConfig {
                host: env::var("SERVER_HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
                port: env::var("SERVER_PORT")
                    .ok()
                    .and_then(|p| p.parse().ok())
                    .unwrap_or(50051),
                max_connections: default_max_connections(),
            },
            storage: StorageConfig {
                bucket: env::var("STORAGE_BUCKET")
                    .map_err(|_| ConfigError::MissingField("STORAGE_BUCKET".into()))?,
                prefix: env::var("STORAGE_PREFIX").unwrap_or_else(|_| "messages".to_string()),
                region: default_region(),
            },
            processing: ProcessingConfig::default(),
            retry: RetryConfig::default(),
            metrics: MetricsConfig::default(),
        };

        config.validate()?;
        Ok(config)
    }

    /// Validates the configuration values
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

    pub fn new(
        server: ServerConfig,
        storage: StorageConfig,
        processing: ProcessingConfig,
        retry: RetryConfig,
        metrics: MetricsConfig,
    ) -> Result<Self, ConfigError> {
        let config = Self {
            server,
            storage,
            processing,
            retry,
            metrics,
        };
        config.validate()?;
        Ok(config)
    }
}

// Default implementations
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

impl Default for MetricsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            push_interval_ms: 10000,
        }
    }
}

fn default_max_connections() -> usize {
    1000
}

fn default_region() -> String {
    "us-west-2".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[test]
    fn test_config_validation() {
        let config = Config {
            server: ServerConfig {
                host: "localhost".into(),
                port: 8080,
                max_connections: 1000,
            },
            storage: StorageConfig {
                bucket: "test-bucket".into(),
                prefix: "test".into(),
                region: "us-west-2".into(),
            },
            processing: ProcessingConfig {
                batch_size: 0,  // Invalid
                batch_timeout_ms: 1000,
            },
            retry: RetryConfig::default(),
            metrics: MetricsConfig::default(),
        };

        assert!(config.validate().is_err());
    }

    #[test]
    fn test_config_from_file() -> Result<(), Box<dyn std::error::Error>> {
        let config_content = r#"
            server:
              host: "0.0.0.0"
              port: 50051
            storage:
              bucket: "test-bucket"
              prefix: "test"
            processing:
              batch_size: 100
              batch_timeout_ms: 5000
        "#;

        let mut file = NamedTempFile::new()?;
        write!(file, "{}", config_content)?;

        let config = Config::from_file(file.path())?;
        assert_eq!(config.server.port, 50051);
        assert_eq!(config.storage.bucket, "test-bucket");

        Ok(())
    }
}
