// Copyright 2026 GlobUid Contributors
// SPDX-License-Identifier: Apache-2.0

//! Basic usage example for GlobUid library.

use std::sync::Arc;

use globuid::{IdGenerator, MemoryStorage, NanoId, NanoIdConfig, Snowflake, SnowflakeConfig, Ulid};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== GlobUid - Multiple Algorithm Demo ===\n");

    // 1. Snowflake (64-bit, time-sortable, distributed)
    println!("1. Snowflake (64-bit numeric, distributed):");
    let config = SnowflakeConfig::default();
    let storage = Arc::new(MemoryStorage::new());
    let snowflake = Snowflake::new(config, storage).await?;

    let id = snowflake.generate().await?;
    println!("   Single ID: {}", id);

    let ids = snowflake.generate_batch(5).await?;
    println!(
        "   Batch: {:?}",
        ids.iter().map(|i| i.as_string()).collect::<Vec<_>>()
    );

    // 2. ULID (128-bit, lexicographically sortable)
    println!("\n2. ULID (26 chars, lexicographically sortable):");
    let ulid = Ulid::with_default();

    let id = ulid.generate().await?;
    println!("   Single ID: {}", id);

    let ids = ulid.generate_batch(5).await?;
    println!(
        "   Batch: {:?}",
        ids.iter().map(|i| i.as_string()).collect::<Vec<_>>()
    );

    // 3. NanoID (configurable length, URL-friendly)
    println!("\n3. NanoID (21 chars default, URL-friendly):");
    let nanoid = NanoId::with_default();

    let id = nanoid.generate().await?;
    println!("   Single ID: {}", id);

    let ids = nanoid.generate_batch(5).await?;
    println!(
        "   Batch: {:?}",
        ids.iter().map(|i| i.as_string()).collect::<Vec<_>>()
    );

    // 4. Custom NanoID (shorter length)
    println!("\n4. Custom NanoID (8 chars):");
    let config = NanoIdConfig::new().length(8);
    let short_nanoid = NanoId::new(config);

    let id = short_nanoid.generate().await?;
    println!("   Single ID: {}", id);

    Ok(())
}
