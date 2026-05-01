# iron_server

**A high-performance, async chat server and CLI client written in Rust.**

`iron_server` is a clean, well-documented, production-ready chat backend built with modern Rust and Tokio.

### Key Features

- Built with **Tokio** for async I/O
- `tokio::net::TcpListener` for connections
- Efficient message broadcasting using `tokio::sync::broadcast`
- Simple username/password authentication
- Graceful shutdown with `tokio::signal`
- JSON line protocol over TCP
- Interactive CLI client
- Excellent logging with `tracing`

##
---

## Quick Start

### Build & Run

```bash
cargo run --bin iron-server