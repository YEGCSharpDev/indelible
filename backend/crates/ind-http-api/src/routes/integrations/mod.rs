pub mod dto;

mod auth;
mod connections;
mod obsidian;
mod miniflux;

use axum::Router;
use axum::routing::{delete, get, post};

use crate::middleware::rate_limit::RateLimiters;
use crate::state::AppState;

pub use auth::{
    __path_authorize_integration, __path_integration_callback, authorize_integration,
    integration_callback,
};
pub use connections::{
    __path_delete_integration, __path_list_integrations, __path_sync_integration,
    delete_integration, list_integrations, sync_integration,
};
pub use dto::{
    AuthorizeIntegrationRequest, AuthorizeIntegrationResponse, CallbackQuery,
    IntegrationConnectionDto, IntegrationListResponse, ObsidianPreviewRequest,
    ObsidianPreviewResponse, ObsidianSettingsDto, SyncIntegrationResponse,
    UpdateObsidianSettingsRequest,
};
pub use miniflux::{
    __path_connect_miniflux, connect_miniflux, ConnectMinifluxRequest,
};
pub use obsidian::{
    __path_get_obsidian_settings, __path_preview_obsidian_export, __path_setup_obsidian_connection,
    __path_update_obsidian_settings, get_obsidian_settings, preview_obsidian_export,
    setup_obsidian_connection, update_obsidian_settings,
};

pub fn integration_routes(_rate_limiters: RateLimiters) -> Router<AppState> {
    Router::new()
        .route("/api/v1/integrations", get(list_integrations))
        .route(
            "/api/v1/integrations/{provider}/authorize",
            post(authorize_integration),
        )
        .route(
            "/api/v1/integrations/{provider}/callback",
            get(integration_callback),
        )
        .route("/api/v1/integrations/{id}", delete(delete_integration))
        .route("/api/v1/integrations/{id}/sync", post(sync_integration))
        .route(
            "/api/v1/integrations/{id}/obsidian/settings",
            get(get_obsidian_settings).patch(update_obsidian_settings),
        )
        .route(
            "/api/v1/integrations/{id}/obsidian/preview",
            post(preview_obsidian_export),
        )
        .route(
            "/api/v1/integrations/obsidian/setup",
            post(setup_obsidian_connection),
        )
        .route(
            "/api/v1/integrations/miniflux/connect",
            post(connect_miniflux),
        )
}
