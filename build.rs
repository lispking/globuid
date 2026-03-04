// Copyright 2026 GlobUid Contributors
// SPDX-License-Identifier: Apache-2.0

//! Build script for generating gRPC code.

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Check if grpc feature is enabled via cargo:rustc-cfg
    let is_grpc = std::env::var("CARGO_FEATURE_GRPC").is_ok();

    if is_grpc {
        tonic_build::configure()
            .build_server(true)
            .build_client(false)
            .compile_protos(&["proto/globuid.proto"], &["proto"])?;
    }

    Ok(())
}
