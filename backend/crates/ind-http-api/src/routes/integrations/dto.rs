use chrono::{DateTime, Utc};
use ind_domain::{IntegrationConnection, IntegrationProvider};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Serialize, ToSchema)]
pub struct IntegrationListResponse {
    pub connections: Vec<IntegrationConnectionDto>,
    /// Lowercase ids of OAuth providers this instance holds credentials for.
    /// A provider absent here cannot be connected until an administrator
    /// configures it.
    #[schema(example = json!([]))]
    pub available_oauth_providers: Vec<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct IntegrationConnectionDto {
    pub id: String,
    #[schema(value_type = String, example = "miniflux")]
    pub provider: IntegrationProvider,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(value_type = Option<String>, format = DateTime)]
    pub last_sync_at: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_error: Option<String>,
    pub config: IntegrationConnectionConfigDto,
    /// Count of queued integration jobs for this connection (sync + export).
    /// Frontend uses this to render a "pending" pill alongside connection status.
    pub pending_jobs: u32,
    #[schema(value_type = String, format = DateTime)]
    pub created_at: DateTime<Utc>,
}

impl IntegrationConnectionDto {
    pub fn from_with_pending(c: IntegrationConnection, pending_jobs: u32) -> Self {
        let config = IntegrationConnectionConfigDto::from_domain(&c.provider, &c.config);
        Self {
            id: c.id.to_string(),
            provider: c.provider,
            status: c.status,
            last_sync_at: c.last_sync_at,
            last_error: c.last_error,
            config,
            pending_jobs,
            created_at: c.created_at,
        }
    }
}

impl From<IntegrationConnection> for IntegrationConnectionDto {
    fn from(c: IntegrationConnection) -> Self {
        Self::from_with_pending(c, 0)
    }
}

/// Provider-shaped configuration payload. Uses serde's tagged enum so the
/// generated OpenAPI schema produces a concrete discriminated union instead
/// of a bare object — mobile/web codegen can model it as a sealed class/union.
#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(tag = "provider", rename_all = "snake_case")]
pub enum IntegrationConnectionConfigDto {
    EmailIngest {
        address: String,
    },
    /// Catch-all for providers that don't have first-class structured config
    /// on this surface yet (currently Logseq, BrowserExtension). Kept as one
    /// variant so generated mobile/web codegen doesn't ship empty
    /// `LogseqConfig` / `BrowserExtensionConfig` types that can never be
    /// instantiated. When a provider gains structured config, add a dedicated
    /// variant and route to it from `from_domain`.
    Other {
        provider_name: String,
    },
}

impl IntegrationConnectionConfigDto {
    pub fn from_domain(
        provider: &IntegrationProvider,
        raw: &serde_json::Value,
    ) -> IntegrationConnectionConfigDto {
        match provider {
            IntegrationProvider::EmailIngest => IntegrationConnectionConfigDto::EmailIngest {
                address: string_field(raw, "address").unwrap_or_default(),
            },
            IntegrationProvider::Custom => IntegrationConnectionConfigDto::Other {
                provider_name: "custom".to_string(),
            },
            IntegrationProvider::Logseq => IntegrationConnectionConfigDto::Other {
                provider_name: "logseq".to_string(),
            },
            IntegrationProvider::BrowserExtension => IntegrationConnectionConfigDto::Other {
                provider_name: "browser_extension".to_string(),
            },
            IntegrationProvider::Miniflux => IntegrationConnectionConfigDto::Other {
                provider_name: "miniflux".to_string(),
            },
        }
    }
}

fn string_field(raw: &serde_json::Value, key: &str) -> Option<String> {
    raw.get(key).and_then(|v| v.as_str()).map(|s| s.to_string())
}

#[derive(Debug, Deserialize, ToSchema, validator::Validate)]
pub struct AuthorizeIntegrationRequest {
    #[serde(default)]
    pub redirect_after: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct AuthorizeIntegrationResponse {
    pub authorize_url: String,
}

/// OAuth callback query string. Code/state are absent when the provider
/// reports an error or when the user navigates to the URL manually; in those
/// cases the handler emits a redirect to the hub with `integration_error=…`.
#[derive(Debug, Deserialize)]
pub struct CallbackQuery {
    #[serde(default)]
    pub code: Option<String>,
    #[serde(default)]
    pub state: Option<String>,
    #[serde(default)]
    pub error: Option<String>,
    #[serde(default)]
    pub error_description: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct SyncIntegrationResponse {
    pub job_id: String,
}
