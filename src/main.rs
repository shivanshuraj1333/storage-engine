use storage_engine::proto::storage_engine::storage_engine_server::StorageEngineServer;
use storage_engine::{config::ProcessingConfig, EngineCore, ListenerServer};

use tokio::sync::mpsc;

use tonic::transport::Server;

use tracing::{info, warn, Level};
use tracing_subscriber::{EnvFilter, FmtSubscriber};

use std::time::Duration;

/*  Details about libraries used
    initialized all libs
    storage engine helps in getting proto generated files

    tokio is used for async comm

    tonic helps in starting the server

    tracing is for logging info and warn logs
*/

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    // Setup logging with explicit level
    FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .with_env_filter(EnvFilter::from_default_env()
            .add_directive(Level::INFO.into()))
        .init();

    // Create channels for message passing
    let (tx, rx) = mpsc::channel(100);

    // Create processing configuration
    let processing_config = ProcessingConfig {
        batch_size: 10,        // Process in batches of 10
        batch_timeout_ms: 10000, // Or every 10 second
    };

    // Create server and engine core
    let mut engine_core = EngineCore::new(rx, processing_config).await?;
    let health_check = engine_core.get_health_check();
    let listener_server = ListenerServer::new(tx, health_check);

    // Add health check logging
    tokio::spawn({
        let health_check = engine_core.get_health_check();
        async move {
            let mut interval = tokio::time::interval(Duration::from_secs(60));
            loop {
                interval.tick().await;
                let status = health_check.get_health_status();
                info!(
                    "Health status: healthy={}, queue_size={}, total_processed={}, failed_writes={}, last_write={}",
                    status.is_healthy,
                    status.queue_size,
                    status.total_processed,
                    status.failed_writes,
                    status.last_write
                );
            }
        }
    });

    // Spawn engine core processing
    tokio::spawn(async move {
        engine_core.process_messages().await;
    });

    // Define server address
    let addr = "[::1]:50051".parse()?;

    // Start gRPC server
    info!("Server listening on {}", addr);

    let server = Server::builder()
        .add_service(StorageEngineServer::new(listener_server))
        .serve(addr);

    let shutdown = shutdown_signal();
    tokio::select! {
        result = server => {
            if let Err(e) = result {
                warn!("Server error: {}", e);
            }
        }
        _ = shutdown => {
            info!("Shutting down gracefully...");
        }
    }

    Ok(())
}

// handles keyboard ctrl+c input as shutdown signal
async fn shutdown_signal() {
    tokio::signal::ctrl_c()
        .await
        .expect("Failed to install CTRL+C signal handler");
}