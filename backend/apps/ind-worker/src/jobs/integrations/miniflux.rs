use std::sync::Arc;
use serde::Deserialize;

use ind_application::AppError;
use ind_application::repos::integration_connection::IntegrationConnectionRepository;
use ind_application::repos::document_lifecycle::{DocumentLifecycle, MaterializeIdentity, SaveToLibraryRequest, MaterializeOrigin};
use ind_application::repos::tag::TagRepository;
use ind_application::repos::event::MutationSideEffects;
use ind_application::repos::library::LibraryRepository;
use ind_domain::{MinifluxSyncConnectionJob, IntegrationProvider, ContentSource, NewUrlDocument, DocumentType, DocumentOriginType, deterministic_origin_id, TagSource, TriageState, LibraryEntryId};
use ind_persistence::repos::{PgIntegrationConnectionRepository, PgTagRepository, PgLibraryRepository};
use sqlx::PgPool;

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct MinifluxResponse {
    total: i32,
    entries: Vec<MinifluxEntry>,
}

#[derive(Debug, Deserialize)]
struct MinifluxFeedCategory {
    title: String,
}

#[derive(Debug, Deserialize)]
struct MinifluxFeed {
    category: Option<MinifluxFeedCategory>,
}

#[derive(Debug, Deserialize)]
struct MinifluxEntry {
    id: i32,
    title: String,
    url: String,
    tags: Option<Vec<String>>,
    feed: Option<MinifluxFeed>,
}

pub struct MinifluxSyncWorker {
    pool: PgPool,
    connection_repo: Arc<dyn IntegrationConnectionRepository>,
    tag_repo: Arc<dyn TagRepository>,
    library: Arc<dyn LibraryRepository>,
    lifecycle: Arc<dyn DocumentLifecycle>,
}

impl MinifluxSyncWorker {
    pub fn new(pool: PgPool, lifecycle: Arc<dyn DocumentLifecycle>) -> Self {
        Self { 
            pool: pool.clone(),
            connection_repo: Arc::new(PgIntegrationConnectionRepository::new(pool.clone())),
            tag_repo: Arc::new(PgTagRepository::new(pool.clone())),
            library: Arc::new(PgLibraryRepository::new(pool)),
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

        if !url.starts_with("http://") && !url.starts_with("https://") {
            return Err(AppError::Domain(ind_domain::DomainError::Validation {
                field: "url".to_string(),
                message: "Miniflux URL must start with http:// or https://".to_string(),
            }));
        }

        let client = reqwest::Client::builder()
            .connect_timeout(std::time::Duration::from_secs(10))
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| AppError::ExternalService {
                service: "miniflux".into(),
                message: format!("Failed to build HTTP client: {e}"),
            })?;
        let res = client
            .get(format!("{}/v1/entries?status=unread&limit=10000", url.trim_end_matches('/')))
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

        for entry in &data.entries {
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
                    
                    // Extract tags and categories
                    let mut names_to_import = Vec::new();
                    if let Some(feed) = &entry.feed {
                        if let Some(category) = &feed.category {
                            if !category.title.trim().is_empty() {
                                names_to_import.push(category.title.trim().to_string());
                            }
                        }
                    }
                    if let Some(tags) = &entry.tags {
                        for tag in tags {
                            if !tag.trim().is_empty() {
                                names_to_import.push(tag.trim().to_string());
                            }
                        }
                    }

                    // Deduplicate tag names
                    names_to_import.sort();
                    names_to_import.dedup();

                    let mut tag_ids = Vec::new();
                    for name in names_to_import {
                        match self.tag_repo.find_or_create_by_name(job.user_id, &name).await {
                            Ok(tag) => tag_ids.push(tag.id),
                            Err(e) => tracing::error!("Failed to find or create tag '{}': {}", name, e),
                        }
                    }

                    if !tag_ids.is_empty() {
                        if let Err(e) = self.tag_repo.replace_for_library_entry_with_source(
                            job.user_id,
                            outcome.entry.id,
                            &tag_ids,
                            TagSource::Import,
                            MutationSideEffects::none(),
                        ).await {
                            tracing::error!("Failed to attach tags to library entry: {}", e);
                        }
                    }

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

        let unread_ids: std::collections::HashSet<i32> = data.entries.iter().map(|e| e.id).collect();

        let rows = sqlx::query(
            r#"
            SELECT l.id as library_entry_id, m.miniflux_id 
            FROM library_entries l
            JOIN miniflux_sync_map m ON l.document_id = m.document_id AND l.user_id = m.user_id
            WHERE l.user_id = $1 AND l.triage_state IN ('Inbox', 'Later')
            "#
        )
        .bind(job.user_id.into_uuid())
        .fetch_all(&self.pool)
        .await
        .unwrap_or_default();

        for row in rows {
            let miniflux_id: i32 = sqlx::Row::get(&row, "miniflux_id");
            let library_entry_id: uuid::Uuid = sqlx::Row::get(&row, "library_entry_id");

            if !unread_ids.contains(&miniflux_id) {
                tracing::info!("Miniflux document {} is no longer unread, archiving library entry {}", miniflux_id, library_entry_id);
                let _ = self.library.set_triage_state(
                    LibraryEntryId::from_uuid(library_entry_id),
                    job.user_id,
                    TriageState::Archive,
                    MutationSideEffects::none()
                ).await;
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

        if !url.starts_with("http://") && !url.starts_with("https://") {
            return Err(AppError::Domain(ind_domain::DomainError::Validation {
                field: "url".to_string(),
                message: "Miniflux URL must start with http:// or https://".to_string(),
            }));
        }

        let client = reqwest::Client::builder()
            .connect_timeout(std::time::Duration::from_secs(10))
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| AppError::ExternalService {
                service: "miniflux".into(),
                message: format!("Failed to build HTTP client: {e}"),
            })?;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_miniflux_response_full() {
        let json = r#"{
            "total": 2,
            "entries": [
                {
                    "id": 42,
                    "title": "Article Title",
                    "url": "https://example.com/article",
                    "tags": ["rust", "tech"],
                    "feed": {
                        "category": {
                            "title": "Technology"
                        }
                    }
                },
                {
                    "id": 43,
                    "title": "Another Article",
                    "url": "https://example.com/article2",
                    "tags": [],
                    "feed": {
                        "category": null
                    }
                }
            ]
        }"#;

        let res: MinifluxResponse = serde_json::from_str(json).expect("deserialize miniflux response");
        assert_eq!(res.total, 2);
        assert_eq!(res.entries.len(), 2);

        let entry = &res.entries[0];
        assert_eq!(entry.id, 42);
        assert_eq!(entry.title, "Article Title");
        assert_eq!(entry.url, "https://example.com/article");
        assert_eq!(entry.tags.as_deref(), Some(&["rust".to_string(), "tech".to_string()][..]));
        assert_eq!(
            entry.feed.as_ref().and_then(|f| f.category.as_ref()).map(|c| c.title.as_str()),
            Some("Technology")
        );

        let entry2 = &res.entries[1];
        assert_eq!(entry2.id, 43);
        assert_eq!(entry2.tags.as_deref(), Some(&[][..]));
        assert!(entry2.feed.as_ref().and_then(|f| f.category.as_ref()).is_none());
    }

    #[test]
    fn test_deserialize_miniflux_response_optional_empty_fields() {
        let json = r#"{
            "total": 0,
            "entries": [
                {
                    "id": 100,
                    "title": "",
                    "url": "https://example.com/empty",
                    "tags": null,
                    "feed": null
                }
            ]
        }"#;

        let res: MinifluxResponse = serde_json::from_str(json).expect("deserialize empty fields");
        assert_eq!(res.total, 0);
        assert_eq!(res.entries.len(), 1);
        assert_eq!(res.entries[0].id, 100);
        assert_eq!(res.entries[0].title, "");
        assert!(res.entries[0].tags.is_none());
        assert!(res.entries[0].feed.is_none());
    }

    #[test]
    fn test_deserialize_miniflux_response_missing_optional_fields() {
        let json = r#"{
            "total": 0,
            "entries": [
                {
                    "id": 101,
                    "title": "Minimal",
                    "url": "https://example.com/minimal"
                }
            ]
        }"#;

        let res: MinifluxResponse = serde_json::from_str(json).expect("deserialize missing optional fields");
        assert_eq!(res.entries.len(), 1);
        assert_eq!(res.entries[0].id, 101);
        assert!(res.entries[0].tags.is_none());
        assert!(res.entries[0].feed.is_none());
    }
}
