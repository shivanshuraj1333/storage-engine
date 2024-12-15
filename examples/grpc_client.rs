/*
    Test client implementation
    Demonstrates gRPC communication
    Uses feature-gated client code
*/

#[cfg(feature = "client")]
use storage_engine::proto::{client::StorageEngineClient, common::Message};

#[cfg(feature = "client")]
use tonic::transport::Channel;

#[cfg(feature = "client")]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Connecting to server at http://[::1]:50051...");

    let channel = Channel::from_static("http://[::1]:50051").connect().await?;

    let mut client = StorageEngineClient::new(channel);

    println!("Connected successfully!");

    let message = Message {
        id: "test_message_1".to_string(),
        content: "Hello, StorageEngine!".to_string(),
        timestamp: chrono::Utc::now().timestamp(),
    };

    println!("Sending message:");
    println!(" -> ID: {}", message.id);
    println!(" -> Content: {}", message.content);
    println!(" -> Timestamp: {}", message.timestamp);

    let request = tonic::Request::new(message);
    let response = client.process_message(request).await?;
    let response = response.into_inner();

    println!("Received response:");
    println!(" -> Success: {}", response.success);
    println!(" -> Message: {}", response.message);

    Ok(())
}
