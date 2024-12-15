use crate::error::ProcessingError;
use crate::proto::{
    TraceService,
    ExportTraceServiceRequest,
    ExportTraceServiceResponse,
};
use tokio::sync::mpsc;
use tonic::{Request, Response, Status};
use std::sync::Arc;
use crate::health::{HealthCheck, HealthStatus};
use tracing::{info, warn, error};
use std::time::Duration;

/// Server component that handles gRPC trace collection requests.
/// Forwards received traces to the processing engine via channels.
pub struct ListenerServer {
    /// Channel for sending messages to the processing engine
    message_sender: mpsc::Sender<ExportTraceServiceRequest>,
    /// Health monitoring for the server
    health_check: Arc<HealthCheck>,
}

impl ListenerServer {
    /// Creates a new ListenerServer instance
    pub fn new(
        sender: mpsc::Sender<ExportTraceServiceRequest>,
        health_check: Arc<HealthCheck>,
    ) -> Self {
        Self {
            message_sender: sender,
            health_check,
        }
    }

    /// Returns the current health status of the server
    pub fn get_health_status(&self) -> HealthStatus {
        self.health_check.get_health_status()
    }

    pub async fn shutdown(&self) -> Result<(), ProcessingError> {
        info!("Server shutting down gracefully...");
        self.health_check.update_status(false);
        
        // Wait for pending messages to be processed
        tokio::time::sleep(Duration::from_secs(1)).await;
        
        Ok(())
    }
}

#[tonic::async_trait]
impl TraceService for ListenerServer {
    /// Handles trace export requests from clients
    /// 
    /// # Arguments
    /// * `request` - The incoming trace export request
    /// 
    /// # Returns
    /// * `Ok(Response)` - If the traces were successfully queued
    /// * `Err(Status)` - If there was an error processing the request
    async fn export(
        &self,
        request: Request<ExportTraceServiceRequest>,
    ) -> Result<Response<ExportTraceServiceResponse>, Status> {
        let message = request.into_inner();

        // Attempt to send message to processing engine
        match self.message_sender.send(message).await {
            Ok(_) => {
                info!("Successfully queued trace data for processing");
                Ok(Response::new(ExportTraceServiceResponse {}))
            }
            Err(e) => {
                warn!("Failed to queue trace data: {}", e);
                Err(Status::internal("Failed to queue trace data"))
            }
        }
    }
}

/// Converts processing errors to gRPC status codes
impl From<ProcessingError> for Status {
    fn from(error: ProcessingError) -> Self {
        match error {
            ProcessingError::ValidationError(msg) => {
                warn!("Validation error: {}", msg);
                Status::invalid_argument(msg)
            }
            ProcessingError::ProcessingFailed(msg) => {
                error!("Processing failed: {}", msg);
                Status::internal(msg)
            }
            ProcessingError::StorageError(msg) => {
                error!("Storage error: {}", msg);
                Status::internal(msg)
            }
            ProcessingError::RateLimitExceeded(msg) => {
                warn!("Rate limit exceeded: {}", msg);
                Status::resource_exhausted(msg)
            }
            ProcessingError::NotFound(msg) => {
                info!("Resource not found: {}", msg);
                Status::not_found(msg)
            }
            ProcessingError::ShutdownError(msg) => {
                error!("Shutdown error: {}", msg);
                Status::internal(msg)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::sync::mpsc;

    #[tokio::test]
    async fn test_export_success() {
        let (tx, mut rx) = mpsc::channel(1);
        let health_check = Arc::new(HealthCheck::new());
        let server = ListenerServer::new(tx, health_check);

        // Create test request
        let request = Request::new(ExportTraceServiceRequest {
            resource_spans: vec![],
        });

        // Send request
        let response = server.export(request).await;
        assert!(response.is_ok());

        // Verify message was queued
        let received = rx.try_recv();
        assert!(received.is_ok());
    }

    #[tokio::test]
    async fn test_export_channel_closed() {
        let (tx, _rx) = mpsc::channel(1);
        let health_check = Arc::new(HealthCheck::new());
        let server = ListenerServer::new(tx, health_check);

        // Drop receiver to close channel
        drop(_rx);

        // Create test request
        let request = Request::new(ExportTraceServiceRequest {
            resource_spans: vec![],
        });

        // Send request should fail
        let response = server.export(request).await;
        assert!(response.is_err());
    }
}
