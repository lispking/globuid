// Copyright 2026 GlobUid Contributors
// SPDX-License-Identifier: Apache-2.0

//! GlobUid server binary entry point.

use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;

use clap::Parser;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use globuid::{IdGenerator, MemoryStorage};

#[derive(Debug, Parser)]
#[command(
    name = "globuid",
    version,
    about = "GlobUid - Global Unique ID Service"
)]
struct Args {
    /// ID algorithm: "snowflake", "ulid", "nanoid"
    #[arg(short = 'a', long, default_value = "snowflake")]
    algorithm: String,

    /// Worker ID (0-1023) for Snowflake
    #[arg(short, long, default_value = "0")]
    worker_id: u16,

    /// Port to listen on
    #[arg(short, long, default_value = "8080")]
    port: u16,

    /// Host to bind to
    #[arg(long, default_value = "0.0.0.0")]
    host: String,

    /// Storage backend: "memory" or "file" (only for Snowflake)
    #[arg(short, long, default_value = "memory")]
    storage: String,

    /// File path for file storage
    #[arg(long)]
    storage_path: Option<PathBuf>,

    /// Protocol: "http" or "grpc"
    #[arg(short = 'P', long, default_value = "http")]
    protocol: String,

    /// ID length for NanoID (default: 21)
    #[arg(long, default_value = "21")]
    nanoid_length: usize,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let args = Args::parse();

    // Validate protocol
    if !matches!(args.protocol.as_str(), "http" | "grpc") {
        eprintln!("Unknown protocol: {}. Supported: http, grpc", args.protocol);
        std::process::exit(1);
    }

    let addr: SocketAddr = format!("{}:{}", args.host, args.port).parse()?;

    // Create generator based on algorithm
    match args.algorithm.as_str() {
        "snowflake" => {
            let config = globuid::SnowflakeConfig {
                worker_id: args.worker_id,
                ..Default::default()
            };

            match args.storage.as_str() {
                "memory" => {
                    let storage = Arc::new(MemoryStorage::with_state(globuid::GeneratorState {
                        worker_id: config.worker_id,
                        last_timestamp: 0,
                        last_sequence: 0,
                    }));
                    let generator = Arc::new(globuid::Snowflake::new(config, storage).await?);

                    tracing::info!("Using Snowflake algorithm with memory storage");
                    run_server(generator, addr, &args.protocol).await?;
                }
                "file" => {
                    let path = args.storage_path.unwrap_or_else(|| {
                        std::env::temp_dir().join(format!("globuid_{}.json", args.worker_id))
                    });
                    let storage = Arc::new(globuid::FileStorage::new(&path));
                    let generator = Arc::new(globuid::Snowflake::new(config, storage).await?);

                    tracing::info!("Using Snowflake algorithm with file storage: {:?}", path);
                    run_server(generator, addr, &args.protocol).await?;
                }
                _ => {
                    eprintln!("Unknown storage backend: {}", args.storage);
                    std::process::exit(1);
                }
            }
        }
        "ulid" => {
            let generator = Arc::new(globuid::Ulid::with_default());
            tracing::info!("Using ULID algorithm");
            run_server(generator, addr, &args.protocol).await?;
        }
        "nanoid" => {
            let config = globuid::NanoIdConfig::new().length(args.nanoid_length);
            let generator = Arc::new(globuid::NanoId::new(config));
            tracing::info!("Using NanoID algorithm (length: {})", args.nanoid_length);
            run_server(generator, addr, &args.protocol).await?;
        }
        _ => {
            eprintln!(
                "Unknown algorithm: {}. Supported: snowflake, ulid, nanoid",
                args.algorithm
            );
            std::process::exit(1);
        }
    }

    Ok(())
}

#[cfg(feature = "http")]
async fn run_server_http<G: IdGenerator + 'static>(
    generator: Arc<G>,
    addr: SocketAddr,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    use globuid::http::{serve, ServerState};

    let state = Arc::new(ServerState { generator });
    serve(state, addr).await
}

#[cfg(not(feature = "http"))]
async fn run_server_http<G: IdGenerator + 'static>(
    _generator: Arc<G>,
    _addr: SocketAddr,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    eprintln!("HTTP feature not enabled. Recompile with --features http");
    std::process::exit(1);
}

#[cfg(feature = "grpc")]
async fn run_server_grpc<G: IdGenerator + 'static>(
    generator: Arc<G>,
    addr: SocketAddr,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    use globuid::grpc::serve;

    serve(generator, addr).await
}

#[cfg(not(feature = "grpc"))]
async fn run_server_grpc<G: IdGenerator + 'static>(
    _generator: Arc<G>,
    _addr: SocketAddr,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    eprintln!("gRPC feature not enabled. Recompile with --features grpc");
    std::process::exit(1);
}

async fn run_server<G: IdGenerator + 'static>(
    generator: Arc<G>,
    addr: SocketAddr,
    protocol: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    match protocol {
        "http" => run_server_http(generator, addr).await,
        "grpc" => run_server_grpc(generator, addr).await,
        _ => unreachable!(),
    }
}
