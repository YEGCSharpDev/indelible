#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::sync::Arc;

use ind_test_support::{StorageBackedMockRenderer, TestDb};
use ind_worker::context::WorkerServicesBuilder;

async fn recovery_context(db: &TestDb) -> ind_worker::context::RecoveryJobDeps {
    let renderer = Arc::new(StorageBackedMockRenderer::new(db.storage().await));
    WorkerServicesBuilder::new(
        db.pool().clone(),
        renderer,
        None,
        db.bucket().to_string(),
        ind_egress::EgressPolicy::permissive(),
        None,
    )
    .expect("worker services build")
    .with_worker_id("auto-heal-journey")
    .without_email_services()
    .build()
    .recovery_jobs()
}

#[tokio::test]
async fn auto_heal_leases_real_maintenance_once() {
    let db = TestDb::new().await;

    let context = recovery_context(&db).await;

    ind_worker::auto_heal::run_auto_heal_once(&context).await;

    let completed: Vec<(String, bool, bool, bool)> = sqlx::query_as(
        "SELECT task_name, last_completed_at IS NOT NULL, lease_owner IS NULL, \
         next_run_at > now() FROM maintenance_tasks ORDER BY task_name",
    )
    .fetch_all(db.pool())
    .await
    .unwrap();
    assert_eq!(
        completed,
        vec![("integrity.check".into(), true, true, true),]
    );
}
