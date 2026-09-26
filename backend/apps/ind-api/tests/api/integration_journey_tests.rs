use ind_application::repos::email_sender::EmailSenderRepository;
use ind_domain::CanonicalAddress;
use ind_persistence::repos::PgEmailSenderRepository;
use ind_test_support::spawn_app;
use reqwest::StatusCode;
use serde_json::json;

use super::common::{assert_json_response as response, assert_status};

#[tokio::test]
async fn email_sender_journey_is_user_scoped_and_persists_preferences() {
    let app = spawn_app().await;
    let owner = app.create_web_session().await;
    let other = app.create_web_session().await;
    let sender = PgEmailSenderRepository::new(app.pool().clone())
        .upsert_for_user(
            owner.user.id,
            &CanonicalAddress::new("newsletter@example.com"),
            Some("<weekly.example>"),
            Some("Weekly"),
        )
        .await
        .unwrap();
    let client = app.authed_client(&owner);
    let list = response(client.get("/api/v1/email-senders").await, StatusCode::OK).await;
    assert_eq!(list["data"][0]["id"], sender.id.to_string());
    let path = format!("/api/v1/email-senders/{}", sender.id);
    let changed = response(
        client
            .patch_json(
                &path,
                &json!({"blocked": true, "render_default": "original"}),
            )
            .await,
        StatusCode::OK,
    )
    .await;
    assert_eq!(changed["blocked"], true);
    assert_eq!(changed["render_default"], "original");
    assert_status(
        app.authed_client(&other)
            .patch_json(&path, &json!({"blocked": false}))
            .await,
        StatusCode::NOT_FOUND,
    )
    .await;
}
