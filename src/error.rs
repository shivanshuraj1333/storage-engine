use thiserror::Error;

#[derive(Error, Debug)]
pub enum ProcessingError {
    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("Processing failed: {0}")]
    ProcessingFailed(String),

    #[error("Storage error: {0}")]
    StorageError(String),

    #[error("Rate limit exceeded: {0}")]
    RateLimitExceeded(String),

    #[error("Resource not found: {0}")]
    NotFound(String),
}

#[derive(Error, Debug)]
pub enum StorageError {
    #[error("Write failed: {0}")]
    WriteFailed(String),

    #[error("Connection failed: {0}")]
    ConnectionError(String),

    #[error("Batch write failed: {0}")]
    BatchWriteFailed(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Retry limit exceeded: {0}")]
    RetryLimitExceeded(String),
}

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Invalid value: {0}")]
    InvalidValue(String),

    #[error("Missing field: {0}")]
    MissingField(String),

    #[error("Invalid format: {0}")]
    InvalidFormat(String),
}

// Add conversion from StorageError to ProcessingError
impl From<StorageError> for ProcessingError {
    fn from(error: StorageError) -> Self {
        match error {
            StorageError::WriteFailed(msg) => ProcessingError::StorageError(format!("Write failed: {}", msg)),
            StorageError::ConnectionError(msg) => ProcessingError::StorageError(format!("Connection failed: {}", msg)),
            StorageError::BatchWriteFailed(msg) => ProcessingError::StorageError(format!("Batch write failed: {}", msg)),
            StorageError::ConfigError(msg) => ProcessingError::StorageError(format!("Config error: {}", msg)),
            StorageError::RetryLimitExceeded(msg) => ProcessingError::StorageError(format!("Retry limit exceeded: {}", msg)),
        }
    }
}
