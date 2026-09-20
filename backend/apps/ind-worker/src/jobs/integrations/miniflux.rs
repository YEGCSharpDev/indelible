use std::sync::Arc;
use serde::Deserialize;

use ind_application::AppError;
use ind_application::repos::integration_connection::IntegrationConnectionRepository;
use ind_application::repos::document_lifecycle::{DocumentLifecycle, MaterializeIdentity, SaveToLibraryRequest, MaterializeOrigin};
use ind_domain::{MinifluxSyncConnectionJob, IntegrationProvider, ContentSource, NewUrlDocument, DocumentType, DocumentOriginType, deterministic_origin_id};
use ind_persistence::repos::PgIntegrationConnectionRepository;
use sqlx::PgPool;

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct MinifluxResponse {
    total: i32,
    entries: Vec<MinifluxEntry>,
}

#[derive(Debug, Deserialize)]
struct MinifluxEntry {
    id: i32,
    title: String,
    url: String,
}

pub struct MinifluxSyncWorker {
    pool: PgPool,
    connection_repo: Arc<dyn IntegrationConnectionRepository>,
    lifecycle: Arc<dyn DocumentLifecycle>,
}

impl MinifluxSyncWorker {
    pub fn new(pool: PgPool, lifecycle: Arc<dyn DocumentLifecycle>) -> Self {
        Self { 
            pool: pool.clone(),
            connection_repo: Arc::new(PgIntegrationConnectionRepository::new(pool)),
            lifecycle,
        }
    }

    pub async fn run(&self, job: MinifluxSyncConnectionJob) -> Result<(), AppError> {
        let connection = self
            .connection_repo
            .find_by_id(job.user_id, job.connection_id)
            .await?
            .ok_or_else(|| AppError::Domain(ind_domain::DomainError::NotFound {
                entity: "IntegrationConnection",
                id: job.connection_id.to_string(),
            }))?;

        if connection.provider != IntegrationProvider::Miniflux {
            return Err(AppError::Domain(ind_domain::DomainError::Validation {
                field: "provider".to_string(),
                message: "Expected Miniflux provider".to_string(),
            }));
        }

        tracing::info!(
            "Syncing Miniflux connection {} for user {}",
            connection.id, job.user_id
        );

        let config = connection.config;
        let url = config.get("url").and_then(|v| v.as_str()).unwrap_or_default();
        let api_key = config.get("api_key").and_then(|v| v.as_str()).unwrap_or_default();

        if url.is_empty() || api_key.is_empty() {
            return Err(AppError::Domain(ind_domain::DomainError::Validation {
                field: "config".to_string(),
                message: "Missing Miniflux URL or API key".to_string(),
            }));
        }

        let client = reqwest::Client::new();
        let res = client
            .get(format!("{}/v1/entries?status=unread", url.trim_end_matches('/')))
            .header("X-Auth-Token", api_key)
            .send()
            .await
            .map_err(|e| {
                tracing::error!("reqwest send error for Miniflux: {:?}", e);
                AppError::ExternalService {
                    service: "miniflux".into(),
                    message: format!("Failed to fetch Miniflux entries: {}", e),
                }
            })?;

        if !res.status().is_success() {
            let status = res.status();
            let body = res.text().await.unwrap_or_default();
            tracing::error!("Miniflux API error status: {} - {}", status, body);
            return Err(AppError::ExternalService {
                service: "miniflux".into(),
                message: format!("Miniflux API returned error status: {}", status),
            });
        }

        let data: MinifluxResponse = res.json().await.map_err(|e| AppError::ExternalService {
            service: "miniflux".into(),
            message: format!("Failed to parse Miniflux response: {}", e),
        })?;

        tracing::info!("Found {} unread Miniflux entries", data.entries.len());

        for entry in data.entries {
            // For testing, we just log and ingest as library entries.
            // We use origin id to prevent duplicates.
            let origin_id = deterministic_origin_id(
                DocumentOriginType::MinifluxItem,
                job.user_id,
                &format!("miniflux:{}", entry.id),
            );
            
            let identity = MaterializeIdentity::Url {
                document: NewUrlDocument {
                    id: ind_domain::DocumentId::new(),
                    user_id: job.user_id,
                    canonical_url: entry.url.clone(),
                    original_url: Some(entry.url.clone()),
                    title: entry.title.clone(),
                    document_type: DocumentType::Article,
                    content_hash: None,
                    author: None,
                    excerpt: None,
                    published_at: None,
                    thumbnail_url: None,
                    language: None,
                    domain: None,
                    lead_image_url: None,
                },
                origin: Some(MaterializeOrigin {
                    origin_type: DocumentOriginType::MinifluxItem,
                    origin_id,
                }),
            };

            let req = SaveToLibraryRequest {
                identity,
                source: ContentSource::Import,
                source_delivery_id: None,
                hide_deliveries: false,
                enqueue_engaged_ai: true,
                restore_policy: Default::default(),
                side_effects: None,
            };

            match self.lifecycle.save_to_library(req).await {
                Ok(outcome) => {
                    tracing::info!("Saved Miniflux entry {} to library (document {})", entry.id, outcome.document.id);
                    
                    // Store the mapping so we can push read states back later
                    let _ = sqlx::query(
                        "INSERT INTO miniflux_sync_map (user_id, document_id, miniflux_id) VALUES ($1, $2, $3) ON CONFLICT DO NOTHING"
                    )
                    .bind(job.user_id.into_uuid())
                    .bind(outcome.document.id.into_uuid())
                    .bind(entry.id)
                    .execute(&self.pool)
                    .await;
                }
                Err(e) => {
                    tracing::error!("Failed to save Miniflux entry {}: {}", entry.id, e);
                }
            }
        }

        self.connection_repo
            .set_last_sync_at(job.connection_id, job.user_id, chrono::Utc::now())
            .await?;

        Ok(())
    }

    pub async fn push_read_state(&self, job: ind_domain::MinifluxPushReadStateJob) -> Result<(), AppError> {
        tracing::info!("Pushing read state for Miniflux document {}", job.document_id);

        let row: Option<sqlx::postgres::PgRow> = sqlx::query(
            "SELECT miniflux_id FROM miniflux_sync_map WHERE user_id = $1 AND document_id = $2"
        )
        .bind(job.user_id.into_uuid())
        .bind(job.document_id.into_uuid())
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::Repository(Box::new(e)))?;

        let miniflux_id: i32 = match row {
            Some(r) => sqlx::Row::get(&r, "miniflux_id"),
            None => {
                tracing::info!("Document {} is not a Miniflux document (or missing map), skipping push", job.document_id);
                return Ok(());
            }
        };

        // Find the user's miniflux connection
        let connections = self.connection_repo.list_by_user(job.user_id).await?;
        let connection = match connections.into_iter().find(|c| c.provider == IntegrationProvider::Miniflux) {
            Some(c) => c,
            None => {
                tracing::info!("No Miniflux connection found for user {}, skipping push", job.user_id);
                return Ok(());
            }
        };

        let config = connection.config;
        let url = config.get("url").and_then(|v| v.as_str()).unwrap_or_default();
        let api_key = config.get("api_key").and_then(|v| v.as_str()).unwrap_or_default();

        if url.is_empty() || api_key.is_empty() {
            return Ok(()); // Connection broken, skip
        }

        let client = reqwest::Client::new();
        let res = client
            .put(format!("{}/v1/entries", url.trim_end_matches('/')))
            .header("X-Auth-Token", api_key)
            .json(&serde_json::json!({
                "entry_ids": [miniflux_id],
                "status": "read"
            }))
            .send()
            .await
            .map_err(|e| AppError::ExternalService {
                service: "miniflux".into(),
                message: format!("Failed to update Miniflux read state: {}", e),
            })?;

        if !res.status().is_success() {
            tracing::error!("Miniflux API error on push read state: {}", res.status());
            return Err(AppError::ExternalService {
                service: "miniflux".into(),
                message: format!("Miniflux API returned error status: {}", res.status()),
            });
        }

        tracing::info!("Successfully pushed read state to Miniflux for entry {}", miniflux_id);
        Ok(())
    }
}
