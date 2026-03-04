// Copyright 2026 GlobUid Contributors
// SPDX-License-Identifier: Apache-2.0

//! NanoID generator.
//!
//! NanoIDs are URL-friendly unique string identifiers with configurable:
//! - Length (default: 21 characters)
//! - Alphabet (default: A-Za-z0-9_-)
//!
//! Example: `V1StGXR8_Z5jdHi6B-myT`

use std::sync::Arc;

use thiserror::Error;

use super::{Id, IdGenerator, IdType};
use std::future::Future;
use std::pin::Pin;

/// Errors that can occur during NanoID generation.
#[derive(Debug, Error)]
pub enum NanoIdError {
    /// Random generation failed.
    #[error("Failed to generate random bytes: {0}")]
    RandomError(String),
}

/// Default alphabet for NanoID.
pub const DEFAULT_ALPHABET: &[u8] =
    b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789-_";

/// Configuration for the NanoID generator.
#[derive(Debug, Clone)]
pub struct NanoIdConfig {
    /// Length of the generated ID. Default: 21
    pub length: usize,
    /// Custom alphabet. Default: A-Za-z0-9_-
    pub alphabet: Arc<[u8]>,
}

impl Default for NanoIdConfig {
    fn default() -> Self {
        Self {
            length: 21,
            alphabet: Arc::from(DEFAULT_ALPHABET),
        }
    }
}

impl NanoIdConfig {
    /// Create a new NanoID configuration.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the ID length.
    pub fn length(mut self, length: usize) -> Self {
        self.length = length;
        self
    }

    /// Set a custom alphabet.
    pub fn alphabet(mut self, alphabet: &[u8]) -> Self {
        self.alphabet = Arc::from(alphabet.to_vec().into_boxed_slice());
        self
    }
}

/// NanoID generator.
#[derive(Debug, Clone)]
pub struct NanoId {
    config: NanoIdConfig,
    mask: u8,
    step: usize,
}

impl NanoId {
    /// Create a new NanoID generator.
    pub fn new(config: NanoIdConfig) -> Self {
        let alphabet_len = config.alphabet.len();

        // Calculate mask for efficient random sampling
        // Find the smallest (2^n - 1) >= alphabet_len - 1
        let mask = {
            let mut m = 1u8;
            while m < alphabet_len as u8 - 1 {
                m = m * 2 + 1;
            }
            m
        };

        // Calculate step (how many bytes we need per generation)
        let step = (config.length * 8)
            .div_ceil(mask.count_ones() as usize)
            .max(1);

        Self { config, mask, step }
    }

    /// Create a NanoID generator with default configuration.
    pub fn with_default() -> Self {
        Self::new(NanoIdConfig::default())
    }

    /// Generate a new NanoID string.
    pub fn generate_string(&self) -> Result<String, NanoIdError> {
        let mut result = String::with_capacity(self.config.length);
        let alphabet = &self.config.alphabet;
        let mut bytes = vec![0u8; self.step * 2]; // Extra buffer

        while result.len() < self.config.length {
            getrandom::fill(&mut bytes).map_err(|e| NanoIdError::RandomError(e.to_string()))?;

            for &byte in &bytes {
                let index = byte & self.mask;
                if (index as usize) < alphabet.len() {
                    result.push(alphabet[index as usize] as char);
                    if result.len() >= self.config.length {
                        break;
                    }
                }
            }
        }

        Ok(result)
    }
}

impl IdGenerator for NanoId {
    type Error = NanoIdError;

    fn generate(&self) -> Pin<Box<dyn Future<Output = Result<Id, Self::Error>> + Send + '_>> {
        Box::pin(async move {
            let s = self.generate_string()?;
            Ok(Id::String(s))
        })
    }

    fn id_type(&self) -> IdType {
        IdType::ShortString
    }
}
