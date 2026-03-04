// Copyright 2026 GlobUid Contributors
// SPDX-License-Identifier: Apache-2.0

//! gRPC server implementation for GlobUid.

use std::sync::Arc;

use tonic::{Request, Response, Status, transport::Server};

use crate::generator::{Id, IdGenerator};

use super::proto::{
    GenerateBatchRequest, GenerateBatchResponse, GenerateRequest, GenerateResponse, HealthRequest,
    HealthResponse,
    glob_uid_server::{GlobUid, GlobUidServer},
};

/// gRPC service implementation.
pub struct GlobUidService<G: IdGenerator> {
    generator: Arc<G>,
}

impl<G: IdGenerator> GlobUidService<G> {
    /// Create a new gRPC service with the given generator.
    pub fn new(generator: Arc<G>) -> Self {
        Self { generator }
    }
}

#[tonic::async_trait]
impl<G: IdGenerator + 'static> GlobUid for GlobUidService<G> {
    async fn generate(
        &self,
        _request: Request<GenerateRequest>,
    ) -> Result<Response<GenerateResponse>, Status> {
        let result: Result<Id, G::Error> = self.generator.generate().await;
        result
            .map(|id| Response::new(GenerateResponse { id: id.as_string() }))
            .map_err(|e| Status::internal(e.to_string()))
    }

    async fn generate_batch(
        &self,
        request: Request<GenerateBatchRequest>,
    ) -> Result<Response<GenerateBatchResponse>, Status> {
        let count = request.into_inner().count as usize;
        let count = count.clamp(1, 10000);

        let result: Result<Vec<Id>, G::Error> = self.generator.generate_batch(count).await;
        result
            .map(|ids| {
                let ids: Vec<String> = ids.iter().map(|id| id.as_string()).collect();
                let count = ids.len() as u32;
                Response::new(GenerateBatchResponse { ids, count })
            })
            .map_err(|e| Status::internal(e.to_string()))
    }

    async fn health(
        &self,
        _request: Request<HealthRequest>,
    ) -> Result<Response<HealthResponse>, Status> {
        Ok(Response::new(HealthResponse {
            status: "ok".to_string(),
        }))
    }
}

/// Start the gRPC server.
pub async fn serve<G: IdGenerator + 'static>(
    generator: Arc<G>,
    addr: std::net::SocketAddr,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let service = GlobUidService::new(generator);

    tracing::info!("gRPC server listening on {}", addr);

    Server::builder()
        .add_service(GlobUidServer::new(service))
        .serve(addr)
        .await?;

    Ok(())
}
