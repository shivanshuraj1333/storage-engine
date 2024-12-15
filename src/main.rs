use storage_engine::proto::storage_engine::storage_engine_server::StorageEngineServer;
use storage_engine::{EngineCore, ListenerServer};
use tokio::sync::mpsc;
use tonic::transport::Server;
use tracing::{info, warn};
use tracing_subscriber::EnvFilter;

async fn shutdown_signal() {
    tokio::signal::ctrl_c()
        .await
        .expect("Failed to install CTRL+C signal handler");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Setup logging
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    // Create channels for message passing
    let (tx, rx) = mpsc::channel(100);

    // Create server and engine core
    let listener_server = ListenerServer::new(tx);
    let mut engine_core = EngineCore::new(rx);

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