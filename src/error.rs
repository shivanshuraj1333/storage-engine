use thiserror::Error;

/// Errors that can occur during message processing
#[derive(Error, Debug)]
pub enum ProcessingError {
    /// Error during message validation
    #[error("Validation error: {0}")]
    ValidationError(String),

    /// Error during message processing
    #[error("Processing failed: {0}")]
    ProcessingFailed(String),

    /// Error during storage operations
    #[error("Storage error: {0}")]
    StorageError(String),

    /// Rate limit exceeded for processing
    #[error("Rate limit exceeded: {0}")]
    RateLimitExceeded(String),

    /// Requested resource not found
    #[error("Resource not found: {0}")]
    NotFound(String),

    #[error("Shutdown error: {0}")]
    ShutdownError(String),
}

/// Errors that can occur during storage operations
#[derive(Error, Debug)]
pub enum StorageError {
    /// Error during write operation
    #[error("Write failed: {0}")]
    WriteFailed(String),

    /// Error connecting to storage
    #[error("Connection failed: {0}")]
    ConnectionError(String),

    /// Error during batch write operation
    #[error("Batch write failed: {0}")]
    BatchWriteFailed(String),

    /// Error in storage configuration
    #[error("Configuration error: {0}")]
    ConfigError(String),

    /// Maximum retry attempts exceeded
    #[error("Retry limit exceeded: {0}")]
    RetryLimitExceeded(String),

    /// Error during read operation
    #[error("Read failed: {0}")]
    ReadFailed(String),
}

/// Errors that can occur during configuration
#[derive(Error, Debug)]
pub enum ConfigError {
    /// Invalid configuration value
    #[error("Invalid value: {0}")]
    InvalidValue(String),

    /// Missing required configuration field
    #[error("Missing field: {0}")]
    MissingField(String),

    /// Invalid configuration format
    #[error("Invalid format: {0}")]
    InvalidFormat(String),
}

// Convert StorageError to ProcessingError
impl From<StorageError> for ProcessingError {
    fn from(error: StorageError) -> Self {
        match error {
            StorageError::WriteFailed(msg) => 
                ProcessingError::StorageError(format!("Write failed: {}", msg)),
            StorageError::ConnectionError(msg) => 
                ProcessingError::StorageError(format!("Connection failed: {}", msg)),
            StorageError::BatchWriteFailed(msg) => 
                ProcessingError::StorageError(format!("Batch write failed: {}", msg)),
            StorageError::ConfigError(msg) => 
                ProcessingError::StorageError(format!("Config error: {}", msg)),
            StorageError::RetryLimitExceeded(msg) => 
                ProcessingError::StorageError(format!("Retry limit exceeded: {}", msg)),
            StorageError::ReadFailed(msg) => 
                ProcessingError::StorageError(format!("Read failed: {}", msg)),
        }
    }
}

impl From<tokio::time::error::Error> for ProcessingError {
    fn from(err: tokio::time::error::Error) -> Self {
        ProcessingError::ShutdownError(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_storage_error_conversion() {
        let storage_error = StorageError::WriteFailed("test error".to_string());
        let processing_error = ProcessingError::from(storage_error);
        
        match processing_error {
            ProcessingError::StorageError(msg) => {
                assert!(msg.contains("Write failed"));
                assert!(msg.contains("test error"));
            }
            _ => panic!("Wrong error type after conversion"),
        }
    }

    #[test]
    fn test_error_display() {
        let error = ProcessingError::ValidationError("invalid input".to_string());
        assert_eq!(
            error.to_string(),
            "Validation error: invalid input"
        );
    }
}
