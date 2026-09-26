use std::sync::Arc;

use ind_persistence::repos::{
    PgHighlightRepository, PgIntegrationConnectionRepository, PgIntegrationOAuthTokenRepository,
};
use sqlx::PgPool;

pub struct Repositories {
    pub highlight: Arc<PgHighlightRepository>,
    pub integration_connection: Arc<PgIntegrationConnectionRepository>,
    pub integration_oauth_token: Arc<PgIntegrationOAuthTokenRepository>,
}

impl Repositories {
    pub fn new(pool: &PgPool) -> Self {
        Self {
            highlight: Arc::new(PgHighlightRepository::new(pool.clone())),
            integration_connection: Arc::new(PgIntegrationConnectionRepository::new(pool.clone())),
            integration_oauth_token: Arc::new(PgIntegrationOAuthTokenRepository::new(pool.clone())),
        }
    }
}
