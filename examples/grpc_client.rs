/*
    Test client implementation
    Demonstrates gRPC communication
    Uses feature-gated client code
*/

#[cfg(feature = "client")]
use storage_engine::proto::{Message, StorageEngineClient};
use tonic::transport::Channel;
use tracing::{info, error};

#[cfg(feature = "client")]
async fn send_message(
    client: &mut StorageEngineClient<Channel>,
    message_id: &str,
    content: &str,
) -> Result<bool, Box<dyn std::error::Error>> {
    let message = Message {
        id: message_id.to_string(),
        content: content.to_string(),
        timestamp: chrono::Utc::now().timestamp(),
    };

    info!(
        "Sending message: [ID: {}, Content: {}, Timestamp: {}]",
        message.id, message.content, message.timestamp
    );

    let request = tonic::Request::new(message);
    let response = client.process_message(request).await?;
    let response = response.into_inner();

    info!(
        "Response received: [Success: {}, Message: {}]",
        response.success, response.message
    );

    Ok(response.success)
}

#[cfg(feature = "client")]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Setup logging
    tracing_subscriber::fmt::init();

    info!("Connecting to server at http://[::1]:50051...");
    let channel = Channel::from_static("http://[::1]:50051").connect().await?;
    let client = StorageEngineClient::new(channel);

    info!("Connected successfully!");

    // Create test messages
    let test_messages = vec![
        ("msg_1", "First test message"),
        ("msg_2", "Second test message"),
        ("msg_3", "Third test message"),
    ];

    // Process messages concurrently
    let handles: Vec<_> = test_messages
        .into_iter()
        .map(|(id, content)| {
            let mut client_clone = client.clone();
            tokio::spawn(async move {
                match send_message(&mut client_clone, id, content).await {
                    Ok(success) => {
                        if success {
                            info!("Message {} processed successfully", id);
                        } else {
                            error!("Message {} failed to process", id);
                        }
                    }
                    Err(e) => {
                        error!("Error sending message {}: {}", id, e);
                    }
                }
            })
        })
        .collect();

    // Wait for all messages to be processed
    for handle in handles {
        handle.await?;
    }

    info!("All messages processed");
    Ok(())
}

#[cfg(not(feature = "client"))]
fn main() {
    println!("This binary requires the 'client' feature to be enabled");
}

