// Copyright 2026 GlobUid Contributors
// SPDX-License-Identifier: Apache-2.0

//! Pluggable storage backend for GlobUid generator.
//!
//! This module provides a storage abstraction that allows different persistence
//! strategies for the ID generator state.

mod file;
mod memory;

pub use file::FileStorage;
pub use memory::MemoryStorage;

use std::error::Error;
use std::future::Future;
use std::pin::Pin;

/// State that needs to be persisted for the ID generator.
#[derive(Debug, Clone, Copy, Default, serde::Serialize, serde::Deserialize)]
pub struct GeneratorState {
    /// The worker/node ID (0-1023)
    pub worker_id: u16,
    /// The last timestamp used for ID generation
    pub last_timestamp: u64,
    /// The last sequence number used in the current millisecond
    pub last_sequence: u64,
}

/// Type alias for storage load result.
pub type StorageLoadResult = Result<GeneratorState, Box<dyn Error + Send + Sync>>;

/// Type alias for storage save result.
pub type StorageSaveResult = Result<(), Box<dyn Error + Send + Sync>>;

/// Storage backend trait for persisting generator state.
///
/// Implementations can choose different persistence strategies:
/// - [`MemoryStorage`]: In-memory storage (default, no persistence)
/// - [`FileStorage`]: File-based persistent storage
/// - Custom implementations for Redis, database, etc.
pub trait Storage: Send + Sync {
    /// Load the generator state from storage.
    fn load(&self) -> Pin<Box<dyn Future<Output = StorageLoadResult> + Send + '_>>;

    /// Save the generator state to storage.
    fn save(
        &self,
        state: GeneratorState,
    ) -> Pin<Box<dyn Future<Output = StorageSaveResult> + Send + '_>>;

    /// Check if the storage is available.
    fn is_available(&self) -> Pin<Box<dyn Future<Output = bool> + Send + '_>> {
        Box::pin(async { true })
    }
}
