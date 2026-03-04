// Copyright 2026 GlobUid Contributors
// SPDX-License-Identifier: Apache-2.0

//! HTTP REST API transport for GlobUid.

mod server;

pub use server::{serve, BatchIdResponse, IdResponse, ServerState};
