use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{HighlightId, ImportJobId, IntegrationConnectionId, LibraryEntryId, UserId};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IntegrationProvider {
    Logseq,
    BrowserExtension,
    EmailIngest,
    Miniflux,
    Custom,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IntegrationOAuthProvider {
    Custom,
}

impl IntegrationOAuthProvider {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Custom => "custom",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ImportMethod {
    Oauth,
    Csv,
    Zip,
}

impl ImportMethod {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Oauth => "oauth",
            Self::Csv => "csv",
            Self::Zip => "zip",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ImportJobStatus {
    AwaitingProvider,
    Pending,
    Running,
    Completed,
    Failed,
    Partial,
    RolledBack,
}

impl ImportJobStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::AwaitingProvider => "awaiting_provider",
            Self::Pending => "pending",
            Self::Running => "running",
            Self::Completed => "completed",
            Self::Failed => "failed",
            Self::Partial => "partial",
            Self::RolledBack => "rolled_back",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ImportItemOutcome {
    Imported,
    Updated,
    Duplicate,
    SkippedPrivate,
    Failed,
}

impl ImportItemOutcome {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Imported => "imported",
            Self::Updated => "updated",
            Self::Duplicate => "duplicate",
            Self::SkippedPrivate => "skipped_private",
            Self::Failed => "failed",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrationConnection {
    pub id: IntegrationConnectionId,
    pub user_id: UserId,
    pub provider: IntegrationProvider,
    pub config: serde_json::Value,
    pub status: String,
    pub last_sync_at: Option<DateTime<Utc>>,
    pub last_error: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    /// Optimistic-locking version. Incremented every time `config` is
    /// mutated through `update_config_with_version`. Settings PATCH
    /// handlers read this alongside the rest of the connection and pass
    /// it back so concurrent writes race-lose with `Conflict` instead of
    /// silently overwriting each other.
    #[serde(default)]
    pub version: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrationOAuthToken {
    pub id: uuid::Uuid,
    pub user_id: UserId,
    pub provider: IntegrationOAuthProvider,
    pub access_token_enc: Vec<u8>,
    pub refresh_token_enc: Option<Vec<u8>>,
    pub token_expires_at: Option<DateTime<Utc>>,
    pub extra: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportJob {
    pub id: ImportJobId,
    pub user_id: UserId,
    pub import_source: crate::ImportSource,
    pub import_method: ImportMethod,
    pub status: ImportJobStatus,
    pub imported_count: i32,
    pub updated_count: i32,
    pub duplicate_count: i32,
    pub skipped_private_count: i32,
    pub failed_count: i32,
    pub raw_artifact_key: Option<String>,
    pub provider_report: Option<serde_json::Value>,
    pub error: Option<String>,
    pub created_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub finished_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportJobItem {
    pub id: uuid::Uuid,
    pub import_job_id: ImportJobId,
    pub external_id: String,
    pub title: Option<String>,
    pub outcome: ImportItemOutcome,
    pub error: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct ImportJobCountsDelta {
    pub imported: i32,
    pub updated: i32,
    pub duplicate: i32,
    pub skipped_private: i32,
    pub failed: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportCursor {
    pub connection_id: IntegrationConnectionId,
    pub library_entry_id: LibraryEntryId,
    pub last_synced_at: Option<DateTime<Utc>>,
    pub last_attempted_at: Option<DateTime<Utc>>,
    pub cursor_version: i32,
    pub last_error: Option<String>,
    pub remote_page_id: Option<String>,
    pub last_exported_highlight_created_at: Option<DateTime<Utc>>,
    pub last_exported_highlight_id: Option<HighlightId>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
