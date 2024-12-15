use tokio::sync::mpsc;
use tracing::{error, info};

use crate::error::ProcessingError;
use crate::proto::storage_engine::{Message, ProcessResponse};

/*
    Contains the EngineCore implementation
    Handles message processing logic
    Implements error handling and validation
*/

pub struct EngineCore {
    message_receiver: mpsc::Receiver<Message>,
}

impl EngineCore {
    pub fn new(receiver: mpsc::Receiver<Message>) -> Self {
        Self {
            message_receiver: receiver,
        }
    }

    pub async fn process_messages(&mut self) {
        while let Some(message) = self.message_receiver.recv().await {
            match self.process_message(message).await {
                Ok(_) => info!("Message processed successfully"),
                Err(e) => error!("Failed to process message: {}", e),
            }
        }
    }

    pub async fn process_message(&self, message: Message) -> Result<ProcessResponse, ProcessingError> {
        // Validate message
        if message.id.is_empty() {
            return Err(ProcessingError::ValidationError("Empty message ID".into()));
        }

        // Log processing
        info!(
            "Processing message {} at timestamp {}",
            message.id, message.timestamp
        );

        // Return success response
        Ok(ProcessResponse {
            success: true,
            message: "Message processed successfully".into(),
        })
    }
}
