use storage_engine::proto::storage_engine::storage_engine_server::StorageEngineServer;
use storage_engine::{EngineCore, ListenerServer};

use tokio::sync::mpsc;

use tonic::transport::Server;

use tracing::{info, warn, Level};
use tracing_subscriber::{EnvFilter, FmtSubscriber};

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

// handles keyboard ctrl+c input as shutdown signal
async fn shutdown_signal() {
    tokio::signal::ctrl_c()
        .await
        .expect("Failed to install CTRL+C signal handler");
}