use aws_sdk_s3::Client as S3Client;
use aws_sdk_s3::config::{Credentials, Region};
use aws_sdk_s3::config::Builder as S3Builder;
use tracing::{info, error};

use crate::error::StorageError;
use async_trait::async_trait;

#[async_trait]
pub trait StorageWriter {
    async fn write(&self, key: &str, data: &[u8]) -> Result<(), StorageError>;
    async fn write_batch(&self, entries: Vec<(&str, &[u8])>) -> Result<(), StorageError>;
    async fn flush(&self) -> Result<(), StorageError>;
}

pub struct S3StorageWriter {
    client: S3Client,
    bucket: String,
    prefix: String,
}

impl S3StorageWriter {
    pub async fn new(bucket: String, prefix: String) -> Result<Self, StorageError> {
        info!("Initializing S3 storage writer for bucket: {}", bucket);
        
        // Configure for LocalStack
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

        // Create S3 config with path style
        let s3_config = S3Builder::from(&config)
            .force_path_style(true)
            .build();

        let client = S3Client::from_conf(s3_config);

        // Verify bucket exists
        match client.head_bucket().bucket(&bucket).send().await {
            Ok(_) => info!("Successfully connected to bucket: {}", bucket),
            Err(e) => {
                error!("Failed to connect to bucket {}: {}", bucket, e);
                return Err(StorageError::ConnectionError(format!("Bucket verification failed: {}", e)));
            }
        }

        Ok(Self {
            client,
            bucket,
            prefix,
        })
    }

    fn get_full_key(&self, key: &str) -> String {
        if self.prefix.is_empty() {
            key.to_string()
        } else {
            format!("{}/{}", self.prefix.trim_end_matches('/'), key)
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
}
