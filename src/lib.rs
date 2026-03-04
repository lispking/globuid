// Copyright 2026 GlobUid Contributors
// SPDX-License-Identifier: Apache-2.0

//! GlobUid - A globally unique ID generator with pluggable algorithms and transport layer.
//!
//! # Features
//!
//! - Multiple ID algorithms: Snowflake, ULID, NanoID
//! - Distributed support with worker IDs (Snowflake)
//! - Pluggable storage backends (memory, file, or custom)
//! - Optional HTTP/gRPC transport layer
//!
//! # Algorithms
//!
//! | Algorithm | Output | Length | Sortable | Use Case |
//! |-----------|--------|--------|----------|----------|
//! | Snowflake | u64 | 64-bit | Time-sortable | Distributed systems |
//! | ULID | String | 26 chars | Lex-sortable | URLs, databases |
//! | NanoID | String | 21 chars | No | URLs, short identifiers |
//!
//! # Quick Start (Library)
//!
//! ```rust,no_run
//! use globuid::{Snowflake, SnowflakeConfig, MemoryStorage, IdGenerator};
//! use std::sync::Arc;
//!
//! #[tokio::main]
//! async fn main() {
//!     // Snowflake (64-bit, distributed)
//!     let config = SnowflakeConfig::default();
//!     let storage = Arc::new(MemoryStorage::new());
//!     let generator = Snowflake::new(config, storage).await.unwrap();
//!     let id = generator.generate().await.unwrap();
//!     println!("Snowflake ID: {}", id);
//!     
//!     // ULID (128-bit, lexicographically sortable)
//!     let ulid = globuid::Ulid::with_default();
//!     let id = ulid.generate().await.unwrap();
//!     println!("ULID: {}", id);
//!     
//!     // NanoID (short URL-friendly)
//!     let nanoid = globuid::NanoId::with_default();
//!     let id = nanoid.generate().await.unwrap();
//!     println!("NanoID: {}", id);
//! }
//! ```
//!
//! # Transport Layers (optional features)
//!
//! - `http`: Enable HTTP REST API server
//! - `grpc`: Enable gRPC server
//! - `full`: Enable all transport layers

pub mod generator;
pub mod storage;

// Re-exports for convenience
pub use generator::{
    Id, IdGenerator, IdType, NanoId, NanoIdConfig, Snowflake, SnowflakeConfig, SnowflakeError,
    Ulid, UlidConfig,
};
pub use storage::{FileStorage, GeneratorState, MemoryStorage, Storage};

// Backward compatibility aliases
pub type Generator<S> = Snowflake<S>;
pub type GeneratorConfig = SnowflakeConfig;
pub type GeneratorError = SnowflakeError;
pub type DefaultGenerator = Snowflake<MemoryStorage>;

#[cfg(feature = "http")]
pub mod http;

#[cfg(feature = "grpc")]
pub mod grpc;
