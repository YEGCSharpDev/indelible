use axum::{
    extract::{Path, State},
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::{
    error::ApiError,
    middleware::{Principal, RequireVerifiedUserAccessJwt},
    response::ApiResponse,
    state::AppState,
};

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct OnboardingResponse {
    pub current_step: i16,
    pub completed: bool,
    pub steps: Vec<OnboardingStepResponse>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct OnboardingStepResponse {
    pub step: i16,
    pub name: String,
    pub completed: bool,
}

impl From<ind_application::ports::OnboardingStepInfo> for OnboardingStepResponse {
    fn from(s: ind_application::ports::OnboardingStepInfo) -> Self {
        Self {
            step: s.step,
            name: s.name,
            completed: s.completed,
        }
    }
}

impl From<ind_application::ports::OnboardingStatus> for OnboardingResponse {
    fn from(status: ind_application::ports::OnboardingStatus) -> Self {
        Self {
            current_step: status.current_step,
            completed: status.completed,
            steps: status
                .steps
                .into_iter()
                .map(OnboardingStepResponse::from)
                .collect(),
        }
    }
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CompleteStepRequest {
    #[schema(value_type = StepData)]
    pub data: serde_json::Value,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct StepData {
    pub display_name: Option<String>,
    pub theme: Option<String>,
    pub source: Option<String>,
    pub feed_urls: Option<Vec<String>>,
    pub chat_provider: Option<String>,
    pub chat_api_key: Option<String>,
    pub chat_endpoint: Option<String>,
    pub chat_model: Option<String>,
    pub embedding_provider: Option<String>,
    pub embedding_api_key: Option<String>,
    pub embedding_endpoint: Option<String>,
    pub embedding_model: Option<String>,
    pub embedding_dim: Option<i32>,
}

#[utoipa::path(
    get,
    path = "/api/v1/onboarding",
    responses(
        (status = 200, description = "Get onboarding status", body = OnboardingResponse),
        (status = 401, description = "Authentication required"),
        (status = 403, description = "Verified user access JWT from a supported client and verified email required"),
    ),
    security(("bearer" = [])),
    tag = "Onboarding",
)]
pub async fn get_onboarding(
    RequireVerifiedUserAccessJwt(Principal { user_id, .. }): RequireVerifiedUserAccessJwt,
    State(state): State<AppState>,
) -> Result<ApiResponse<OnboardingResponse>, ApiError> {
    let status = state
        .onboarding_ops
        .get_onboarding(user_id)
        .await
        .map_err(ApiError::from)?;
    Ok(ApiResponse::new(OnboardingResponse::from(status)))
}

#[utoipa::path(
    post,
    path = "/api/v1/onboarding/steps/{step}/complete",
    params(
        ("step" = i16, Path, description = "Step number"),
    ),
    request_body = CompleteStepRequest,
    responses(
        (status = 200, description = "Complete onboarding step", body = OnboardingResponse),
        (status = 401, description = "Authentication required"),
        (status = 403, description = "Verified user access JWT from a supported client and verified email required"),
    ),
    security(("bearer" = [])),
    tag = "Onboarding",
)]
pub async fn complete_step(
    RequireVerifiedUserAccessJwt(Principal { user_id, .. }): RequireVerifiedUserAccessJwt,
    State(state): State<AppState>,
    Path(step): Path<i16>,
    Json(body): Json<CompleteStepRequest>,
) -> Result<ApiResponse<OnboardingResponse>, ApiError> {
    let status = state
        .onboarding_ops
        .complete_step(user_id, step, body.data)
        .await
        .map_err(ApiError::from)?;

    Ok(ApiResponse::new(OnboardingResponse::from(status)))
}

#[utoipa::path(
    post,
    path = "/api/v1/onboarding/skip",
    responses(
        (status = 200, description = "Skip onboarding", body = OnboardingResponse),
        (status = 401, description = "Authentication required"),
        (status = 403, description = "Verified user access JWT from a supported client and verified email required"),
    ),
    security(("bearer" = [])),
    tag = "Onboarding",
)]
pub async fn skip_onboarding(
    RequireVerifiedUserAccessJwt(Principal { user_id, .. }): RequireVerifiedUserAccessJwt,
    State(state): State<AppState>,
) -> Result<ApiResponse<OnboardingResponse>, ApiError> {
    let status = state
        .onboarding_ops
        .skip_onboarding(user_id)
        .await
        .map_err(ApiError::from)?;

    Ok(ApiResponse::new(OnboardingResponse::from(status)))
}

pub fn onboarding_routes() -> Router<AppState> {
    Router::new()
        .route("/api/v1/onboarding", get(get_onboarding))
        .route(
            "/api/v1/onboarding/steps/{step}/complete",
            post(complete_step),
        )
        .route("/api/v1/onboarding/skip", post(skip_onboarding))
}
