use tokio::sync::mpsc;
use tracing::{error, info};

use crate::proto::common::Message;
use crate::error::ProcessingError;

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
            match self.process_single_message(message).await {
                Ok(_) => info!("Message processed successfully"),
                Err(e) => error!("Failed to process message: {}", e),
            }
        }
    }

    async fn process_single_message(&self, message: Message) -> Result<(), ProcessingError> {
        // Validate message
        if message.id.is_empty() {
            return Err(ProcessingError::InvalidMessage(
                "Message ID is required".to_string(),
            ));
        }
        // Basic message processing
        info!(
            "Processing message: id={}, timestamp={}",
            message.id, message.timestamp
        );

        // TODO: Add metadata extraction
        // TODO: Add storage writing logic

        Ok(())
    }
}
