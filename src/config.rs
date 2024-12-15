use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub storage: StorageConfig,
    pub processing: ProcessingConfig,
}

#[derive(Debug, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Deserialize)]
pub struct StorageConfig {
    pub bucket: String,
    pub prefix: String,
}

#[derive(Debug, Deserialize)]
pub struct ProcessingConfig {
    pub batch_size: usize,
    pub batch_timeout_ms: u64,
}
