// Copyright 2026 GlobUid Contributors
// SPDX-License-Identifier: Apache-2.0

//! In-memory storage backend.

use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use tokio::sync::RwLock;

use super::{GeneratorState, Storage, StorageLoadResult, StorageSaveResult};

/// In-memory storage backend.
///
/// This storage does not persist data across restarts.
/// Suitable for single-instance deployments where worker ID is statically configured.
#[derive(Debug, Default)]
pub struct MemoryStorage {
    state: Arc<RwLock<GeneratorState>>,
}

impl MemoryStorage {
    /// Create a new in-memory storage with default state.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a new in-memory storage with the given initial state.
    pub fn with_state(state: GeneratorState) -> Self {
        Self {
            state: Arc::new(RwLock::new(state)),
        }
    }
}

impl Storage for MemoryStorage {
    fn load(&self) -> Pin<Box<dyn Future<Output = StorageLoadResult> + Send + '_>> {
        Box::pin(async {
            let state = self.state.read().await;
            Ok(*state)
        })
    }

    fn save(
        &self,
        state: GeneratorState,
    ) -> Pin<Box<dyn Future<Output = StorageSaveResult> + Send + '_>> {
        Box::pin(async move {
            let mut current = self.state.write().await;
            *current = state;
            Ok(())
        })
    }
}
