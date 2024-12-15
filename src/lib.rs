pub mod config;
pub mod core;
pub mod error;
pub mod health;
pub mod proto;
pub mod reader;
pub mod server;
pub mod storage;

// Re-export commonly used types
pub use config::{Config, ProcessingConfig};
pub use core::EngineCore;
pub use error::{ConfigError, ProcessingError, StorageError};
pub use server::ListenerServer;
pub use reader::SpanReader;  // Add this
pub use storage::S3StorageWriter;  // Add this
