use std::sync::Arc;

use ind_domain::{IntegrationOAuthProvider, UserId};
use ind_persistence::repos::PgOAuthFlowRepository;
use ind_test_support::TestDb;

use super::{
    IntegrationOAuthError, IntegrationOAuthProviderAdapter, IntegrationOAuthService,
    ProviderTokens, RepositoryIntegrationOAuthFlowStore, integration_oauth_error_to_app_error,
};

struct CustomAdapter;

#[async_trait::async_trait]
impl IntegrationOAuthProviderAdapter for CustomAdapter {
    fn provider(&self) -> IntegrationOAuthProvider {
        IntegrationOAuthProvider::Custom
    }

    fn authorize_url(&self, state: &str, redirect_uri: &str) -> String {
        format!("https://custom.example/authorize?state={state}&redirect_uri={redirect_uri}")
    }

    async fn exchange_code(
        &self,
        code: &str,
        state: &str,
    ) -> Result<ProviderTokens, IntegrationOAuthError> {
        assert_eq!(code, "provider-code");
        assert!(!state.is_empty());
        Ok(ProviderTokens {
            access_token: "custom-access".into(),
            refresh_token: Some("custom-refresh".into()),
            expires_at: None,
            extra: serde_json::json!({"workspace_id": "workspace-1"}),
        })
    }

    async fn revoke_token(&self, access_token: &str) -> Result<(), IntegrationOAuthError> {
        assert!(!access_token.is_empty());
        Ok(())
    }
}

#[tokio::test]
async fn sealed_flow_round_trips_once_with_provider_scope_and_error_projection() {
    let db = TestDb::new().await;
    let store = Arc::new(RepositoryIntegrationOAuthFlowStore::new(Arc::new(
        PgOAuthFlowRepository::new(db.pool().clone()),
    )));
    let service = IntegrationOAuthService::new(
        vec![Arc::new(CustomAdapter)],
        store,
        b"integration-oauth-boundary-secret",
        "https://api.example.com".into(),
    );
    assert_eq!(
        service.configured_providers(),
        vec![IntegrationOAuthProvider::Custom]
    );
    assert!(service.has_provider(IntegrationOAuthProvider::Custom));
    let user_id = UserId::new();
    let started = service
        .start(
            user_id,
            IntegrationOAuthProvider::Custom,
            Some("/settings/integrations".into()),
        )
        .await
        .unwrap();
    assert!(started.authorize_url.contains(&started.state));
    assert!(
        started
            .authorize_url
            .contains("/api/v1/integrations/custom/callback")
    );

    let completed = service
        .complete(
            IntegrationOAuthProvider::Custom,
            "provider-code",
            &started.state,
        )
        .await
        .unwrap();
    assert_eq!(completed.user_id, user_id);
    assert_eq!(completed.provider, IntegrationOAuthProvider::Custom);
    assert_eq!(completed.tokens.access_token, "custom-access");
    assert_eq!(
        completed.redirect_after.as_deref(),
        Some("/settings/integrations")
    );
    assert!(matches!(
        service
            .complete(
                IntegrationOAuthProvider::Custom,
                "provider-code",
                &started.state,
            )
            .await,
        Err(IntegrationOAuthError::InvalidState)
    ));

    let unconfigured = IntegrationOAuthService::new(
        Vec::new(),
        Arc::new(RepositoryIntegrationOAuthFlowStore::new(Arc::new(
            PgOAuthFlowRepository::new(db.pool().clone()),
        ))),
        b"integration-oauth-boundary-secret",
        "https://api.example.com".into(),
    );
    assert!(matches!(
        unconfigured
            .start(user_id, IntegrationOAuthProvider::Custom, None)
            .await,
        Err(IntegrationOAuthError::ProviderNotConfigured(
            IntegrationOAuthProvider::Custom
        ))
    ));

    for error in [
        IntegrationOAuthError::InvalidState,
        IntegrationOAuthError::ProviderMismatch,
        IntegrationOAuthError::InvalidCredentials,
        IntegrationOAuthError::Exchange("provider failed".into()),
        IntegrationOAuthError::Configuration("bad config".into()),
    ] {
        let projected = integration_oauth_error_to_app_error(error);
        assert!(!projected.to_string().is_empty());
    }
}
