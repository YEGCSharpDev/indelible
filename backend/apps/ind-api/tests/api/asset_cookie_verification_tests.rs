use ind_auth::{decode_asset_cookie_secret, sign_asset_cookie};
use ind_test_support::test_app::TEST_ASSET_COOKIE_SECRET_HEX;
use reqwest::{StatusCode, header};

use super::common::{SaveScenario, document_id_from_response};

#[tokio::test]
async fn suspended_users_asset_cookie_cannot_reach_asset_routes() {
    let scenario = SaveScenario::new().await;
    let saved = scenario
        .extension_reader_save("https://example.com/unverified-asset-cookie")
        .await;
    let document_id = document_id_from_response(&saved);
    let user_id = scenario.web.user.id;

    sqlx::query("UPDATE users SET status = 'suspended' WHERE id = $1")
        .bind(user_id.as_uuid())
        .execute(scenario.app.pool())
        .await
        .expect("mark asset-cookie user suspended");

    let cookie = sign_asset_cookie(
        &user_id,
        &decode_asset_cookie_secret(TEST_ASSET_COOKIE_SECRET_HEX)
            .expect("test asset cookie secret is hex"),
    );
    let cookie = format!("ind_asset={cookie}");
    let paths = [
        format!("/api/v1/assets/documents/{document_id}/readable_html"),
        format!("/api/v1/assets/{user_id}/avatars/missing.png"),
    ];

    for path in paths {
        let response = scenario
            .app
            .client()
            .get(format!("{}{}", scenario.app.address, path))
            .header(header::COOKIE, &cookie)
            .send()
            .await
            .expect("suspended asset-cookie request");
        assert_eq!(
            response.status(),
            StatusCode::UNAUTHORIZED,
            "a suspended user asset cookie must not reach {path}"
        );
    }
}
