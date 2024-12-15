//! Example demonstrating how to implement a custom storage backend
use storage_engine::storage::StorageWriter;

#[derive(Debug)]
struct CustomStorage {
    // Custom implementation details
}

#[async_trait]
impl StorageWriter for CustomStorage {
    async fn write(&self, key: &str, data: &[u8]) -> Result<(), StorageError> {
        // Custom implementation
    }
} 