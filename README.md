# Kermes

A high-performance proxy server and client for debugging Kubernetes environments, written in Rust.

## Overview

Kermes provides two components:

1. **kermes-server**: A lightweight TCP proxy server designed to run as a debug container in Kubernetes
2. **kermes-client**: A local proxy client that connects to the server and forwards traffic bidirectionally

## Features

- ⚡ **High Performance**: Built with Rust and Tokio for async I/O and maximum throughput
- 🔌 **Socket Proxy**: Full TCP socket proxy support with bidirectional data transfer
- 🚢 **Kubernetes Ready**: Server designed for deployment as a K8s debug container
- 🎯 **Simple**: Minimal configuration, just works
- 📊 **Observable**: Integrated logging with tracing

## Architecture

```
Local Application → kermes-client (local) → kermes-server (K8s) → Target Service
```

## Installation

### Prerequisites

- Rust 1.70+ (for building from source)
- Docker (for containerization)
- kubectl (for Kubernetes deployment)

### Building from Source

```bash
# Build both packages
cargo build --release

# Build only server
cargo build --release --package kermes-server

# Build only client
cargo build --release --package kermes-client
```

Binaries will be available in `target/release/`.

### Docker Image

Build the server Docker image:

```bash
docker build -f Dockerfile.server -t kermes-server:latest .
```

## Usage

### Server (K8s Debug Container)

Deploy to Kubernetes:

```bash
kubectl apply -f k8s/deployment.yaml
```

Or run directly:

```bash
kermes-server --bind 0.0.0.0 --port 8080
```

Options:
- `-b, --bind <ADDRESS>`: Bind address (default: 0.0.0.0)
- `-p, --port <PORT>`: Port to listen on (default: 8080)

### Client (Local Proxy)

```bash
kermes-client --local-port 3000 --remote <SERVER_ADDRESS>:8080
```

Options:
- `-l, --local-port <PORT>`: Local port to listen on (default: 3000)
- `-b, --local-bind <ADDRESS>`: Local bind address (default: 127.0.0.1)
- `-r, --remote <ADDRESS>`: Remote server address (required)

Example:

```bash
# Forward local port 3000 to kermes-server in K8s
kermes-client --local-port 3000 --remote kermes-server.kermes-debug.svc.cluster.local:8080
```

## Kubernetes Integration

The server integrates with Kubernetes by:
- Running as a lightweight debug container
- Using minimal resources (64Mi memory, 100m CPU)
- Supporting standard K8s networking
- Ready for service mesh integration

### Port Forwarding to Access Server

```bash
kubectl port-forward -n kermes-debug svc/kermes-server 8080:8080
```

Then connect with the client:

```bash
kermes-client --remote localhost:8080
```

## Configuration

### Environment Variables

Both server and client support:
- `RUST_LOG`: Set logging level (e.g., `info`, `debug`, `trace`)

Example:

```bash
RUST_LOG=debug kermes-server --port 8080
```

## Development

### Project Structure

```
kermes/
├── Cargo.toml              # Workspace configuration
├── kermes-server/          # Server package (K8s debug container)
│   ├── Cargo.toml
│   └── src/
│       └── main.rs
├── kermes-client/          # Client package (local proxy)
│   ├── Cargo.toml
│   └── src/
│       └── main.rs
├── k8s/                    # Kubernetes manifests
│   └── deployment.yaml
└── Dockerfile.server       # Server container image
```

### Dependencies

- **tokio**: Async runtime for high-performance I/O
- **clap**: CLI argument parsing
- **tracing**: Structured logging
- **kube**: Kubernetes API client (server only)
- **anyhow**: Error handling

### Running Tests

```bash
cargo test
```

### Linting

```bash
cargo clippy
cargo fmt
```

## Performance

Kermes is designed for high performance:
- Zero-copy I/O where possible
- Async/await for efficient concurrency
- Minimal memory footprint
- No unnecessary allocations in hot paths

## Security Considerations

- Run server with minimal privileges in K8s
- Use network policies to restrict access
- Consider TLS for production deployments
- Monitor and log all connections

## License

MIT - See LICENSE file for details

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

