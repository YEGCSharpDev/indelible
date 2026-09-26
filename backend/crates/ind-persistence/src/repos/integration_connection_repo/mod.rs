use std::collections::HashMap;
use std::str::FromStr;

use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use ind_application::AppError;
use ind_application::repos::integration_connection::IntegrationConnectionRepository;
use ind_domain::{
    DomainError, IntegrationConnection, IntegrationConnectionId, IntegrationProvider, UserId,
};

mod model;

use model::{ConnectionRow, provider_to_str};

pub struct PgIntegrationConnectionRepository {
    pool: PgPool,
}

impl PgIntegrationConnectionRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

pub(super) fn map_err(err: sqlx::Error) -> AppError {
    super::map_sqlx_error(
        "integration_connection",
        "integration connection already exists",
        err,
    )
}

#[async_trait::async_trait]
impl IntegrationConnectionRepository for PgIntegrationConnectionRepository {
    async fn create(
        &self,
        connection: IntegrationConnection,
    ) -> Result<IntegrationConnection, AppError> {
        let row = sqlx::query_as!(
            ConnectionRow,
            r#"INSERT INTO integration_connections
                (id, user_id, provider, config, status, last_sync_at, last_error, created_at, updated_at, version)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
               RETURNING id, user_id, provider, config, status, last_sync_at, last_error, created_at, updated_at, version"#,
            connection.id.into_uuid(),
            connection.user_id.into_uuid(),
            provider_to_str(connection.provider),
            connection.config,
            connection.status,
            connection.last_sync_at,
            connection.last_error,
            connection.created_at,
            connection.updated_at,
            connection.version,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(map_err)?;

        IntegrationConnection::try_from(row)
    }

    async fn upsert_by_user_provider(
        &self,
        user_id: UserId,
        provider: IntegrationProvider,
        config: serde_json::Value,
        status: &str,
    ) -> Result<IntegrationConnection, AppError> {
        let now = Utc::now();
        let id = Uuid::now_v7();
        let row = sqlx::query_as!(
            ConnectionRow,
            r#"INSERT INTO integration_connections
                (id, user_id, provider, config, status, created_at, updated_at)
               VALUES ($1, $2, $3, $4, $5, $6, $6)
               ON CONFLICT (user_id, provider) DO UPDATE
                 SET config = integration_connections.config || EXCLUDED.config,
                     status = EXCLUDED.status,
                     updated_at = EXCLUDED.updated_at,
                     last_error = NULL,
                     version = integration_connections.version + 1
               RETURNING id, user_id, provider, config, status, last_sync_at, last_error, created_at, updated_at, version"#,
            id,
            user_id.into_uuid(),
            provider_to_str(provider),
            config,
            status,
            now,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(map_err)?;

        IntegrationConnection::try_from(row)
    }

    async fn find_by_id(
        &self,
        user_id: UserId,
        id: IntegrationConnectionId,
    ) -> Result<Option<IntegrationConnection>, AppError> {
        let row = sqlx::query_as!(
            ConnectionRow,
            r#"SELECT id, user_id, provider, config, status, last_sync_at, last_error, created_at, updated_at, version
               FROM integration_connections
               WHERE id = $1 AND user_id = $2"#,
            id.into_uuid(),
            user_id.into_uuid(),
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(map_err)?;

        row.map(IntegrationConnection::try_from).transpose()
    }

    async fn list_by_user(&self, user_id: UserId) -> Result<Vec<IntegrationConnection>, AppError> {
        let rows = sqlx::query_as!(
            ConnectionRow,
            r#"SELECT id, user_id, provider, config, status, last_sync_at, last_error, created_at, updated_at, version
               FROM integration_connections
               WHERE user_id = $1
               ORDER BY created_at ASC"#,
            user_id.into_uuid(),
        )
        .fetch_all(&self.pool)
        .await
        .map_err(map_err)?;

        rows.into_iter()
            .map(IntegrationConnection::try_from)
            .collect()
    }

    async fn set_status(
        &self,
        id: IntegrationConnectionId,
        user_id: UserId,
        status: &str,
    ) -> Result<(), AppError> {
        let result = sqlx::query!(
            r#"UPDATE integration_connections
               SET status = $3, updated_at = now()
               WHERE id = $1 AND user_id = $2"#,
            id.into_uuid(),
            user_id.into_uuid(),
            status,
        )
        .execute(&self.pool)
        .await
        .map_err(map_err)?;

        if result.rows_affected() == 0 {
            return Err(AppError::Domain(DomainError::NotFound {
                entity: "integration_connection",
                id: id.to_string(),
            }));
        }
        Ok(())
    }

    async fn set_last_sync_at(
        &self,
        id: IntegrationConnectionId,
        user_id: UserId,
        at: DateTime<Utc>,
    ) -> Result<(), AppError> {
        sqlx::query!(
            r#"UPDATE integration_connections
               SET last_sync_at = $3, updated_at = now()
               WHERE id = $1 AND user_id = $2"#,
            id.into_uuid(),
            user_id.into_uuid(),
            at,
        )
        .execute(&self.pool)
        .await
        .map_err(map_err)?;
        Ok(())
    }

    async fn set_last_error(
        &self,
        id: IntegrationConnectionId,
        user_id: UserId,
        error: Option<String>,
    ) -> Result<(), AppError> {
        sqlx::query!(
            r#"UPDATE integration_connections
               SET last_error = $3, updated_at = now()
               WHERE id = $1 AND user_id = $2"#,
            id.into_uuid(),
            user_id.into_uuid(),
            error.as_deref(),
        )
        .execute(&self.pool)
        .await
        .map_err(map_err)?;
        Ok(())
    }

    async fn update_config(
        &self,
        id: IntegrationConnectionId,
        user_id: UserId,
        config: serde_json::Value,
    ) -> Result<(), AppError> {
        // Bump version even on the unchecked path so a subsequent
        // `update_config_with_version` can still detect that someone
        // (e.g. an OAuth re-auth) wrote in between.
        let result = sqlx::query!(
            r#"UPDATE integration_connections
               SET config = $3, updated_at = now(), version = version + 1
               WHERE id = $1 AND user_id = $2"#,
            id.into_uuid(),
            user_id.into_uuid(),
            config,
        )
        .execute(&self.pool)
        .await
        .map_err(map_err)?;

        if result.rows_affected() == 0 {
            return Err(AppError::Domain(DomainError::NotFound {
                entity: "integration_connection",
                id: id.to_string(),
            }));
        }
        Ok(())
    }

    async fn update_config_with_version(
        &self,
        id: IntegrationConnectionId,
        user_id: UserId,
        expected_version: i64,
        config: serde_json::Value,
    ) -> Result<i64, AppError> {
        // Atomic read-and-bump: the WHERE clause demands the version the
        // caller saw; a concurrent writer that bumped it between read and
        // write makes this match zero rows.
        let row = sqlx::query!(
            r#"UPDATE integration_connections
               SET config = $4, updated_at = now(), version = version + 1
               WHERE id = $1 AND user_id = $2 AND version = $3
               RETURNING version"#,
            id.into_uuid(),
            user_id.into_uuid(),
            expected_version,
            config,
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(map_err)?;

        match row {
            Some(record) => Ok(record.version),
            None => {
                // Disambiguate "row exists but version moved" (Conflict)
                // from "row truly missing" (NotFound) for clearer caller
                // semantics: the API layer maps Conflict to 409 and
                // NotFound to 404.
                let exists = sqlx::query_scalar!(
                    "SELECT 1 AS hit FROM integration_connections WHERE id = $1 AND user_id = $2",
                    id.into_uuid(),
                    user_id.into_uuid(),
                )
                .fetch_optional(&self.pool)
                .await
                .map_err(map_err)?
                .is_some();

                if exists {
                    Err(AppError::Domain(DomainError::Conflict {
                        entity: "integration_connection",
                        message: "settings were updated concurrently; reload and retry".to_string(),
                    }))
                } else {
                    Err(AppError::Domain(DomainError::NotFound {
                        entity: "integration_connection",
                        id: id.to_string(),
                    }))
                }
            }
        }
    }

    async fn delete(&self, id: IntegrationConnectionId, user_id: UserId) -> Result<(), AppError> {
        let result = sqlx::query!(
            r#"DELETE FROM integration_connections
               WHERE id = $1 AND user_id = $2"#,
            id.into_uuid(),
            user_id.into_uuid(),
        )
        .execute(&self.pool)
        .await
        .map_err(map_err)?;

        if result.rows_affected() == 0 {
            return Err(AppError::Domain(DomainError::NotFound {
                entity: "integration_connection",
                id: id.to_string(),
            }));
        }
        Ok(())
    }

    async fn count_pending_jobs_per_connection(
        &self,
        user_id: UserId,
    ) -> Result<HashMap<IntegrationConnectionId, u32>, AppError> {
        // job_outbox has no user/connection columns; both live in `payload`.
        // Filtering by serialized `user_id` (prefixed string form) keeps the
        // query scoped to the caller; we then parse the prefixed connection_id
        // back into a typed key.
        let user_key = user_id.to_string();
        let rows = sqlx::query!(
            r#"SELECT payload->>'connection_id' AS connection_id,
                      COUNT(*)::bigint AS "count!"
               FROM job_outbox
               WHERE dispatched_at IS NULL
                 AND job_type LIKE 'integration.%'
                 AND payload->>'user_id' = $1
               GROUP BY payload->>'connection_id'"#,
            user_key,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(map_err)?;

        let mut out = HashMap::with_capacity(rows.len());
        for row in rows {
            let Some(raw) = row.connection_id else {
                continue;
            };
            // Skip integration jobs whose payloads predate the typed
            // connection_id field rather than failing the whole listing.
            let Ok(id) = IntegrationConnectionId::from_str(&raw) else {
                continue;
            };
            let count = u32::try_from(row.count.max(0)).unwrap_or(u32::MAX);
            out.insert(id, count);
        }
        Ok(out)
    }
}
