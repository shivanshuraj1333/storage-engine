use aws_sdk_s3::Client as S3Client;
use aws_sdk_s3::config::{Credentials, Region};
use aws_sdk_s3::config::Builder as S3Builder;
use tracing::{info, error};
use opentelemetry::sdk::export::trace::SpanData;
use serde::{Deserialize, Serialize};
use serde_json::{self, json};
use std::time::{SystemTime, UNIX_EPOCH, Duration};
use std::convert::TryInto;

use crate::error::StorageError;
use async_trait::async_trait;
use crate::health::HealthStatus;

/// Trait defining storage operations for the engine.
/// Implementations should handle data persistence and retrieval.
#[async_trait]
pub trait StorageWriter {
    /// Writes a single data entry with the given key
    async fn write(&self, key: &str, data: &[u8]) -> Result<(), StorageError>;
    
    /// Writes multiple data entries in batch
    async fn write_batch(&self, entries: Vec<(&str, &[u8])>) -> Result<(), StorageError>;
    
    /// Ensures all pending writes are persisted
    async fn flush(&self) -> Result<(), StorageError>;
    
    /// Writes a collection of spans to storage
    async fn write_spans(&self, spans: Vec<SpanData>) -> Result<(), StorageError>;
}

/// Represents a stored span with serializable fields
#[derive(Debug, Serialize, Deserialize)]
pub struct StoredSpan {
    /// Unique identifier for the trace this span belongs to
    pub trace_id: String,
    /// Unique identifier for this span
    pub span_id: String,
    /// Name of the operation this span represents
    pub name: String,
    /// Type of span (client, server, etc.)
    pub kind: String,
    /// Start time in nanoseconds since epoch
    pub start_time: u64,
    /// End time in nanoseconds since epoch
    pub end_time: u64,
    /// Status of the operation (success, error, etc.)
    pub status: String,
}

/// Represents a span entry in storage with metadata
#[derive(Debug)]
pub struct SpanEntry {
    /// Storage key for the span
    pub key: String,
    /// Last modification time of the span data
    pub last_modified: SystemTime,
}

/// S3-compatible storage implementation
pub struct S3StorageWriter {
    /// S3 client for storage operations
    client: S3Client,
    /// Target bucket name
    bucket: String,
    /// Key prefix for all stored objects
    prefix: String,
}

impl S3StorageWriter {
    /// Creates a new S3StorageWriter instance
    pub async fn new(bucket: String, prefix: String) -> Result<Self, StorageError> {
        info!("Initializing S3 storage writer for bucket: {}", bucket);
        
        let client = Self::create_s3_client().await?;
        Self::verify_bucket_access(&client, &bucket).await?;

        Ok(Self { client, bucket, prefix })
    }

    /// Creates and configures an S3 client
    async fn create_s3_client() -> Result<S3Client, StorageError> {
        let credentials = Credentials::new(
            "test", 
            "test", 
            None, 
            None, 
            "dummy"
        );

        let config = aws_config::defaults(aws_config::BehaviorVersion::latest())
            .credentials_provider(credentials)
            .endpoint_url("http://localhost:4566")
            .region(Region::new("us-east-1"))
            .load()
            .await;

        let s3_config = S3Builder::from(&config)
            .force_path_style(true)
            .build();

        Ok(S3Client::from_conf(s3_config))
    }

    /// Verifies access to the target bucket
    async fn verify_bucket_access(client: &S3Client, bucket: &str) -> Result<(), StorageError> {
        match client.head_bucket().bucket(bucket).send().await {
            Ok(_) => {
                info!("Successfully connected to bucket: {}", bucket);
                Ok(())
            }
            Err(e) => {
                error!("Failed to connect to bucket {}: {}", bucket, e);
                Err(StorageError::ConnectionError(format!("Bucket verification failed: {}", e)))
            }
        }
    }

    /// Constructs a full storage key with prefix
    fn get_full_key(&self, key: &str) -> String {
        if self.prefix.is_empty() {
            key.to_string()
        } else {
            format!("{}/{}", self.prefix.trim_end_matches('/'), key)
        }
    }

    /// Lists spans in storage with pagination
    pub async fn list_spans(&self, limit: usize) -> Result<Vec<SpanEntry>, StorageError> {
        let objects = self.client
            .list_objects_v2()
            .bucket(&self.bucket)
            .prefix(&format!("{}/", self.prefix))
            .max_keys(limit as i32)
            .send()
            .await
            .map_err(|e| StorageError::ReadFailed(e.to_string()))?;

        let mut spans = Vec::new();
        for object in objects.contents() {
            if let (Some(key), Some(last_modified)) = (object.key(), object.last_modified()) {
                let seconds: u64 = last_modified.secs()
                    .try_into()
                    .map_err(|_| StorageError::ReadFailed("Invalid timestamp".into()))?;
                let system_time = UNIX_EPOCH + Duration::from_secs(seconds);
                
                spans.push(SpanEntry {
                    key: key.to_string(),
                    last_modified: system_time,
                });
            }
        }

        spans.sort_by(|a, b| b.last_modified.cmp(&a.last_modified));
        Ok(spans.into_iter().take(limit).collect())
    }

    /// Reads a stored span by its key
    pub async fn read_span(&self, key: &str) -> Result<StoredSpan, StorageError> {
        let response = self.client
            .get_object()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await
            .map_err(|e| StorageError::ReadFailed(e.to_string()))?;

        let data = response
            .body
            .collect()
            .await
            .map_err(|e| StorageError::ReadFailed(e.to_string()))?;

        serde_json::from_slice(&data.into_bytes())
            .map_err(|e| StorageError::ReadFailed(e.to_string()))
    }

    pub fn get_health_status(&self) -> HealthStatus {
        HealthStatus {
            is_healthy: true,  // TODO: Implement proper health check
            last_write: 0,     // TODO: Track last write
            queue_size: 0,     // TODO: Track queue size
            total_processed: 0, // TODO: Track processed count
            failed_writes: 0,  // TODO: Track failed writes
        }
    }
}

#[async_trait]
impl StorageWriter for S3StorageWriter {
    async fn write(&self, key: &str, data: &[u8]) -> Result<(), StorageError> {
        let full_key = self.get_full_key(key);
        
        info!("Writing object to S3: {}/{}", self.bucket, full_key);
        
        match self.client
            .put_object()
            .bucket(&self.bucket)
            .key(&full_key)
            .body(data.to_vec().into())
            .send()
            .await
        {
            Ok(_) => {
                info!("Successfully wrote object: {}/{}", self.bucket, full_key);
                Ok(())
            }
            Err(e) => {
                error!("Failed to write object {}/{}: {}", self.bucket, full_key, e);
                Err(StorageError::WriteFailed(e.to_string()))
            }
        }
    }

    async fn write_batch(&self, entries: Vec<(&str, &[u8])>) -> Result<(), StorageError> {
        for (key, data) in entries {
            self.write(key, data).await?;
        }
        Ok(())
    }

    async fn flush(&self) -> Result<(), StorageError> {
        // S3 writes are immediate, no need to flush
        Ok(())
    }

    async fn write_spans(&self, spans: Vec<SpanData>) -> Result<(), StorageError> {
        for span in spans {
            let key = format!("{}/{}/{}.json", 
                self.prefix,
                span.span_context.trace_id(),
                span.span_context.span_id()
            );

            // Convert SpanData to a serializable format
            let span_json = json!({
                "trace_id": span.span_context.trace_id().to_string(),
                "span_id": span.span_context.span_id().to_string(),
                "name": span.name,
                "kind": format!("{:?}", span.span_kind),
                "start_time": span.start_time.duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos(),
                "end_time": span.end_time.duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos(),
                "status": format!("{:?}", span.status),
            });

            let data = serde_json::to_vec(&span_json)
                .map_err(|e| StorageError::WriteFailed(e.to_string()))?;

            self.write(&key, &data).await?;
        }
        Ok(())
    }
}
