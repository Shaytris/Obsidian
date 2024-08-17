mod ws_server;
mod channel;
mod message;
mod utils;
mod mutliconnect;

use tokio::net::TcpListener;
use ws_server::handle_connection;

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:8080").await.expect("Failed to bind port");

    println!("WebSocket server listening on port 8080");

    while let Ok((stream, _)) = listener.accept().await {
        tokio::spawn(async move {
            if let Err(e) = handle_connection(stream).await {
                eprintln!("Error handling connection: {:?}", e);
            }
        });
    }
}

fn shutdown_gracefully() {
    // Handle graceful shutdown of the server
    println!("Server is shutting down...");
    // Clean up database connections, notify users, etc.
}

fn print_server_stats() {
    // Function to print server stats for monitoring
    println!("Server statistics:");
    // Example stats: active connections, message throughput, etc.
}