use std::sync::Arc;

use futures::future::BoxFuture;
use ind_application::AppError;
use ind_application::ports::{
    IntegrationAuthorizeStart, IntegrationOperations, IntegrationSyncEnqueued,
};
use ind_application::repos::outbox::JobOutboxRepository;
use ind_domain::UserId;

// -- IntegrationOperations --

use ind_auth::integration_oauth_error_to_app_error as map_integration_oauth_err;

pub struct IntegrationOperationsService {
    connection_repo:
        Arc<dyn ind_application::repos::integration_connection::IntegrationConnectionRepository>,
    oauth_token_repo:
        Arc<dyn ind_application::repos::integration_oauth_token::IntegrationOAuthTokenRepository>,
    sync_service: crate::integration_sync::IntegrationSyncService,
    oauth_service: Arc<ind_auth::integration_oauth::IntegrationOAuthService>,
    credential_cipher: Option<Arc<ind_auth::CredentialCipher>>,
}

impl IntegrationOperationsService {
    pub fn new(
        connection_repo: Arc<
            dyn ind_application::repos::integration_connection::IntegrationConnectionRepository,
        >,
        oauth_token_repo: Arc<
            dyn ind_application::repos::integration_oauth_token::IntegrationOAuthTokenRepository,
        >,
        outbox_repo: Arc<dyn JobOutboxRepository>,
        oauth_service: Arc<ind_auth::integration_oauth::IntegrationOAuthService>,
        credential_cipher: Option<Arc<ind_auth::CredentialCipher>>,
    ) -> Self {
        let sync_service = crate::integration_sync::IntegrationSyncService::new(
            connection_repo.clone(),
            outbox_repo,
        );
        Self {
            connection_repo,
            oauth_token_repo,
            sync_service,
            oauth_service,
            credential_cipher,
        }
    }

    fn require_cipher(&self) -> Result<&ind_auth::CredentialCipher, AppError> {
        self.credential_cipher
            .as_deref()
            .ok_or_else(|| AppError::ExternalService {
                service: "integration_oauth".to_string(),
                message: "auth.credential_key is required for integration OAuth flows".to_string(),
            })
    }
}

fn integration_connection_config_from_tokens(
    provider: ind_domain::IntegrationOAuthProvider,
    extra: &serde_json::Value,
) -> serde_json::Value {
    match provider {
        ind_domain::IntegrationOAuthProvider::Custom => extra.clone(),
    }
}

impl IntegrationOperations for IntegrationOperationsService {
    fn configured_oauth_providers(&self) -> Vec<ind_domain::IntegrationOAuthProvider> {
        // The callback seals the returned tokens with the credential cipher.
        // Without it the flow can start but can never be stored, so a
        // provider missing the key is not actually connectable.
        if self.credential_cipher.is_none() {
            return Vec::new();
        }
        self.oauth_service.configured_providers()
    }

    fn list_connections(
        &self,
        user_id: UserId,
    ) -> BoxFuture<'_, Result<Vec<ind_domain::IntegrationConnection>, AppError>> {
        Box::pin(async move { self.connection_repo.list_by_user(user_id).await })
    }

    fn pending_jobs_per_connection(
        &self,
        user_id: UserId,
    ) -> BoxFuture<
        '_,
        Result<std::collections::HashMap<ind_domain::IntegrationConnectionId, u32>, AppError>,
    > {
        Box::pin(async move {
            self.connection_repo
                .count_pending_jobs_per_connection(user_id)
                .await
        })
    }

    fn authorize(
        &self,
        user_id: UserId,
        provider: ind_domain::IntegrationOAuthProvider,
        redirect_after: Option<String>,
    ) -> BoxFuture<'_, Result<IntegrationAuthorizeStart, AppError>> {
        Box::pin(async move {
            let started = self
                .oauth_service
                .start(user_id, provider, redirect_after)
                .await
                .map_err(map_integration_oauth_err)?;
            Ok(IntegrationAuthorizeStart {
                authorize_url: started.authorize_url,
            })
        })
    }

    fn callback(
        &self,
        provider: ind_domain::IntegrationOAuthProvider,
        code: &str,
        state: &str,
    ) -> BoxFuture<'_, Result<ind_domain::IntegrationConnection, AppError>> {
        let code = code.to_string();
        let state = state.to_string();
        Box::pin(async move {
            let cipher = self.require_cipher()?;
            let completed = self
                .oauth_service
                .complete(provider, &code, &state)
                .await
                .map_err(map_integration_oauth_err)?;

            let access_enc = cipher.seal(completed.tokens.access_token.as_bytes());
            let refresh_enc = completed
                .tokens
                .refresh_token
                .as_ref()
                .map(|rt| cipher.seal(rt.as_bytes()));

            self.oauth_token_repo
                .upsert(
                    completed.user_id,
                    completed.provider,
                    access_enc,
                    refresh_enc,
                    completed.tokens.expires_at,
                    completed.tokens.extra.clone(),
                )
                .await?;

            let connection_provider = match completed.provider {
                ind_domain::IntegrationOAuthProvider::Custom => {
                    ind_domain::IntegrationProvider::Custom
                }
            };

            let config = integration_connection_config_from_tokens(
                completed.provider,
                &completed.tokens.extra,
            );
            let connection = self
                .connection_repo
                .upsert_by_user_provider(completed.user_id, connection_provider, config, "active")
                .await?;

            Ok(connection)
        })
    }

    fn delete_connection(
        &self,
        user_id: UserId,
        connection_id: ind_domain::IntegrationConnectionId,
    ) -> BoxFuture<'_, Result<(), AppError>> {
        Box::pin(async move {
            let connection = self
                .connection_repo
                .find_by_id(user_id, connection_id)
                .await?
                .ok_or_else(|| {
                    AppError::Domain(ind_domain::DomainError::NotFound {
                        entity: "IntegrationConnection",
                        id: connection_id.to_string(),
                    })
                })?;

            let oauth_provider = match connection.provider {
                ind_domain::IntegrationProvider::Custom => {
                    Some(ind_domain::IntegrationOAuthProvider::Custom)
                }
                _ => None,
            };

            // Revoke the upstream grant BEFORE deleting anything local. A grant
            // left installed after "disconnect" is a live credential the user
            // believes is dead, so failure keeps both rows for a retry. A token
            // row can only exist if a cipher sealed it, which makes a missing
            // cipher here a misconfiguration rather than a skippable step.
            if let Some(op) = oauth_provider
                && let Some(token_row) = self
                    .oauth_token_repo
                    .find_by_user_provider(user_id, op)
                    .await?
            {
                let cipher = self.require_cipher()?;
                let access_token = cipher
                    .open(&token_row.access_token_enc)
                    .ok()
                    .and_then(|bytes| String::from_utf8(bytes).ok())
                    .ok_or_else(|| AppError::ExternalService {
                        service: "integration_oauth".to_string(),
                        message: "stored provider token could not be decrypted".to_string(),
                    })?;
                self.oauth_service
                    .revoke(op, &access_token)
                    .await
                    .map_err(map_integration_oauth_err)?;
            }

            // Token first: if the token delete fails, the connection is still
            // present so disconnect can be retried (revocation is idempotent).
            // The reverse order would strand an orphaned token row behind a
            // connection lookup that no longer succeeds.
            if let Some(op) = oauth_provider {
                self.oauth_token_repo
                    .delete_by_user_provider(user_id, op)
                    .await?;
            }
            self.connection_repo.delete(connection_id, user_id).await?;
            Ok(())
        })
    }

    fn sync_now(
        &self,
        user_id: UserId,
        connection_id: ind_domain::IntegrationConnectionId,
    ) -> BoxFuture<'_, Result<IntegrationSyncEnqueued, AppError>> {
        Box::pin(self.sync_service.sync_now(user_id, connection_id))
    }

    fn setup_miniflux_connection(
        &self,
        user_id: UserId,
        url: String,
        api_key: String,
    ) -> BoxFuture<'_, Result<ind_domain::IntegrationConnection, AppError>> {
        Box::pin(async move {
            let config = serde_json::json!({
                "url": url,
                "api_key": api_key,
            });
            self.connection_repo
                .upsert_by_user_provider(
                    user_id,
                    ind_domain::IntegrationProvider::Miniflux,
                    config,
                    "connected",
                )
                .await
        })
    }
}
