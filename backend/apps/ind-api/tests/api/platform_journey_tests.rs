use ind_test_support::spawn_app;
use reqwest::StatusCode;
use serde_json::json;

use super::common::{assert_json_response, assert_status};

#[tokio::test]
async fn home_and_webhook_settings_persist_with_tenant_isolation() {
    let app = spawn_app().await;
    let owner = app.create_web_session().await;
    let stranger = app.create_web_session().await;
    let owner_client = app.authed_client(&owner);
    let stranger_client = app.authed_client(&stranger);

    let saved = assert_json_response(
        owner_client
            .post_json(
                "/api/v1/library",
                &json!({"url": "https://example.com/home-platform", "title": "Home Platform"}),
            )
            .await,
        StatusCode::OK,
    )
    .await;
    let dashboard = assert_json_response(
        owner_client
            .get("/api/v1/home?widgets=recently_added,reading_stats")
            .await,
        StatusCode::OK,
    )
    .await;
    assert_eq!(
        dashboard["recently_added"]["items"][0]["id"],
        saved["document_id"]
    );
    assert!(dashboard.get("continue_reading").is_none());
    assert!(dashboard["reading_stats"]["documents_read"].is_number());

    let settings = assert_json_response(
        owner_client
            .patch_json(
                "/api/v1/settings/home",
                &json!({
                    "widget_order": ["recently_added", "reading_stats"],
                    "hidden_widgets": ["feed_digest"]
                }),
            )
            .await,
        StatusCode::OK,
    )
    .await;
    let persisted = assert_json_response(
        owner_client.get("/api/v1/settings/home").await,
        StatusCode::OK,
    )
    .await;
    assert_eq!(persisted, settings);

    let webhook = assert_json_response(
        owner_client
            .post_json(
                "/api/v1/webhooks",
                &json!({
                    "name": "Library events",
                    "url": "https://127.0.0.1:1/indelible-webhook",
                    "events": ["library_entry.saved"],
                    "is_active": true
                }),
            )
            .await,
        StatusCode::CREATED,
    )
    .await;
    let webhook_id = webhook["id"].as_str().expect("webhook id");
    let original_secret = webhook["raw_secret"].as_str().expect("webhook secret");
    assert_status(
        stranger_client
            .patch_json(
                &format!("/api/v1/webhooks/{webhook_id}"),
                &json!({"is_active": false}),
            )
            .await,
        StatusCode::NOT_FOUND,
    )
    .await;
    let paused = assert_json_response(
        owner_client
            .patch_json(
                &format!("/api/v1/webhooks/{webhook_id}"),
                &json!({"name": "Paused events", "is_active": false}),
            )
            .await,
        StatusCode::OK,
    )
    .await;
    assert_eq!(paused["last_status"], "paused");
    assert_json_response(
        owner_client
            .patch_json(
                &format!("/api/v1/webhooks/{webhook_id}"),
                &json!({"is_active": true}),
            )
            .await,
        StatusCode::OK,
    )
    .await;
    let rotated = assert_json_response(
        owner_client
            .post_json(
                &format!("/api/v1/webhooks/{webhook_id}/rotate-secret"),
                &json!({}),
            )
            .await,
        StatusCode::OK,
    )
    .await;
    assert_ne!(rotated["raw_secret"], original_secret);
    let hooks =
        assert_json_response(owner_client.get("/api/v1/webhooks").await, StatusCode::OK).await;
    assert_eq!(hooks["data"][0]["events"][0], "library_entry.saved");
    let delivery = assert_json_response(
        owner_client
            .post_json(
                &format!("/api/v1/webhooks/{webhook_id}/test"),
                &json!({"event": "library_entry.saved"}),
            )
            .await,
        StatusCode::OK,
    )
    .await;
    assert!(delivery["status_code"].is_null());
    assert_eq!(delivery["attempt"], 1);
    let deliveries = assert_json_response(
        owner_client
            .get(&format!("/api/v1/webhooks/{webhook_id}/deliveries"))
            .await,
        StatusCode::OK,
    )
    .await;
    assert_eq!(deliveries["data"][0]["id"], delivery["id"]);
    assert_eq!(deliveries["data"][0]["target"], webhook["url"]);
    assert_status(
        owner_client
            .delete(&format!("/api/v1/webhooks/{webhook_id}"))
            .await,
        StatusCode::NO_CONTENT,
    )
    .await;
}
