use anyhow::{Context, Result};
use clap::Parser;
use std::net::SocketAddr;
use tokio::io;
use tokio::net::{TcpListener, TcpStream};
use tracing::{error, info, warn};

#[derive(Parser, Debug)]
#[command(author, version, about = "Kermes Client - Local proxy client", long_about = None)]
struct Args {
    /// Local port to listen on
    #[arg(short, long, default_value = "3000")]
    local_port: u16,

    /// Local bind address
    #[arg(short = 'b', long, default_value = "127.0.0.1")]
    local_bind: String,

    /// Remote server address (e.g., localhost:8080)
    #[arg(short, long)]
    remote: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let args = Args::parse();
    let local_addr = format!("{}:{}", args.local_bind, args.local_port);

    info!("Starting Kermes Client");
    info!("Local proxy: {}", local_addr);
    info!("Remote server: {}", args.remote);

    let listener = TcpListener::bind(&local_addr)
        .await
        .context("Failed to bind local address")?;
    info!("Listening on {}", local_addr);

    loop {
        match listener.accept().await {
            Ok((client_socket, client_addr)) => {
                info!("Accepted connection from {}", client_addr);
                let remote = args.remote.clone();

                tokio::spawn(async move {
                    if let Err(e) = handle_client(client_socket, client_addr, &remote).await {
                        error!("Error handling connection from {}: {}", client_addr, e);
                    }
                });
            }
            Err(e) => {
                error!("Failed to accept connection: {}", e);
            }
        }
    }
}

async fn handle_client(
    mut client_socket: TcpStream,
    client_addr: SocketAddr,
    remote_addr: &str,
) -> Result<()> {
    // Connect to remote server
    info!("Connecting to remote server: {}", remote_addr);
    let mut server_socket = TcpStream::connect(remote_addr)
        .await
        .context("Failed to connect to remote server")?;
    info!("Connected to remote server");

    // Bidirectional proxy: copy data between client and server
    let (mut client_read, mut client_write) = client_socket.split();
    let (mut server_read, mut server_write) = server_socket.split();

    let client_to_server = async {
        let result = io::copy(&mut client_read, &mut server_write).await;
        match &result {
            Ok(bytes) => info!("Client {} -> Server: {} bytes", client_addr, bytes),
            Err(e) => warn!("Error copying client {} -> server: {}", client_addr, e),
        }
        result
    };

    let server_to_client = async {
        let result = io::copy(&mut server_read, &mut client_write).await;
        match &result {
            Ok(bytes) => info!("Server -> Client {}: {} bytes", client_addr, bytes),
            Err(e) => warn!("Error copying server -> client {}: {}", client_addr, e),
        }
        result
    };

    // Run both directions concurrently
    tokio::select! {
        _ = client_to_server => {},
        _ = server_to_client => {},
    }

    info!("Connection from {} closed", client_addr);
    Ok(())
}
