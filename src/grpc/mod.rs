// Copyright 2026 GlobUid Contributors
// SPDX-License-Identifier: Apache-2.0

//! gRPC transport for GlobUid.

mod server;

pub use server::serve;

// Include generated proto code
pub mod proto {
    tonic::include_proto!("globuid");
}
