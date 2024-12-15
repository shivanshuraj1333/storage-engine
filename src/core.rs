use std::time::Duration;
use std::sync::Arc;

use tokio::sync::mpsc;
use tokio::time::{self, Instant};

use tracing::{error, info};

use crate::config::ProcessingConfig;

use crate::error::{ProcessingError, StorageError};

use crate::proto::storage_engine::{Message, ProcessResponse};

use crate::storage::{S3StorageWriter, StorageWriter};

use crate::health::HealthCheck;

/// Core engine that handles message processing and batching
pub struct EngineCore {
    message_receiver: mpsc::Receiver<Message>,
    batch_size: usize,
    batch_timeout: Duration,
    message_queue: Vec<Message>,
    storage_writer: S3StorageWriter,
    health_check: Arc<HealthCheck>,
}

impl EngineCore {
    /// Creates a new EngineCore with the specified configuration
    pub async fn new(receiver: mpsc::Receiver<Message>, config: ProcessingConfig) -> Result<Self, StorageError> {
        let storage_writer = S3StorageWriter::new(
            "my-test-bucket".to_string(),
            "messages".to_string(),
        ).await?;

        Ok(Self {
            message_receiver: receiver,
            batch_size: config.batch_size,
            batch_timeout: Duration::from_millis(config.batch_timeout_ms),
            message_queue: Vec::with_capacity(config.batch_size),
            storage_writer,
            health_check: Arc::new(HealthCheck::new()),
        })
    }

    pub fn get_health_check(&self) -> Arc<HealthCheck> {
        Arc::clone(&self.health_check)
    }

    /// Main message processing loop
    pub async fn process_messages(&mut self) {
        let mut batch_timer = time::interval_at(
            Instant::now() + self.batch_timeout,
            self.batch_timeout,
        );

        loop {
            tokio::select! {
                _ = batch_timer.tick() => {
                    if !self.message_queue.is_empty() {
                        self.process_batch().await;
                    }
                }
                Some(message) = self.message_receiver.recv() => {
                    self.message_queue.push(message);
                    if self.message_queue.len() >= self.batch_size {
                        self.process_batch().await;
                        batch_timer.reset();
                    }
                }
                else => break,
            }
            self.health_check.update_queue_size(self.message_queue.len() as u64);
        }
    }

    /// Processes a batch of messages
    async fn process_batch(&mut self) {
        info!("Processing batch of {} messages", self.message_queue.len());
        
        let messages = std::mem::take(&mut self.message_queue);
        for message in messages {
            match self.process_message(message).await {
                Ok(_) => info!("Message processed successfully"),
                Err(e) => error!("Failed to process message: {}", e),
            }
        }
    }

    /// Processes a single message
    async fn process_message(&self, message: Message) -> Result<ProcessResponse, ProcessingError> {
        if message.id.is_empty() {
            return Err(ProcessingError::ValidationError("Empty message ID".into()));
        }

        // Prepare message data
        let data = format!(
            "Message ID: {}\nContent: {}\nTimestamp: {}", 
            message.id, 
            message.content,
            message.timestamp
        );

        // Write to storage
        match self.storage_writer
            .write(&message.id, data.as_bytes())
            .await
        {
            Ok(_) => {
                self.health_check.record_successful_write();
                info!(
                    "Processing message {} at timestamp {}",
                    message.id, message.timestamp
                );
            }
            Err(e) => {
                self.health_check.record_failed_write();
                return Err(ProcessingError::StorageError(e.to_string()));
            }
        }

        Ok(ProcessResponse {
            success: true,
            message: "Message processed and stored successfully".into(),
        })
    }

    pub async fn shutdown(&mut self) -> Result<(), ProcessingError> {
        info!("Initiating graceful shutdown...");
        
        // Process remaining messages
        while let Some(message) = self.message_queue.pop() {
            self.process_message(message).await?;
        }
        
        // Flush any remaining writes
        self.storage_writer.flush().await?;
        
        info!("Shutdown complete");
        Ok(())
    }
}
