use axum::{
    extract::{Path, State},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};

use crate::{error::ApiError, 
    Principal,
    state::AppState,
    RequireVerifiedUserAccessJwt,
};

#[derive(Serialize, Deserialize, utoipa::ToSchema)]
pub struct OnboardingResponse {
    pub has_completed_onboarding: bool,
    pub onboarding_step: i16,
}

#[derive(Serialize, Deserialize, utoipa::ToSchema)]
pub struct OnboardingStepResponse {
    pub step: i16,
    pub metadata: Option<serde_json::Value>,
}

pub struct ApiResponse<T> {
    pub data: T,
}

impl<T> ApiResponse<T> {
    pub fn new(data: T) -> Self {
        Self { data }
    }
}

impl<T: Serialize> IntoResponse for ApiResponse<T> {
    fn into_response(self) -> axum::response::Response {
        Json(self.data).into_response()
    }
}

#[derive(Deserialize, utoipa::ToSchema)]
pub struct CompleteStepRequest {
    pub data: serde_json::Value,
}

#[derive(Deserialize, utoipa::ToSchema)]
pub struct StepData {
    #[serde(default)]
    pub feed_urls: Option<Vec<String>>,
}

pub async fn get_onboarding(
    RequireVerifiedUserAccessJwt(Principal { user_id, .. }): RequireVerifiedUserAccessJwt,
    State(state): State<AppState>,
) -> Result<ApiResponse<OnboardingResponse>, ApiError> {
    let status = state
        .onboarding_ops
        .get_onboarding(user_id)
        .await
        .map_err(ApiError::from)?;
    Ok(ApiResponse::new(OnboardingResponse {
        has_completed_onboarding: status.completed,
        onboarding_step: status.current_step,
    }))
}

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

    Ok(ApiResponse::new(OnboardingResponse {
        has_completed_onboarding: status.completed,
        onboarding_step: status.current_step,
    }))
}

pub async fn skip_onboarding(
    RequireVerifiedUserAccessJwt(Principal { user_id, .. }): RequireVerifiedUserAccessJwt,
    State(state): State<AppState>,
) -> Result<ApiResponse<OnboardingResponse>, ApiError> {
    let status = state
        .onboarding_ops
        .skip_onboarding(user_id)
        .await
        .map_err(ApiError::from)?;

    Ok(ApiResponse::new(OnboardingResponse {
        has_completed_onboarding: status.completed,
        onboarding_step: status.current_step,
    }))
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
