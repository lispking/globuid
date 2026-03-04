// Copyright 2026 GlobUid Contributors
// SPDX-License-Identifier: Apache-2.0

//! HTTP server for the GlobUid service.

use std::net::SocketAddr;
use std::sync::Arc;

use axum::{
    Router,
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::get,
};
use serde::{Deserialize, Serialize};

use crate::generator::{Id, IdGenerator};

/// Query parameters for batch ID generation.
#[derive(Debug, Deserialize)]
pub struct BatchQuery {
    /// Number of IDs to generate (1-10000).
    #[serde(default = "default_count")]
    pub count: usize,
}

fn default_count() -> usize {
    1
}

/// Response for single ID generation.
#[derive(Debug, Serialize)]
pub struct IdResponse {
    pub id: String,
}

/// Response for batch ID generation.
#[derive(Debug, Serialize)]
pub struct BatchIdResponse {
    pub ids: Vec<String>,
    pub count: usize,
}

/// Error response.
#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
}

/// Server state containing the generator.
#[derive(Debug, Clone)]
pub struct ServerState<G: IdGenerator> {
    pub generator: Arc<G>,
}

/// Create the HTTP router for the GlobUid service.
pub fn create_router<G: IdGenerator + 'static>() -> Router<Arc<ServerState<G>>> {
    Router::new()
        .route("/id", get(generate_id::<G>))
        .route("/id/batch", get(generate_batch::<G>))
        .route("/health", get(health_check))
}

/// Generate a single ID.
async fn generate_id<G: IdGenerator>(
    State(state): State<Arc<ServerState<G>>>,
) -> Result<Json<IdResponse>, (StatusCode, Json<ErrorResponse>)> {
    let result: Result<Id, G::Error> = state.generator.generate().await;
    result
        .map(|id| Json(IdResponse { id: id.as_string() }))
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: e.to_string(),
                }),
            )
        })
}

/// Generate multiple IDs in batch.
async fn generate_batch<G: IdGenerator>(
    State(state): State<Arc<ServerState<G>>>,
    Query(query): Query<BatchQuery>,
) -> Result<Json<BatchIdResponse>, (StatusCode, Json<ErrorResponse>)> {
    let count = query.count.clamp(1, 10000);

    let result: Result<Vec<Id>, G::Error> = state.generator.generate_batch(count).await;
    result
        .map(|ids| {
            let ids: Vec<String> = ids.iter().map(|id| id.as_string()).collect();
            let count = ids.len();
            Json(BatchIdResponse { ids, count })
        })
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: e.to_string(),
                }),
            )
        })
}

/// Health check endpoint.
async fn health_check() -> impl IntoResponse {
    Json(serde_json::json!({
        "status": "ok"
    }))
}

/// Start the HTTP server.
pub async fn serve<G: IdGenerator + 'static>(
    state: Arc<ServerState<G>>,
    addr: SocketAddr,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let app = create_router::<G>().with_state(state);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    tracing::info!("HTTP server listening on {}", addr);

    axum::serve(listener, app).await?;

    Ok(())
}
