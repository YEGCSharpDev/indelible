use ind_application::repos::collection::CollectionRepository;
use ind_application::repos::event::MutationSideEffects;
use ind_application::repos::library::LibraryRepository;
use ind_application::repos::lifecycle_outbox::search_reindex_document_outbox;
use ind_domain::DocumentType;
use ind_persistence::repos::{PgCollectionRepository, PgLibraryRepository};
use ind_test_support::{
    CollectionFactory, DocumentFactory, LibraryEntryFactory, TestDb, UserFactory,
};

async fn saved_entry(
    pool: &sqlx::PgPool,
    user_id: ind_domain::UserId,
) -> (ind_domain::DocumentId, ind_domain::LibraryEntryId) {
    let document = DocumentFactory::new(user_id)
        .with_document_type(DocumentType::Article)
        .insert(pool)
        .await;
    let entry = LibraryEntryFactory::new(user_id, document.id)
        .insert(pool)
        .await;
    (document.id, entry.id)
}

#[tokio::test]
async fn purge_keeps_document_and_cascades_membership() {
    let db = TestDb::new().await;
    let pool = db.pool().clone();
    let user = UserFactory::default().insert(&pool).await;
    let collections = PgCollectionRepository::new(pool.clone());
    let library = PgLibraryRepository::new(pool.clone());

    let (document_id, entry_id) = saved_entry(&pool, user.id).await;
    let collection = CollectionFactory::new(user.id).insert(&pool).await;
    collections
        .add_library_entry_to_collection(user.id, collection.id, entry_id)
        .await
        .unwrap();

    library
        .purge(entry_id, user.id, MutationSideEffects::none())
        .await
        .unwrap();

    let document_exists =
        sqlx::query_scalar::<_, bool>("SELECT EXISTS(SELECT 1 FROM documents WHERE id = $1)")
            .bind(document_id.into_uuid())
            .fetch_one(&pool)
            .await
            .unwrap();
    assert!(document_exists);
    assert_eq!(collections.count_items(collection.id).await.unwrap(), 0);
}

#[tokio::test]
async fn trash_restore_and_purge_are_tenant_scoped_with_atomic_side_effects() {
    let db = TestDb::new().await;
    let pool = db.pool().clone();
    let owner = UserFactory::default().insert(&pool).await;
    let foreign = UserFactory::default().insert(&pool).await;
    let (document_id, entry_id) = saved_entry(&pool, owner.id).await;
    let library = PgLibraryRepository::new(pool.clone());
    let side_effects = || {
        MutationSideEffects::with_outbox(search_reindex_document_outbox(
            document_id,
            chrono::Utc::now(),
        ))
    };

    assert!(
        library
            .soft_delete(entry_id, foreign.id, side_effects())
            .await
            .is_err()
    );
    assert!(
        library
            .find_by_id(entry_id, owner.id)
            .await
            .unwrap()
            .unwrap()
            .entry
            .deleted_at
            .is_none()
    );

    library
        .soft_delete(entry_id, owner.id, side_effects())
        .await
        .unwrap();
    library
        .restore(entry_id, owner.id, side_effects())
        .await
        .unwrap();
    library
        .purge(entry_id, owner.id, side_effects())
        .await
        .unwrap();

    let events: Vec<String> = sqlx::query_scalar(
        "SELECT event_type FROM domain_events \
         WHERE user_id = $1 AND payload->>'library_entry_id' = $2 ORDER BY created_at, id",
    )
    .bind(owner.id.into_uuid())
    .bind(entry_id.to_string())
    .fetch_all(&pool)
    .await
    .unwrap();
    assert_eq!(
        events,
        [
            "library_entry.trashed",
            "library_entry.restored",
            "library_entry.permanently_deleted"
        ]
    );
    assert_eq!(
        sqlx::query_scalar::<_, i64>(
            "SELECT count(*) FROM job_outbox WHERE job_type = 'search.reindex_document'",
        )
        .fetch_one(&pool)
        .await
        .unwrap(),
        1
    );
}
