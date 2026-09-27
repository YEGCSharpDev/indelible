use std::sync::Arc;

use ind_persistence::repos::{
    PgCollectionRepository, PgRefreshTokenRepository, PgTagRepository, PgUsageCounterRepository,
    PgUserRepository,
};
use sqlx::PgPool;

pub(crate) struct Repositories {
    pub collection: Arc<PgCollectionRepository>,
    pub refresh_token: Arc<PgRefreshTokenRepository>,
    pub tag: Arc<PgTagRepository>,
    pub usage_counter: Arc<PgUsageCounterRepository>,
    pub user: Arc<PgUserRepository>,
}

impl Repositories {
    pub fn new(pool: &PgPool) -> Self {
        Self {
            collection: Arc::new(PgCollectionRepository::new(pool.clone())),
            refresh_token: Arc::new(PgRefreshTokenRepository::new(pool.clone())),
            tag: Arc::new(PgTagRepository::new(pool.clone())),
            usage_counter: Arc::new(PgUsageCounterRepository::new(pool.clone())),
            user: Arc::new(PgUserRepository::new(pool.clone())),
        }
    }
}
