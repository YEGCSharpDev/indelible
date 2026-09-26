#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use ind_test_support::spawn_app;
use serde_json::json;

/// A self-hosted instance without OAuth credentials must tell clients so
/// through the catalog, and an authorize attempt must fail with actionable
/// guidance rather than a leaked persistence identifier.
#[tokio::test]
async fn unconfigured_instance_reports_no_oauth_providers_and_rejects_authorize() {
    let app = spawn_app().await;
    let session = app.create_web_session().await;
    let client = app.authed_client(&session);

    let response = client.get("/api/v1/integrations").await;
    assert_eq!(response.status(), 200);
    let body: serde_json::Value = response.json().await.unwrap();
    assert_eq!(
        body["available_oauth_providers"],
        json!([]),
        "unconfigured instance must report an empty provider list, got {body}"
    );

    let response = client
        .post_json("/api/v1/integrations/custom/authorize", &json!({}))
        .await;
    assert_eq!(
        response.status(),
        503,
        "authorize against an unconfigured provider must be a service-unavailable error"
    );
    let error_body = response.text().await.unwrap();
    assert!(
        !error_body.contains("integration_provider not found"),
        "the internal entity identifier must not leak to clients, got {error_body}"
    );
}

#[tokio::test]
async fn unknown_provider_returns_not_found() {
    let app = spawn_app().await;
    let session = app.create_web_session().await;
    let client = app.authed_client(&session);

    let response = client
        .post_json("/api/v1/integrations/unknown/authorize", &json!({}))
        .await;
    assert_eq!(response.status(), 404);
}
