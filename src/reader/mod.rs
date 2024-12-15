use axum::{
    routing::get,
    Router,
    Json,
    extract::{Query, State},
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use crate::storage::{S3StorageWriter, StoredSpan};
use crate::error::StorageError;

/// Query parameters for span retrieval
#[derive(Debug, Deserialize)]
pub struct SpanQuery {
    /// Maximum number of spans to return
    limit: Option<usize>,
}

/// Summary of a span for API responses
#[derive(Debug, Serialize)]
pub struct SpanSummary {
    /// Unique identifier for the trace
    trace_id: String,
    /// Unique identifier for the span
    span_id: String,
    /// Name of the operation
    name: String,
    /// Start time in nanoseconds since epoch
    timestamp: u64,
    /// Duration of the span in nanoseconds
    duration_ns: u64,
}

impl From<StoredSpan> for SpanSummary {
    fn from(span: StoredSpan) -> Self {
        Self {
            trace_id: span.trace_id,
            span_id: span.span_id,
            name: span.name,
            timestamp: span.start_time,
            duration_ns: span.end_time - span.start_time,
        }
    }
}

/// HTTP server component for querying spans
#[derive(Clone)]
pub struct SpanReader {
    /// Storage backend for retrieving spans
    storage: Arc<S3StorageWriter>,
}

impl SpanReader {
    /// Creates a new SpanReader with the specified storage backend
    pub fn new(storage: Arc<S3StorageWriter>) -> Self {
        Self { storage }
    }

    /// Retrieves recent spans from storage
    pub async fn get_recent_spans(&self, limit: usize) -> Result<Vec<SpanSummary>, StorageError> {
        let spans = self.storage.list_spans(limit).await?;
        
        let mut summaries = Vec::new();
        for span in spans {
            if let Ok(content) = self.storage.read_span(&span.key).await {
                summaries.push(SpanSummary::from(content));
            }
        }

        Ok(summaries)
    }

    /// Creates an Axum router with span query endpoints
    pub fn router(self) -> Router {
        Router::new()
            .route("/spans", get(Self::handle_get_spans))
            .route("/health", get(Self::handle_health_check))
            .with_state(Arc::new(self))
    }

    /// Handler for GET /spans endpoint
    async fn handle_get_spans(
        State(reader): State<Arc<SpanReader>>,
        Query(query): Query<SpanQuery>,
    ) -> Json<Vec<SpanSummary>> {
        let limit = query.limit.unwrap_or(5);
        
        // Attempt to get spans, return empty list on error
        let spans = reader.get_recent_spans(limit).await
            .unwrap_or_else(|e| {
                tracing::error!("Failed to get spans: {}", e);
                Vec::new()
            });
            
        Json(spans)
    }

    /// Handler for health check endpoint
    async fn handle_health_check(
        State(reader): State<Arc<SpanReader>>,
    ) -> impl IntoResponse {
        let status = reader.storage.get_health_status();
        Json(status)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::StorageWriter;
    use mockall::predicate::*;
    use mockall::mock;

    mock! {
        StorageWriter {
            fn list_spans(&self, limit: usize) -> Result<Vec<SpanEntry>, StorageError>;
            fn read_span(&self, key: &str) -> Result<StoredSpan, StorageError>;
        }
    }

    #[tokio::test]
    async fn test_get_recent_spans() {
        // TODO: Add tests for span retrieval
    }
}