# Clusterizer

A Minecraft server cluster management tool for efficiently managing multiple Minecraft server instances.

## Overview

Clusterizer is a Rust-based system that helps you manage and coordinate multiple Minecraft server nodes. It provides a REST API server and a CLI client for easy cluster management.

## Architecture

The project is a Rust workspace consisting of several crates:

- **api**: REST API client library
- **cli**: Command-line interface for cluster management
- **client**: Shared client utilities
- **common**: Common types and utilities
- **server**: REST API server (powered by Axum)
- **util**: Utility functions

## Features

- Multi-node Minecraft server cluster management
- REST API backend with HMAC authentication
- PostgreSQL database for cluster state persistence
- Cross-platform CLI client
- Zip-based server distribution

## Prerequisites

- Rust 1.70+ (with Rust 2024 edition support)
- PostgreSQL database
- Cargo

## Installation

```bash
# Clone the repository
git clone https://github.com/MinecraftAtHome/clusterizer.git
cd clusterizer

# Build all workspace members
cargo build --release
```

## Configuration

Create a `.env` file in the server directory:

```env
DATABASE_URL=postgres://user:password@localhost/clusterizer
SERVER_PORT=8080
API_KEY=your-secret-api-key
```

## Usage

### Start the Server

```bash
cd server
cargo run
```

### Register a New Node

```bash
cd cli
cargo run -- register --name "node-1" --server-url "http://localhost:8080"
```

### Run a Node

```bash
cargo run -- run --server-url "http://localhost:8080" --api-key "your-api-key"
```

## Project Structure

```
clusterizer/
├── api/          # API client library
├── cli/          # CLI application
├── client/       # Client utilities
├── common/       # Shared types and utilities
├── server/       # REST API server
└── util/        # Utility functions
```

## License

See individual crate licenses.
