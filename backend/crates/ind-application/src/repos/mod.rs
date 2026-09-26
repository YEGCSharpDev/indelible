use serde::{Deserialize, Serialize};

pub mod account_purge;
pub mod apalis_job;
pub mod api_token;
pub mod authorization_code;
pub mod background_job_recovery;
pub mod billing;
pub mod billing_usage_event;
pub mod collection;
pub mod dead_letter;
pub mod document;
pub mod document_asset;
pub mod document_lifecycle;
pub mod document_note;
pub mod document_reprocess;
pub mod document_upload;
pub mod email_alias;
pub mod email_ingest;
pub mod email_sender;
pub mod email_unsubscribe_commit;
pub mod email_unsubscribe_target;
pub mod email_verification;
pub mod entity;
pub mod event;
pub mod export_cursor;
pub mod export_subject;
pub mod feed;
pub mod feed_delivery;
pub mod highlight;
pub mod home;
pub mod import_job;
pub mod integration_connection;
pub mod integration_oauth_token;
pub mod integrity;
pub mod library;
pub mod lifecycle_outbox;
pub mod maintenance;
pub mod notification_preferences;
pub mod oauth_flow;
pub mod oauth_identity;
pub mod outbox;
pub mod password_reset;
pub mod playback_state;
pub mod prepared_content;
pub mod refresh_token;
pub mod retention_cleanup;
pub mod search;
pub mod search_reindex;
pub mod smart_list;
pub mod tag;
pub mod usage_counter;
pub mod user;
pub mod user_document_state;
pub mod user_preferences;
pub mod webhook;

pub use retention_cleanup::{
    FeedDeliveryPruneCounts, FeedDeliveryRetentionWindows, RetentionCleanupRepository,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cursor(pub String);

#[derive(Debug)]
pub struct Page<T> {
    pub items: Vec<T>,
    pub next_cursor: Option<Cursor>,
}
