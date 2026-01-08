# Load Balancer

A small, async HTTP request proxy / load-balancer written in Rust using axum. It forwards incoming requests to backend servers, runs a background health checker, and exposes a simple health endpoint.

## Quick Start

Prerequisites:

- Rust toolchain (stable)
- Cargo

Create a `.env` file at the project root (example):

```bash
AVAILABLE_SERVERS=http://localhost:3001|4,http://localhost:3002|8
PORT=8080
```

Build and run:

```sh
cargo run
# or for optimized build
cargo run --release
```

The server binds to the configured PORT.

## Endpoints

- `GET /status` — local health check.

## Configuration

This project reads configuration from environment variables. The important variables are:

`AVAILABLE_SERVERS` — comma-separated list of backend base URLs and their weights (e.g. http://host:port|weight).

`PORT` — port to bind the load balancer to.

## Behavior notes

The middleware converts incoming request body to JSON (if present) and forwards via the reqwest client. Health checks use the `/status` endpoint of each backend. All other incoming requests are intercepted and proxied to one of the configured backend servers. The background worker is spawned automatically from main and logs failing servers. Implement a real load-balancing algorithm in. Persist server list in Redis or another service instead of the env var [TODO]
