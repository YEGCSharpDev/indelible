use chrono::Utc;
use ind_domain::{ApiPermission, ApiTokenId, ClientType, Theme, User, UserId, UserStatus};

use super::{
    EXTENSION_ACCESS_POLICY, MOBILE_ACCESS_POLICY, USER_ACCESS_POLICY, VERIFIED_USER_ACCESS_POLICY,
    VERIFIED_WEB_ACCESS_POLICY, WEB_ACCESS_POLICY, authorize_jwt_access,
};
use crate::error::ApiError;
use crate::middleware::{ApiCredential, Principal};

fn principal(credential: ApiCredential) -> Principal {
    let user_id = UserId::new();
    let now = Utc::now();
    Principal {
        user: User {
            id: user_id,
            username: "reader".to_string(),
            password_hash: None,
            display_name: "Reader".to_string(),
            avatar_url: None,
            locale: None,
            timezone: "UTC".to_string(),
            theme: Theme::System,
            onboarding_completed: true,
            onboarding_step: 0,
            status: UserStatus::Active,
            created_at: now,
            updated_at: now,
        },
        user_id,
        credential,
    }
}

fn jwt(client_type: ClientType) -> Principal {
    principal(ApiCredential::UserAccessJwt { client_type })
}

fn assert_forbidden(error: ApiError, expected: &str) {
    let ApiError::Forbidden { message } = error else {
        panic!("expected forbidden, got {error:?}");
    };
    assert_eq!(message, expected);
}

#[test]
fn user_access_jwt_policy_accepts_all_ordinary_clients_and_rejects_other_credentials() {
    for client_type in [
        ClientType::Web,
        ClientType::Ios,
        ClientType::Android,
        ClientType::Desktop,
        ClientType::Cli,
    ] {
        authorize_jwt_access(&jwt(client_type), USER_ACCESS_POLICY)
            .unwrap_or_else(|error| panic!("{client_type:?} JWT was denied: {error:?}"));
    }

    let extension_error = authorize_jwt_access(&jwt(ClientType::Extension), USER_ACCESS_POLICY)
        .expect_err("Extension must remain isolated from ordinary user access");
    assert_forbidden(extension_error, "account session required");

    let pat = principal(ApiCredential::PersonalAccessToken {
        token_id: ApiTokenId::new(),
        permissions: vec![ApiPermission::LibraryRead],
    });
    let pat_error = authorize_jwt_access(&pat, USER_ACCESS_POLICY)
        .expect_err("PAT must not satisfy a JWT-only policy");
    assert_forbidden(pat_error, "account session required");
}

#[test]
fn verified_user_access_jwt_policy_accepts_ordinary_clients() {
    authorize_jwt_access(&jwt(ClientType::Desktop), VERIFIED_USER_ACCESS_POLICY)
        .expect("desktop JWT must be accepted");

    authorize_jwt_access(&jwt(ClientType::Cli), VERIFIED_USER_ACCESS_POLICY)
        .expect("CLI JWT must be accepted");

    let extension_error =
        authorize_jwt_access(&jwt(ClientType::Extension), VERIFIED_USER_ACCESS_POLICY)
            .expect_err("Extension must be denied");
    assert_forbidden(extension_error, "user access JWT required");
}

#[test]
fn verified_web_access_jwt_policy_requires_web() {
    let mobile_error = authorize_jwt_access(&jwt(ClientType::Ios), VERIFIED_WEB_ACCESS_POLICY)
        .expect_err("non-Web JWT must be denied");
    assert_forbidden(mobile_error, "web access required");

    authorize_jwt_access(&jwt(ClientType::Web), VERIFIED_WEB_ACCESS_POLICY)
        .expect("Web JWT must be accepted");
}

#[test]
fn web_access_compatibility_policy_accepts_only_web() {
    authorize_jwt_access(&jwt(ClientType::Web), WEB_ACCESS_POLICY)
        .expect("the extension exchange must accept a Web JWT");

    for client_type in [ClientType::Ios, ClientType::Extension] {
        let error = authorize_jwt_access(&jwt(client_type), WEB_ACCESS_POLICY)
            .expect_err("non-Web JWT must be denied");
        assert_forbidden(error, "web access required");
    }

    let pat = principal(ApiCredential::PersonalAccessToken {
        token_id: ApiTokenId::new(),
        permissions: vec![ApiPermission::LibraryRead],
    });
    let error = authorize_jwt_access(&pat, WEB_ACCESS_POLICY)
        .expect_err("PAT must not satisfy the Web-only compatibility policy");
    assert_forbidden(error, "web access required");
}

#[test]
fn extension_access_jwt_policy_accepts_only_extension() {
    authorize_jwt_access(&jwt(ClientType::Extension), EXTENSION_ACCESS_POLICY)
        .expect("Extension access must be accepted");

    let error = authorize_jwt_access(&jwt(ClientType::Web), EXTENSION_ACCESS_POLICY)
        .expect_err("Web JWT must not satisfy Extension-only access");
    assert_forbidden(error, "extension access required");
}

#[test]
fn mobile_access_jwt_policy_accepts_only_ios_and_android() {
    for client_type in [ClientType::Ios, ClientType::Android] {
        authorize_jwt_access(&jwt(client_type), MOBILE_ACCESS_POLICY)
            .unwrap_or_else(|error| panic!("{client_type:?} JWT was denied: {error:?}"));
    }

    for client_type in [ClientType::Web, ClientType::Desktop, ClientType::Cli] {
        let error = authorize_jwt_access(&jwt(client_type), MOBILE_ACCESS_POLICY)
            .expect_err("non-mobile JWT must be denied");
        assert_forbidden(error, "mobile access required");
    }
}
