use thiserror::Error;

#[derive(Error, Debug)]
pub enum ProcessingError {
    #[error("Invalid message: {0}")]
    InvalidMessage(String),

    #[error("Processing failed: {0}")]
    ProcessingFailed(String),

    #[error("Channel error: {0}")]
    ChannelError(String),
}

#[derive(Error, Debug)]
pub enum StorageError {
    #[error("Write failed: {0}")]
    WriteFailed(String),

    #[error("Batch write failed: {0}")]
    BatchWriteFailed(String),
}
