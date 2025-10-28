use anyhow::Result;
use clap::Parser;
use std::net::SocketAddr;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tracing::{error, info, warn};

#[derive(Parser, Debug)]
#[command(author, version, about = "Kermes Server - K8s debug container proxy server", long_about = None)]
struct Args {
    /// Port to listen on
    #[arg(short, long, default_value = "8080")]
    port: u16,

    /// Bind address
    #[arg(short, long, default_value = "0.0.0.0")]
    bind: String,
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
    let addr = format!("{}:{}", args.bind, args.port);

    info!("Starting Kermes Server on {}", addr);

    let listener = TcpListener::bind(&addr).await?;
    info!("Listening on {}", addr);

    loop {
        match listener.accept().await {
            Ok((socket, peer_addr)) => {
                info!("Accepted connection from {}", peer_addr);
                tokio::spawn(async move {
                    if let Err(e) = handle_connection(socket, peer_addr).await {
                        error!("Error handling connection from {}: {}", peer_addr, e);
                    }
                });
            }
            Err(e) => {
                error!("Failed to accept connection: {}", e);
            }
        }
    }
}

async fn handle_connection(mut socket: TcpStream, peer_addr: SocketAddr) -> Result<()> {
    let mut buf = vec![0u8; 8192];

    loop {
        let n = match socket.read(&mut buf).await {
            Ok(0) => {
                info!("Connection closed by {}", peer_addr);
                return Ok(());
            }
            Ok(n) => n,
            Err(e) => {
                warn!("Error reading from {}: {}", peer_addr, e);
                return Err(e.into());
            }
        };

        // Echo back for now - in a real proxy, this would forward to the target
        if let Err(e) = socket.write_all(&buf[..n]).await {
            warn!("Error writing to {}: {}", peer_addr, e);
            return Err(e.into());
        }
    }
}
