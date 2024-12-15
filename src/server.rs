use crate::error::ProcessingError;
use crate::proto::storage_engine::{
    storage_engine_server::StorageEngine,
    Message, ProcessResponse
};
use tokio::sync::mpsc;
use tonic::{Request, Response, Status};

/*
    Implements the gRPC server (ListenerServer)
    Handles incoming message requests
    Forwards messages to EngineCore via channels
*/

pub struct ListenerServer {
    message_sender: mpsc::Sender<Message>,
}

impl ListenerServer {
    pub fn new(sender: mpsc::Sender<Message>) -> Self {
        Self {
            message_sender: sender,
        }
    }
}

#[tonic::async_trait]
impl StorageEngine for ListenerServer {
    async fn process_message(
        &self,
        request: Request<Message>,
    ) -> Result<Response<ProcessResponse>, Status> {
        let message = request.into_inner();

        // Send message to engine core
        match self.message_sender.send(message).await {
            Ok(_) => Ok(Response::new(ProcessResponse {
                success: true,
                message: "Message queued for processing".to_string(),
            })),
            Err(_) => Err(Status::internal("Failed to queue message for processing")),
        }
    }
}

impl From<ProcessingError> for Status {
    fn from(error: ProcessingError) -> Self {
        match error {
            ProcessingError::ValidationError(msg) => Status::invalid_argument(msg),
            ProcessingError::ProcessingFailed(msg) => Status::internal(msg),
            ProcessingError::StorageError(msg) => Status::internal(msg),
        }
    }
}
