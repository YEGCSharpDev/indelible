use reqwest::StatusCode;
use serde_json::json;

use ind_test_support::spawn_app;

#[tokio::test]
async fn connect_miniflux_succeeds_with_valid_payload() {
    let app = spawn_app().await;
    let session = app.create_web_session().await;
    let client = app.authed_client(&session);

    let response = client
        .post_json(
            "/api/v1/integrations/miniflux/connect",
            &json!({
                "url": "https://miniflux.example.com",
                "api_key": "secret-api-key-123",
            }),
        )
        .await;

    assert_eq!(response.status(), StatusCode::OK);
    let body: serde_json::Value = response.json().await.unwrap();
    assert_eq!(body["provider"], "miniflux");
    assert_eq!(body["status"], "connected");
    assert_eq!(body["pending_jobs"], 0);
    assert!(!body["id"].as_str().unwrap().is_empty());
}

#[tokio::test]
async fn connect_miniflux_rejects_invalid_url() {
    let app = spawn_app().await;
    let session = app.create_web_session().await;
    let client = app.authed_client(&session);

    let response = client
        .post_json(
            "/api/v1/integrations/miniflux/connect",
            &json!({
                "url": "not-a-valid-url",
                "api_key": "secret-api-key-123",
            }),
        )
        .await;

    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn connect_miniflux_rejects_empty_api_key() {
    let app = spawn_app().await;
    let session = app.create_web_session().await;
    let client = app.authed_client(&session);

    let response = client
        .post_json(
            "/api/v1/integrations/miniflux/connect",
            &json!({
                "url": "https://miniflux.example.com",
                "api_key": "",
            }),
        )
        .await;

    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
}
