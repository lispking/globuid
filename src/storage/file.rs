// Copyright 2026 GlobUid Contributors
// SPDX-License-Identifier: Apache-2.0

//! File-based persistent storage backend.

use std::future::Future;
use std::path::PathBuf;
use std::pin::Pin;

use tokio::fs;
use tokio::io::AsyncWriteExt;

use super::{GeneratorState, Storage, StorageLoadResult, StorageSaveResult};

/// File-based persistent storage backend.
///
/// Persists generator state to a file, surviving application restarts.
/// Useful for preventing ID collisions after restarts.
#[derive(Debug)]
pub struct FileStorage {
    path: PathBuf,
}

impl FileStorage {
    /// Create a new file storage with the given file path.
    pub fn new<P: Into<PathBuf>>(path: P) -> Self {
        Self { path: path.into() }
    }

    /// Create a file storage with the default path in the system temp directory.
    pub fn with_default_path() -> Self {
        let path = std::env::temp_dir().join("globuid_state.json");
        Self::new(path)
    }
}

impl Storage for FileStorage {
    fn load(&self) -> Pin<Box<dyn Future<Output = StorageLoadResult> + Send + '_>> {
        Box::pin(async {
            match fs::read(&self.path).await {
                Ok(data) => {
                    let state: GeneratorState = serde_json::from_slice(&data)?;
                    Ok(state)
                }
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(GeneratorState::default()),
                Err(e) => Err(Box::new(e) as _),
            }
        })
    }

    fn save(
        &self,
        state: GeneratorState,
    ) -> Pin<Box<dyn Future<Output = StorageSaveResult> + Send + '_>> {
        Box::pin(async move {
            let data = serde_json::to_vec_pretty(&state)?;
            let mut file = fs::File::create(&self.path).await?;
            file.write_all(&data).await?;
            file.sync_all().await?;
            Ok(())
        })
    }

    fn is_available(&self) -> Pin<Box<dyn Future<Output = bool> + Send + '_>> {
        Box::pin(async { fs::File::create(&self.path).await.is_ok() })
    }
}
