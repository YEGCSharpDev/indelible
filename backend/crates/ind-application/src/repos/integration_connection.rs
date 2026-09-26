use std::collections::HashMap;

use chrono::{DateTime, Utc};

use crate::error::AppError;
use ind_domain::{IntegrationConnection, IntegrationConnectionId, IntegrationProvider, UserId};

#[async_trait::async_trait]
pub trait IntegrationConnectionRepository: Send + Sync {
    async fn create(
        &self,
        connection: IntegrationConnection,
    ) -> Result<IntegrationConnection, AppError>;

    async fn upsert_by_user_provider(
        &self,
        user_id: UserId,
        provider: IntegrationProvider,
        config: serde_json::Value,
        status: &str,
    ) -> Result<IntegrationConnection, AppError>;

    async fn find_by_id(
        &self,
        user_id: UserId,
        id: IntegrationConnectionId,
    ) -> Result<Option<IntegrationConnection>, AppError>;

    async fn list_by_user(&self, user_id: UserId) -> Result<Vec<IntegrationConnection>, AppError>;

    async fn set_status(
        &self,
        id: IntegrationConnectionId,
        user_id: UserId,
        status: &str,
    ) -> Result<(), AppError>;

    async fn set_last_sync_at(
        &self,
        id: IntegrationConnectionId,
        user_id: UserId,
        at: DateTime<Utc>,
    ) -> Result<(), AppError>;

    async fn set_last_error(
        &self,
        id: IntegrationConnectionId,
        user_id: UserId,
        error: Option<String>,
    ) -> Result<(), AppError>;

    async fn update_config(
        &self,
        id: IntegrationConnectionId,
        user_id: UserId,
        config: serde_json::Value,
    ) -> Result<(), AppError>;

    /// Optimistic-locking variant of `update_config`. Compares the
    /// caller-provided `expected_version` against the current row;
    /// returns `Ok(new_version)` on success or `Err(DomainError::Conflict)`
    /// when the version no longer matches (i.e. someone else's PATCH won
    /// the race). Settings handlers use this so two concurrent PATCHes
    /// targeting different fields can't silently overwrite each other.
    async fn update_config_with_version(
        &self,
        id: IntegrationConnectionId,
        user_id: UserId,
        expected_version: i64,
        config: serde_json::Value,
    ) -> Result<i64, AppError>;

    async fn delete(&self, id: IntegrationConnectionId, user_id: UserId) -> Result<(), AppError>;

    /// Returns counts of integration jobs in the outbox that have not yet
    /// dispatched, grouped by connection id. Backed by a single subquery
    /// against `job_outbox` filtered to integration-namespaced job types.
    /// Connections with no queued jobs are absent from the map.
    async fn count_pending_jobs_per_connection(
        &self,
        user_id: UserId,
    ) -> Result<HashMap<IntegrationConnectionId, u32>, AppError>;
}
