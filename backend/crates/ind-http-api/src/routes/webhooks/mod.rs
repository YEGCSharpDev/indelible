use axum::Router;
use axum::extract::{Path, State};
use axum::routing::{get, patch, post};
use chrono::{DateTime, Utc};
use ind_application::ports::WebhookOperations;
use ind_application::webhooks::is_known_webhook_event;
use ind_domain::{WebhookDelivery, WebhookEndpoint, WebhookEndpointId};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::error::{ApiError, FieldError};
use crate::extract::Json;
use crate::middleware::{RequireWebhooksRead, RequireWebhooksWrite};
use crate::response::{ApiResponse, EmptyResponse};
use crate::state::AppState;

mod dto;
pub(crate) mod handlers;
mod helpers;

pub use dto::*;
use handlers::*;

pub fn webhook_routes() -> Router<AppState> {
    Router::new()
        .route(
            "/api/v1/webhooks",
            get(list_webhook_endpoints).post(create_webhook_endpoint),
        )
        .route(
            "/api/v1/webhooks/{webhook_id}",
            patch(update_webhook_endpoint).delete(delete_webhook_endpoint),
        )
        .route(
            "/api/v1/webhooks/{webhook_id}/rotate-secret",
            post(rotate_webhook_secret),
        )
        .route(
            "/api/v1/webhooks/{webhook_id}/test",
            post(test_webhook_endpoint),
        )
        .route(
            "/api/v1/webhooks/{webhook_id}/deliveries",
            get(list_webhook_deliveries),
        )
}
