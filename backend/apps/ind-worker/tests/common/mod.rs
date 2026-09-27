#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::sync::Arc;

use ind_test_support::{StorageBackedMockRenderer, TestDb};
use ind_worker::context::{WorkerContext, WorkerServicesBuilder};

// Shared across worker test binaries; not every binary uses both constructors.
#[allow(dead_code)]
pub async fn build_worker_ctx(db: &TestDb) -> WorkerContext {
    build_worker_ctx_with_renderer(db).await.0
}

/// Like `build_worker_ctx` but also returns the mock renderer handle so tests can queue
/// success/failure/partial render results.
pub async fn build_worker_ctx_with_renderer(
    db: &TestDb,
) -> (WorkerContext, Arc<StorageBackedMockRenderer>) {
    let storage = db.storage().await;
    let renderer = Arc::new(StorageBackedMockRenderer::new(storage.clone()));
    let ctx = WorkerServicesBuilder::new(
        db.pool().clone(),
        renderer.clone(),
        Some(storage),
        db.bucket().to_string(),
        ind_egress::EgressPolicy::permissive(),
        None,
    )
    .expect("worker services build")
    .with_worker_id("test-worker")
    .build();

    (ctx, renderer)
}
