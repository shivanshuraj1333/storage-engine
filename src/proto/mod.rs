/*
    Protocol buffer module organization
    Provides type definitions and service interfaces
    Separates client and server concerns
*/

// Include the generated proto code
pub mod storage_engine {
    tonic::include_proto!("storage_engine");
}

// Re-export commonly used types
pub use storage_engine::{Message, ProcessResponse};

// Re-export server types
pub use storage_engine::storage_engine_server::{StorageEngine, StorageEngineServer};

// Re-export client types when client feature is enabled
#[cfg(feature = "client")]
pub use storage_engine::storage_engine_client::StorageEngineClient;
