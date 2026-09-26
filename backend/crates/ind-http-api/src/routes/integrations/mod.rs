pub mod dto;

mod auth;
mod connections;
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
    IntegrationConnectionDto, IntegrationListResponse, SyncIntegrationResponse,
};
pub use miniflux::{__path_connect_miniflux, ConnectMinifluxRequest, connect_miniflux};

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
            "/api/v1/integrations/miniflux/connect",
            post(connect_miniflux),
        )
}
