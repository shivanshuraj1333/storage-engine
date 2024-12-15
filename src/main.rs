use storage_engine::{
    proto::TraceServiceServer,
    config::ProcessingConfig,
    EngineCore,
    ListenerServer,
    SpanReader,
    S3StorageWriter,
    health::HealthCheck,
    proto::ExportTraceServiceRequest,
};
use tokio::sync::mpsc;
use tonic::transport::Server as GrpcServer;
use axum::serve;
use tracing::{info, warn, Level};
use tracing_subscriber::{EnvFilter, FmtSubscriber};
use std::net::SocketAddr;
use std::sync::Arc;
use std::future::Future;

/// Main entry point for the storage engine server.
/// Sets up and runs both gRPC and HTTP servers for trace collection and querying.
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging with tracing
    setup_logging();

    // Initialize core components
    let (_config, message_sender, engine_core) = setup_core_components().await?;

    // Initialize and spawn the engine core processing
    let health_check = engine_core.get_health_check();
    spawn_engine_core(engine_core);

    // Initialize gRPC server for trace collection
    let grpc_server = setup_grpc_server(message_sender, health_check, "[::1]:50051")?;

    // Initialize HTTP server for span querying
    let (http_server, _http_addr) = setup_http_server().await?;
    
    // Run both servers and handle shutdown
    run_servers(grpc_server, http_server).await?;

    Ok(())
}

/// Sets up logging with tracing subscriber
fn setup_logging() {
    FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .with_env_filter(EnvFilter::from_default_env()
            .add_directive(Level::INFO.into()))
        .init();
}

/// Initializes core components including channels and processing configuration
async fn setup_core_components() -> Result<(
    ProcessingConfig, 
    mpsc::Sender<ExportTraceServiceRequest>, 
    EngineCore
), Box<dyn std::error::Error>> {
    let (tx, rx) = mpsc::channel(100);
    
    let processing_config = ProcessingConfig {
        batch_size: 10,
        batch_timeout_ms: 10000,
    };

    let engine_core = EngineCore::new(rx, processing_config.clone()).await?;
    
    Ok((processing_config, tx, engine_core))
}

/// Spawns the engine core processing task
fn spawn_engine_core(mut engine_core: EngineCore) {
    tokio::spawn(async move {
        engine_core.process_messages().await;
    });
}

/// Sets up the gRPC server for trace collection
fn setup_grpc_server(
    tx: mpsc::Sender<ExportTraceServiceRequest>,
    health_check: Arc<HealthCheck>,
    addr: &str,
) -> Result<impl Future<Output = Result<(), tonic::transport::Error>>, Box<dyn std::error::Error>> {
    let addr = addr.parse()?;
    let listener_server = ListenerServer::new(tx, health_check);
    
    info!("gRPC server listening on {}", addr);
    Ok(GrpcServer::builder()
        .add_service(TraceServiceServer::new(listener_server))
        .serve(addr))
}

/// Sets up the HTTP server for span querying
async fn setup_http_server() -> Result<(
    impl Future<Output = Result<(), std::io::Error>>, 
    SocketAddr
), Box<dyn std::error::Error>> {
    let storage = Arc::new(S3StorageWriter::new(
        "my-test-bucket".to_string(),
        "messages".to_string(),
    ).await?);
    
    let reader = SpanReader::new(Arc::clone(&storage));
    let app = reader.router();
    
    let http_addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    let listener = tokio::net::TcpListener::bind(http_addr).await?;
    
    info!("HTTP server listening on {}", http_addr);
    Ok((
        async move { 
            serve::serve(listener, app).await
        },
        http_addr
    ))
}

/// Runs both gRPC and HTTP servers, handling graceful shutdown
async fn run_servers(
    grpc_server: impl Future<Output = Result<(), tonic::transport::Error>>,
    http_server: impl Future<Output = Result<(), std::io::Error>>,
) -> Result<(), Box<dyn std::error::Error>> {
    tokio::select! {
        result = grpc_server => {
            if let Err(e) = result {
                warn!("gRPC server error: {}", e);
            }
        }
        result = http_server => {
            if let Err(e) = result {
                warn!("HTTP server error: {}", e);
            }
        }
        _ = shutdown_signal() => {
            info!("Shutting down gracefully...");
        }
    }
    Ok(())
}

/// Handles CTRL+C signal for graceful shutdown
async fn shutdown_signal() {
    tokio::signal::ctrl_c()
        .await
        .expect("Failed to install CTRL+C signal handler");
}