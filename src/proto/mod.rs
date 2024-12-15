/*
    Protocol buffer module organization
    Provides type definitions and service interfaces
    Separates client and server concerns
*/

// Only need one include for the generated protobuf types
pub mod storage_engine {
    tonic::include_proto!("storage_engine");
}

// Common types used by both client and server
pub mod common {
    pub use super::storage_engine::{Message, ProcessResponse};
}

// Types used by the server implementation
pub mod server {
    pub use super::common::*;
    pub use super::storage_engine::storage_engine_server::{StorageEngine, StorageEngineServer};
}

#[cfg(feature = "client")]
pub mod client {
    pub use super::common::*;
    pub use super::storage_engine::storage_engine_client::StorageEngineClient;
}
