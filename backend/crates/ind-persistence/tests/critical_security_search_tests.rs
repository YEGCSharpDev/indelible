use chrono::Utc;
use ind_application::repos::api_token::ApiTokenRepository;
use ind_domain::{ApiPermission, ApiToken, ApiTokenId, UserId};
use ind_persistence::repos::PgApiTokenRepository;
use ind_test_support::{TestDb, UserFactory};

fn api_token(user_id: UserId) -> ApiToken {
    ApiToken {
        id: ApiTokenId::new(),
        user_id,
        name: "automation".into(),
        token_hash: uuid::Uuid::now_v7().to_string(),
        prefix: "ind_test".into(),
        permissions: vec![ApiPermission::LibraryRead],
        last_used_at: None,
        expires_at: Some(Utc::now() + chrono::Duration::hours(1)),
        created_at: Utc::now(),
    }
}

#[tokio::test]
async fn api_token_mutations_are_owner_scoped_and_revocation_is_immediate() {
    let db = TestDb::new().await;
    let owner = UserFactory::new().insert(db.pool()).await;
    let foreign = UserFactory::new().insert(db.pool()).await;
    let repo = PgApiTokenRepository::new(db.pool().clone());
    let token = repo.create(api_token(owner.id)).await.unwrap();

    assert!(
        repo.find_by_id(token.id, foreign.id)
            .await
            .unwrap()
            .is_none()
    );
    assert!(repo.delete(token.id, foreign.id).await.is_err());
    assert!(
        repo.find_by_token_hash(&token.token_hash)
            .await
            .unwrap()
            .is_some()
    );

    repo.delete(token.id, owner.id).await.unwrap();
    assert!(
        repo.find_by_token_hash(&token.token_hash)
            .await
            .unwrap()
            .is_none()
    );
}
