use std::borrow::Cow;
use std::time::Duration;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::time::{self, Instant};
use tracing::{error, info};

use crate::config::ProcessingConfig;
use crate::error::{ProcessingError, StorageError};
use crate::proto::{ExportTraceServiceRequest, ExportTraceServiceResponse, Span};
use crate::storage::{S3StorageWriter, StorageWriter};
use crate::health::HealthCheck;

use opentelemetry::{
    sdk::{
        export::trace::SpanData,
        trace::{EvictedHashMap, EvictedQueue},
    },
    trace::{SpanContext, SpanId, SpanKind, Status, TraceFlags, TraceId, TraceState},
};

/// Core engine responsible for processing and storing trace data.
/// Handles message batching, span conversion, and storage operations.
pub struct EngineCore {
    /// Channel for receiving trace messages
    message_receiver: mpsc::Receiver<ExportTraceServiceRequest>,
    /// Maximum number of messages to process in a batch
    batch_size: usize,
    /// Maximum time to wait before processing a partial batch
    batch_timeout: Duration,
    /// Queue for accumulating messages before batch processing
    message_queue: Vec<ExportTraceServiceRequest>,
    /// Storage backend for persisting trace data
    storage_writer: S3StorageWriter,
    /// Health monitoring for the engine
    health_check: Arc<HealthCheck>,
}

impl EngineCore {
    /// Creates a new EngineCore with the specified configuration
    pub async fn new(
        receiver: mpsc::Receiver<ExportTraceServiceRequest>,
        config: ProcessingConfig,
    ) -> Result<Self, StorageError> {
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

    /// Returns a reference to the health check monitor
    pub fn get_health_check(&self) -> Arc<HealthCheck> {
        Arc::clone(&self.health_check)
    }

    /// Main message processing loop
    /// Handles batching of messages and triggers processing based on:
    /// - Batch size threshold
    /// - Timeout threshold
    pub async fn process_messages(&mut self) {
        let mut batch_timer = time::interval_at(
            Instant::now() + self.batch_timeout,
            self.batch_timeout,
        );

        loop {
            tokio::select! {
                // Process batch on timer tick if queue not empty
                _ = batch_timer.tick() => {
                    if !self.message_queue.is_empty() {
                        self.process_batch().await;
                    }
                }
                // Process new messages as they arrive
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

    /// Processes a batch of accumulated messages
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

    /// Processes a single message, converting it to spans and storing them
    async fn process_message(
        &self,
        request: ExportTraceServiceRequest
    ) -> Result<ExportTraceServiceResponse, ProcessingError> {
        let spans = self.convert_request_to_spans(request)?;
        
        self.storage_writer.write_spans(spans).await
            .map_err(|e| ProcessingError::StorageError(e.to_string()))?;

        self.health_check.record_successful_write();
        Ok(ExportTraceServiceResponse {})
    }

    /// Converts a trace request into OpenTelemetry spans
    fn convert_request_to_spans(
        &self,
        request: ExportTraceServiceRequest
    ) -> Result<Vec<SpanData>, ProcessingError> {
        let mut spans = Vec::new();
        
        for resource_spans in request.resource_spans {
            for scope_spans in resource_spans.scope_spans {
                for span in scope_spans.spans {
                    spans.push(self.convert_span(span)?);
                }
            }
        }

        Ok(spans)
    }

    /// Converts a proto span into an OpenTelemetry span
    fn convert_span(&self, span: Span) -> Result<SpanData, ProcessingError> {
        let parent_span_id = if !span.parent_span_id.is_empty() {
            SpanId::from_hex(&hex::encode(&span.parent_span_id))
                .map_err(|e| ProcessingError::ValidationError(e.to_string()))?
        } else {
            SpanId::INVALID
        };

        Ok(SpanData {
            span_context: self.create_span_context(&span)?,
            parent_span_id,
            span_kind: SpanKind::Client,
            name: Cow::from(span.name),
            start_time: std::time::SystemTime::UNIX_EPOCH + 
                std::time::Duration::from_nanos(span.start_time_unix_nano),
            end_time: std::time::SystemTime::UNIX_EPOCH + 
                std::time::Duration::from_nanos(span.end_time_unix_nano),
            attributes: EvictedHashMap::new(Default::default(), 128),
            events: EvictedQueue::new(128),
            links: EvictedQueue::new(128),
            status: Status::Ok,
            resource: Default::default(),
            instrumentation_lib: Default::default(),
        })
    }

    /// Creates a span context from a proto span
    fn create_span_context(&self, span: &Span) -> Result<SpanContext, ProcessingError> {
        Ok(SpanContext::new(
            TraceId::from_hex(&hex::encode(&span.trace_id))
                .map_err(|e| ProcessingError::ValidationError(e.to_string()))?,
            SpanId::from_hex(&hex::encode(&span.span_id))
                .map_err(|e| ProcessingError::ValidationError(e.to_string()))?,
            TraceFlags::default(),
            false,
            TraceState::default(),
        ))
    }

    /// Performs graceful shutdown, processing remaining messages
    pub async fn shutdown(&mut self) -> Result<(), ProcessingError> {
        info!("Initiating graceful shutdown...");
        
        while let Some(message) = self.message_queue.pop() {
            self.process_message(message).await?;
        }
        
        self.storage_writer.flush().await?;
        info!("Shutdown complete");
        Ok(())
    }
}
