// Copyright 2026 GlobUid Contributors
// SPDX-License-Identifier: Apache-2.0

//! Snowflake-based unique ID generator.
//!
//! Generates 64-bit IDs with the following structure:
//! ```text
//! | 1 bit sign | 41 bits timestamp | 10 bits worker_id | 12 bits sequence |
//! ```
//!
//! - 41 bits timestamp: ~69 years from epoch
//! - 10 bits worker_id: 1024 nodes
//! - 12 bits sequence: 4096 IDs per millisecond per node

use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use thiserror::Error;
use tokio::sync::Mutex;

use crate::storage::Storage;

use super::{Id, IdGenerator, IdType};
use std::future::Future;
use std::pin::Pin;

/// Errors that can occur during Snowflake ID generation.
#[derive(Debug, Error)]
pub enum SnowflakeError {
    /// Clock moved backwards, which would cause duplicate IDs.
    #[error("Clock moved backwards. Refusing to generate ID.")]
    ClockMovedBackwards,

    /// Storage operation failed.
    #[error("Storage error: {0}")]
    StorageError(#[from] Box<dyn std::error::Error + Send + Sync>),

    /// Worker ID is out of valid range.
    #[error("Worker ID {0} is out of range (0-{MAX_WORKER_ID})", MAX_WORKER_ID = SnowflakeConfig::MAX_WORKER_ID)]
    InvalidWorkerId(u16),
}

/// Configuration for the Snowflake generator.
#[derive(Debug, Clone, Copy)]
pub struct SnowflakeConfig {
    /// Worker/node ID (0-1023). Default: 0
    pub worker_id: u16,
    /// Custom epoch timestamp in milliseconds. Default: 2025-01-01 00:00:00 UTC
    pub epoch: u64,
}

impl Default for SnowflakeConfig {
    fn default() -> Self {
        Self {
            worker_id: 0,
            // 2025-01-01 00:00:00 UTC in milliseconds
            epoch: 1735689600000,
        }
    }
}

impl SnowflakeConfig {
    /// Maximum valid worker ID (10 bits = 1023).
    pub const MAX_WORKER_ID: u16 = 1023;
}

/// Internal state for Snowflake generator.
#[derive(Debug, Clone, Copy, Default)]
struct SnowflakeState {
    last_timestamp: u64,
    last_sequence: u64,
}

/// Snowflake-based unique ID generator.
#[derive(Debug)]
pub struct Snowflake<S: Storage> {
    config: SnowflakeConfig,
    storage: Arc<S>,
    state: Mutex<SnowflakeState>,
}

impl<S: Storage> Snowflake<S> {
    /// Bits allocated for sequence number.
    const SEQUENCE_BITS: u8 = 12;
    /// Bits allocated for worker ID.
    const WORKER_ID_BITS: u8 = 10;

    /// Maximum sequence number (12 bits = 4095).
    const MAX_SEQUENCE: u64 = (1 << Self::SEQUENCE_BITS) - 1;
    /// Worker ID shift position.
    const WORKER_ID_SHIFT: u8 = Self::SEQUENCE_BITS;
    /// Timestamp shift position.
    const TIMESTAMP_SHIFT: u8 = Self::SEQUENCE_BITS + Self::WORKER_ID_BITS;

    /// Create a new Snowflake generator.
    pub async fn new(config: SnowflakeConfig, storage: Arc<S>) -> Result<Self, SnowflakeError> {
        if config.worker_id > SnowflakeConfig::MAX_WORKER_ID {
            return Err(SnowflakeError::InvalidWorkerId(config.worker_id));
        }

        let stored = storage.load().await?;
        let initial_state = SnowflakeState {
            last_timestamp: stored.last_timestamp,
            last_sequence: stored.last_sequence,
        };

        Ok(Self {
            config,
            storage,
            state: Mutex::new(initial_state),
        })
    }

    /// Generate a new unique ID as u64.
    pub async fn generate_u64(&self) -> Result<u64, SnowflakeError> {
        let mut state = self.state.lock().await;
        let current_timestamp = self.current_timestamp()?;

        let (timestamp, sequence) = if current_timestamp < state.last_timestamp {
            return Err(SnowflakeError::ClockMovedBackwards);
        } else if current_timestamp == state.last_timestamp {
            let sequence = state.last_sequence + 1;
            if sequence > Self::MAX_SEQUENCE {
                let timestamp = self.wait_for_next_millis(state.last_timestamp).await;
                (timestamp, 0)
            } else {
                (current_timestamp, sequence)
            }
        } else {
            (current_timestamp, 0)
        };

        let id = ((timestamp - self.config.epoch) << Self::TIMESTAMP_SHIFT)
            | (self.config.worker_id as u64) << Self::WORKER_ID_SHIFT
            | sequence;

        state.last_timestamp = timestamp;
        state.last_sequence = sequence;
        self.storage
            .save(crate::storage::GeneratorState {
                worker_id: self.config.worker_id,
                last_timestamp: timestamp,
                last_sequence: sequence,
            })
            .await?;

        Ok(id)
    }

    fn current_timestamp(&self) -> Result<u64, SnowflakeError> {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .map_err(|_| SnowflakeError::ClockMovedBackwards)
    }

    async fn wait_for_next_millis(&self, last_timestamp: u64) -> u64 {
        let mut timestamp = self.current_timestamp().unwrap_or(last_timestamp);
        while timestamp <= last_timestamp {
            tokio::time::sleep(std::time::Duration::from_millis(1)).await;
            timestamp = self.current_timestamp().unwrap_or(last_timestamp);
        }
        timestamp
    }
}

impl<S: Storage + 'static> IdGenerator for Snowflake<S> {
    type Error = SnowflakeError;

    fn generate(&self) -> Pin<Box<dyn Future<Output = Result<Id, Self::Error>> + Send + '_>> {
        Box::pin(async move {
            let id = self.generate_u64().await?;
            Ok(Id::Numeric64(id))
        })
    }

    fn id_type(&self) -> IdType {
        IdType::Numeric64
    }
}

impl Snowflake<crate::storage::MemoryStorage> {
    /// Create a Snowflake generator with default configuration and memory storage.
    pub fn with_default() -> Self {
        let config = SnowflakeConfig::default();
        let storage = Arc::new(crate::storage::MemoryStorage::with_state(
            crate::storage::GeneratorState {
                worker_id: config.worker_id,
                last_timestamp: 0,
                last_sequence: 0,
            },
        ));

        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current()
                .block_on(async { Self::new(config, storage).await.unwrap() })
        })
    }
}
