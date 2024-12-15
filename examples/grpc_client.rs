/*
    OpenTelemetry Trace Client Example
    
    This example demonstrates:
    - Connecting to the trace collector
    - Generating sample trace data
    - Sending traces using OTLP format
    - Handling responses and errors
*/

use storage_engine::proto::{
    TraceServiceClient,
    ExportTraceServiceRequest,
    ResourceSpans, ScopeSpans, Span,
};
use tonic::transport::Channel;
use tracing::{info, error};
use uuid::Uuid;
use std::time::{SystemTime, UNIX_EPOCH};

#[cfg(feature = "client")]
fn generate_sample_trace() -> ExportTraceServiceRequest {
    // Generate unique IDs for the trace
    let trace_id = Uuid::new_v4().as_bytes().to_vec();
    let span_id = Uuid::new_v4().as_bytes()[..8].to_vec();
    
    // Get current timestamp
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos() as u64;

    // Create a sample span
    let span = Span {
        trace_id,
        span_id,
        parent_span_id: vec![],
        name: "process_order".to_string(),
        kind: 1, // INTERNAL
        start_time_unix_nano: now,
        end_time_unix_nano: now + 1_000_000_000, // 1 second later
        attributes: vec![],
        dropped_attributes_count: 0,
        events: vec![],
        dropped_events_count: 0,
        links: vec![],
        dropped_links_count: 0,
        status: None,
        flags: 0,
        trace_state: String::new(),
    };

    // Create the request structure
    ExportTraceServiceRequest {
        resource_spans: vec![ResourceSpans {
            resource: None,
            scope_spans: vec![ScopeSpans {
                scope: None,
                spans: vec![span],
                schema_url: String::new(),
            }],
            schema_url: String::new(),
        }],
    }
}

#[cfg(feature = "client")]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Setup logging
    tracing_subscriber::fmt::init();

    // Connect to the collector
    info!("Connecting to trace collector at http://[::1]:50051...");
    let channel = Channel::from_static("http://[::1]:50051").connect().await?;
    let mut client = TraceServiceClient::new(channel);
    info!("Connected successfully!");

    // Generate and send multiple traces
    for i in 1..=3 {
        info!("Sending trace batch {}...", i);
        let request = generate_sample_trace();
        
        match client.export(request).await {
            Ok(_) => info!("Successfully exported trace batch {}", i),
            Err(e) => error!("Failed to export trace batch {}: {}", i, e),
        }
    }

    info!("All trace batches sent");
    Ok(())
}

#[cfg(not(feature = "client"))]
fn main() {
    println!("This binary requires the 'client' feature to be enabled");
}

