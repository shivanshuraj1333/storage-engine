use async_trait::async_trait;
use crate::error::StorageError;

#[async_trait]
pub trait StorageWriter {
    async fn write(&self, data: &[u8]) -> Result<(), StorageError>;
    async fn write_batch(&self, batch: Vec<&[u8]>) -> Result<(), StorageError>;
}

#[allow(dead_code)]
pub struct CloudStorageWriter {
    bucket: String,
    prefix: String,
}

impl CloudStorageWriter {
    pub fn new(bucket: String, prefix: String) -> Self {
        Self { bucket, prefix }
    }
}

#[async_trait]
impl StorageWriter for CloudStorageWriter {
    async fn write(&self, _data: &[u8]) -> Result<(), StorageError> {
        // TODO: Implement cloud storage writing logic
        Ok(())
    }

    async fn write_batch(&self, _batch: Vec<&[u8]>) -> Result<(), StorageError> {
        // TODO: Implement batch writing
        Ok(())
    }
}