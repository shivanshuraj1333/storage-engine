use storage_engine::proto::server::StorageEngineServer;
use storage_engine::{EngineCore, ListenerServer};
use tokio::sync::mpsc;
use tonic::transport::Server;
use tracing_subscriber::EnvFilter;

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
    println!("Server listening on {}", addr);
    Server::builder()
        .add_service(StorageEngineServer::new(listener_server))
        .serve(addr)
        .await?;

    Ok(())
}
