# GlobUid

A globally unique ID generator with pluggable algorithms and transport layer, written in Rust.

[![Crates.io](https://img.shields.io/crates/v/globuid.svg)](https://crates.io/crates/globuid)
[![Documentation](https://docs.rs/globuid/badge.svg)](https://docs.rs/globuid)
[![License: Apache 2.0](https://img.shields.io/badge/License-Apache2.0-yellow.svg)](https://opensource.org/license/apache-2-0)
[<img alt="build status" src="https://img.shields.io/github/actions/workflow/status/lispking/globuid/ci.yml?branch=main&style=for-the-badge" height="20">](https://github.com/lispking/globuid/actions?query=branch%3Amain)
[![Ask DeepWiki](https://deepwiki.com/badge.svg)](https://deepwiki.com/lispking/globuid)

## Features

- **Multiple ID Algorithms**: Snowflake, ULID, NanoID
- **Pluggable Storage**: Memory, File, or implement your own
- **Optional Transport Layer**: HTTP REST API or gRPC
- **High Performance**: Async/await with Tokio runtime
- **Distributed Support**: Worker ID support for Snowflake algorithm
- **Zero Dependencies Core**: Use as a library without HTTP/gRPC overhead

## Algorithms Comparison

| Algorithm | Output | Length | Sortable | Distributed | Use Case |
|-----------|--------|--------|----------|-------------|----------|
| **Snowflake** | `u64` | 64-bit | Time-sortable | ✓ (1024 nodes) | Distributed systems, databases |
| **ULID** | `String` | 26 chars | Lexicographically | ✗ | URLs, distributed databases |
| **NanoID** | `String` | Configurable | ✗ | ✗ | Short URLs, session IDs |

### Snowflake

64-bit unique ID with the following structure:

```
| 1 bit sign | 41 bits timestamp | 10 bits worker_id | 12 bits sequence |
```

- **41 bits timestamp**: ~69 years from custom epoch
- **10 bits worker_id**: 1024 nodes maximum
- **12 bits sequence**: 4096 IDs per millisecond per node

### ULID

Universally Unique Lexicographically Sortable Identifier:

```
| 48 bits timestamp | 80 bits randomness |
```

- 26 characters, Base32 encoded
- Case-insensitive
- URL-safe
- Monotonic increment support

### NanoID

URL-friendly unique string identifier:

- Default: 21 characters
- Customizable length and alphabet
- URL-safe characters: `A-Za-z0-9_-`

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
globuid = "0.1.0"

# With HTTP support
globuid = { version = "0.1.0", features = ["http"] }

# With gRPC support
globuid = { version = "0.1.0", features = ["grpc"] }

# With all features
globuid = { version = "0.1.0", features = ["full"] }
```

## Usage

### As a Library

#### Snowflake (Distributed Systems)

```rust
use globuid::{Snowflake, SnowflakeConfig, MemoryStorage, IdGenerator};
use std::sync::Arc;

#[tokio::main]
async fn main() {
    let config = SnowflakeConfig {
        worker_id: 1,  // Unique worker ID (0-1023)
        ..Default::default()
    };
    
    let storage = Arc::new(MemoryStorage::new());
    let generator = Snowflake::new(config, storage).await.unwrap();
    
    // Generate single ID
    let id = generator.generate().await.unwrap();
    println!("ID: {}", id);
    
    // Generate batch
    let ids = generator.generate_batch(10).await.unwrap();
    for id in ids {
        println!("ID: {}", id);
    }
}
```

#### ULID (Lexicographically Sortable)

```rust
use globuid::{Ulid, IdGenerator};

#[tokio::main]
async fn main() {
    let generator = Ulid::with_default();
    
    let id = generator.generate().await.unwrap();
    println!("ULID: {}", id);  // e.g., "01ARZ3NDEKTSV4RRFFQ69G5FAV"
}
```

#### NanoID (Short URLs)

```rust
use globuid::{NanoId, NanoIdConfig, IdGenerator};

#[tokio::main]
async fn main() {
    // Default: 21 characters
    let generator = NanoId::with_default();
    let id = generator.generate().await.unwrap();
    println!("NanoID: {}", id);  // e.g., "V1StGXR8_Z5jdHi6B-myT"
    
    // Custom length
    let config = NanoIdConfig::new().length(10);
    let short_generator = NanoId::new(config);
    let id = short_generator.generate().await.unwrap();
    println!("Short ID: {}", id);  // e.g., "IRFa-VaH2b"
}
```

### As a Server

#### HTTP REST API

```bash
# Start HTTP server with Snowflake
cargo run --features http -- --algorithm snowflake --port 8080

# With ULID
cargo run --features http -- --algorithm ulid --port 8080

# With NanoID (custom length)
cargo run --features http -- --algorithm nanoid --nanoid-length 10 --port 8080
```

**API Endpoints:**

| Method | Endpoint | Description |
|--------|----------|-------------|
| `GET` | `/health` | Health check |
| `GET` | `/id` | Generate single ID |
| `GET` | `/id/batch?count=N` | Generate N IDs (max 10000) |

**Example:**

```bash
# Health check
curl http://localhost:8080/health
# {"status":"ok"}

# Single ID
curl http://localhost:8080/id
# {"id":"287628074806149120"}

# Batch IDs
curl "http://localhost:8080/id/batch?count=5"
# {"ids":["...","..."],"count":5}
```

#### gRPC

```bash
# Start gRPC server
cargo run --features grpc -- --algorithm ulid --protocol grpc --port 9090
```

**Proto Definition:**

```protobuf
service GlobUid {
    rpc Generate(GenerateRequest) returns (GenerateResponse);
    rpc GenerateBatch(GenerateBatchRequest) returns (GenerateBatchResponse);
    rpc Health(HealthRequest) returns (HealthResponse);
}
```

## Storage Backends

### Memory Storage (Default)

Non-persistent in-memory storage. Suitable for single-instance deployments.

```rust
use globuid::MemoryStorage;
use std::sync::Arc;

let storage = Arc::new(MemoryStorage::new());
```

### File Storage

Persistent file-based storage. Survives application restarts.

```rust
use globuid::FileStorage;
use std::sync::Arc;

let storage = Arc::new(FileStorage::new("/path/to/state.json"));
```

### Custom Storage

Implement the `Storage` trait for custom backends (Redis, PostgreSQL, etc.):

```rust
use globuid::{Storage, GeneratorState};
use std::pin::Pin;
use std::future::Future;

struct RedisStorage { /* ... */ }

impl Storage for RedisStorage {
    fn load(&self) -> Pin<Box<dyn Future<Output = Result<GeneratorState, Box<dyn std::error::Error + Send + Sync>>> + Send + '_>> {
        // Load state from Redis
    }
    
    fn save(&self, state: GeneratorState) -> Pin<Box<dyn Future<Output = Result<(), Box<dyn std::error::Error + Send + Sync>>> + Send + '_>> {
        // Save state to Redis
    }
}
```

## CLI Options

```
GlobUid - Global Unique ID Service

Usage: globuid [OPTIONS]

Options:
  -a, --algorithm <ALGORITHM>  ID algorithm: "snowflake", "ulid", "nanoid" [default: snowflake]
  -w, --worker-id <WORKER_ID>  Worker ID (0-1023) for Snowflake [default: 0]
  -p, --port <PORT>            Port to listen on [default: 8080]
      --host <HOST>            Host to bind to [default: 0.0.0.0]
  -s, --storage <STORAGE>      Storage backend: "memory" or "file" [default: memory]
      --storage-path <PATH>    File path for file storage
  -P, --protocol <PROTOCOL>    Protocol: "http" or "grpc" [default: http]
      --nanoid-length <LEN>    ID length for NanoID [default: 21]
  -h, --help                   Print help
  -V, --version                Print version
```

## Feature Flags

| Feature | Description |
|---------|-------------|
| `http` | Enable HTTP REST API server |
| `grpc` | Enable gRPC server |
| `full` | Enable all transport layers |

## Performance

Benchmarks on Apple M1 (single thread):

| Algorithm | Ops/sec |
|-----------|---------|
| Snowflake | ~2M/sec |
| ULID | ~1.5M/sec |
| NanoID (21 chars) | ~800K/sec |

## License

Licensed under the Apache License, Version 2.0. See [LICENSE](LICENSE) for details.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.
