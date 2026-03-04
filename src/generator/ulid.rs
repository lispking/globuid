// Copyright 2026 GlobUid Contributors
// SPDX-License-Identifier: Apache-2.0

//! ULID (Universally Unique Lexicographically Sortable Identifier) generator.
//!
//! ULIDs are 128-bit identifiers that are:
//! - Lexicographically sortable
//! - Case-insensitive
//! - URL-safe (no special characters)
//! - 26 characters long (Base32 encoded)
//!
//! Structure:
//! ```text
//! | 48 bits timestamp | 80 bits randomness |
//! ```

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use thiserror::Error;

use super::{Id, IdGenerator, IdType};
use std::future::Future;
use std::pin::Pin;

/// Errors that can occur during ULID generation.
#[derive(Debug, Error)]
pub enum UlidError {
    /// Clock moved backwards.
    #[error("Clock moved backwards. Refusing to generate ID.")]
    ClockMovedBackwards,

    /// Entropy generation failed.
    #[error("Failed to generate entropy: {0}")]
    EntropyError(String),
}

/// Configuration for the ULID generator.
#[derive(Debug, Clone, Copy)]
pub struct UlidConfig {
    /// Use monotonic increment for same-millisecond IDs.
    pub monotonic: bool,
}

impl Default for UlidConfig {
    fn default() -> Self {
        Self { monotonic: true }
    }
}

/// ULID generator.
#[derive(Debug)]
pub struct Ulid {
    config: UlidConfig,
    last_timestamp: AtomicU64,
    last_random: AtomicU64,
}

impl Ulid {
    /// Create a new ULID generator.
    pub fn new(config: UlidConfig) -> Self {
        Self {
            config,
            last_timestamp: AtomicU64::new(0),
            last_random: AtomicU64::new(0),
        }
    }

    /// Create a ULID generator with default configuration.
    pub fn with_default() -> Self {
        Self::new(UlidConfig::default())
    }

    /// Generate a new ULID string.
    pub fn generate_string(&self) -> Result<String, UlidError> {
        let bytes = self.generate_bytes()?;
        Ok(encode_ulid(&bytes))
    }

    /// Generate a new ULID as 16 bytes.
    pub fn generate_bytes(&self) -> Result<[u8; 16], UlidError> {
        let timestamp = self.current_timestamp()?;

        let (timestamp, random) = if self.config.monotonic {
            let last_ts = self.last_timestamp.load(Ordering::SeqCst);

            if timestamp < last_ts {
                return Err(UlidError::ClockMovedBackwards);
            }

            let mut random_bytes = [0u8; 10];
            getrandom::fill(&mut random_bytes)
                .map_err(|e| UlidError::EntropyError(e.to_string()))?;

            if timestamp == last_ts {
                // Increment the random part for monotonicity
                let last_rand = self.last_random.load(Ordering::SeqCst);
                let new_rand = last_rand.wrapping_add(1);
                self.last_random.store(new_rand, Ordering::SeqCst);

                // Convert new_rand back to bytes for the first 8 bytes of random
                let rand_bytes = new_rand.to_be_bytes();
                random_bytes[0..8].copy_from_slice(&rand_bytes);
            } else {
                self.last_timestamp.store(timestamp, Ordering::SeqCst);
                // Store first 8 bytes of random for potential monotonic increment
                let mut rand_val_bytes = [0u8; 8];
                rand_val_bytes.copy_from_slice(&random_bytes[0..8]);
                let rand_val = u64::from_be_bytes(rand_val_bytes);
                self.last_random.store(rand_val, Ordering::SeqCst);
            }

            (timestamp, random_bytes)
        } else {
            let mut random_bytes = [0u8; 10];
            getrandom::fill(&mut random_bytes)
                .map_err(|e| UlidError::EntropyError(e.to_string()))?;
            (timestamp, random_bytes)
        };

        // Combine timestamp and random into 16 bytes
        // ULID structure: 48 bits timestamp + 80 bits random
        let mut bytes = [0u8; 16];

        // Timestamp: 48 bits (6 bytes) - most significant bytes of the 64-bit timestamp
        let ts_bytes = timestamp.to_be_bytes();
        bytes[0..6].copy_from_slice(&ts_bytes[2..8]); // Take last 6 bytes (48 bits)

        // Random: 80 bits (10 bytes)
        bytes[6..16].copy_from_slice(&random);

        Ok(bytes)
    }

    fn current_timestamp(&self) -> Result<u64, UlidError> {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .map_err(|_| UlidError::ClockMovedBackwards)
    }
}

impl IdGenerator for Ulid {
    type Error = UlidError;

    fn generate(&self) -> Pin<Box<dyn Future<Output = Result<Id, Self::Error>> + Send + '_>> {
        Box::pin(async move {
            let s = self.generate_string()?;
            Ok(Id::String(s))
        })
    }

    fn id_type(&self) -> IdType {
        IdType::String128
    }
}

/// Encode ULID bytes to Base32 string.
fn encode_ulid(bytes: &[u8; 16]) -> String {
    const ALPHABET: &[u8; 32] = b"0123456789ABCDEFGHJKMNPQRSTVWXYZ";

    // ULID: 128 bits = 16 bytes
    // Encoded as 26 characters in Base32 (each char = 5 bits)
    // Total: 26 * 5 = 130 bits, but we only use 128 bits

    let mut result = String::with_capacity(26);

    // Process bytes in a stream, extracting 5 bits at a time
    // 16 bytes = 128 bits = 25.6 * 5 bits, so we get 26 characters with padding

    let mut buffer: u128 = 0;
    let mut bits_in_buffer = 0;
    let mut byte_idx = 0;

    for _ in 0..26 {
        // Ensure we have at least 5 bits in the buffer
        while bits_in_buffer < 5 && byte_idx < 16 {
            buffer = (buffer << 8) | (bytes[byte_idx] as u128);
            bits_in_buffer += 8;
            byte_idx += 1;
        }

        // Extract 5 bits
        if bits_in_buffer >= 5 {
            bits_in_buffer -= 5;
            let index = ((buffer >> bits_in_buffer) & 0x1F) as usize;
            result.push(ALPHABET[index] as char);
        } else {
            // Use remaining bits (shouldn't happen with valid ULID)
            let index = ((buffer << (5 - bits_in_buffer)) & 0x1F) as usize;
            result.push(ALPHABET[index] as char);
        }
    }

    result
}
