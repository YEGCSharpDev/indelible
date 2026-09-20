use axum::extract::State;
use serde::Deserialize;

use crate::error::ApiError;
use crate::middleware::RequireIntegrationsWrite;
use crate::response::ApiResponse;
use crate::state::AppState;
use super::dto::IntegrationConnectionDto;

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct ConnectMinifluxRequest {
    pub url: String,
    pub api_key: String,
}

#[utoipa::path(
    post,
    path = "/api/v1/integrations/miniflux/connect",
    request_body = ConnectMinifluxRequest,
    responses(
        (status = 200, description = "Miniflux connected", body = IntegrationConnectionDto),
        (status = 401, description = "Authentication required"),
        (status = 400, description = "Invalid Miniflux credentials"),
    ),
    security(("bearer" = []), ("api_token" = [])),
    extensions(("x-indelible-permissions" = json!(["integrations:write"]))),
    tag = "Integrations",
)]
pub async fn connect_miniflux(
    RequireIntegrationsWrite {
        principal: auth_user,
        ..
    }: RequireIntegrationsWrite,
    State(state): State<AppState>,
    axum::extract::Json(payload): axum::extract::Json<ConnectMinifluxRequest>,
) -> Result<ApiResponse<IntegrationConnectionDto>, ApiError> {
    let ops = state.integration_ops.as_ref().ok_or(ApiError::NotFound {
        entity: "integrations",
        id: String::new(),
    })?;

    let connection = ops
        .setup_miniflux_connection(auth_user.user_id, payload.url, payload.api_key)
        .await
        .map_err(ApiError::from)?;

    // We start with 0 pending jobs for a new connection
    Ok(ApiResponse::new(IntegrationConnectionDto::from_with_pending(connection, 0)))
}
