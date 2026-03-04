// Copyright 2026 GlobUid Contributors
// SPDX-License-Identifier: Apache-2.0

//! ID generation algorithms.
//!
//! This module provides multiple ID generation algorithms:
//! - **Snowflake**: 64-bit time-sorted IDs for distributed systems
//! - **ULID**: 128-bit lexicographically sortable IDs
//! - **NanoID**: URL-friendly short unique strings

mod nanoid;
mod snowflake;
mod ulid;

use std::error::Error;
use std::future::Future;
use std::pin::Pin;

pub use nanoid::{NanoId, NanoIdConfig, NanoIdError};
pub use snowflake::{Snowflake, SnowflakeConfig, SnowflakeError};
pub use ulid::{Ulid, UlidConfig, UlidError};

/// The type of ID to generate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum IdType {
    /// 64-bit numeric ID (Snowflake)
    #[default]
    Numeric64,
    /// 128-bit ID as string (ULID)
    String128,
    /// Short URL-friendly string (NanoID)
    ShortString,
}

/// Generated ID wrapper that can hold different ID types.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Id {
    /// 64-bit numeric ID
    Numeric64(u64),
    /// String-based ID (ULID, NanoID, etc.)
    String(String),
}

impl Id {
    /// Get the ID as a string representation.
    pub fn as_string(&self) -> String {
        match self {
            Id::Numeric64(n) => n.to_string(),
            Id::String(s) => s.clone(),
        }
    }

    /// Get the ID as a 64-bit number (if applicable).
    pub fn as_u64(&self) -> Option<u64> {
        match self {
            Id::Numeric64(n) => Some(*n),
            Id::String(_) => None,
        }
    }
}

impl std::fmt::Display for Id {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Id::Numeric64(n) => write!(f, "{}", n),
            Id::String(s) => write!(f, "{}", s),
        }
    }
}

impl From<u64> for Id {
    fn from(n: u64) -> Self {
        Id::Numeric64(n)
    }
}

impl From<String> for Id {
    fn from(s: String) -> Self {
        Id::String(s)
    }
}

impl From<&str> for Id {
    fn from(s: &str) -> Self {
        Id::String(s.to_string())
    }
}

/// Trait for ID generators.
///
/// Implement this trait to create custom ID generation algorithms.
pub trait IdGenerator: Send + Sync {
    /// The error type for this generator.
    type Error: Error + Send + Sync + 'static;

    /// Generate a single unique ID.
    fn generate(&self) -> Pin<Box<dyn Future<Output = Result<Id, Self::Error>> + Send + '_>>;

    /// Generate multiple unique IDs in batch.
    #[allow(clippy::type_complexity)]
    fn generate_batch(
        &self,
        count: usize,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<Id>, Self::Error>> + Send + '_>> {
        Box::pin(async move {
            let mut ids = Vec::with_capacity(count);
            for _ in 0..count {
                ids.push(self.generate().await?);
            }
            Ok(ids)
        })
    }

    /// Get the type of IDs this generator produces.
    fn id_type(&self) -> IdType;
}
