use thiserror::Error;

#[derive(Error, Debug)]
pub enum ProcessingError {
    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("Processing failed: {0}")]
    ProcessingFailed(String),

    #[error("Storage error: {0}")]
    StorageError(String),
}

#[derive(Error, Debug)]
pub enum StorageError {
    #[error("Write failed: {0}")]
    WriteFailed(String),

    #[error("Batch write failed: {0}")]
    BatchWriteFailed(String),
}
