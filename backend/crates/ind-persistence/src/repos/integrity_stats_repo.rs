use async_trait::async_trait;
use sqlx::PgPool;

use ind_application::error::AppError;
use ind_application::repos::integrity::{IntegrityStats, IntegrityStatsRepository};

pub struct PgIntegrityStatsRepository {
    pool: PgPool,
}

impl PgIntegrityStatsRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl IntegrityStatsRepository for PgIntegrityStatsRepository {
    async fn stats(&self) -> Result<IntegrityStats, AppError> {
        let row = sqlx::query_as::<_, IntegrityStatsRow>(
            r#"
            SELECT
                (
                    SELECT COUNT(DISTINCT d.id)::BIGINT
                    FROM documents d
                    WHERE (
                            EXISTS (
                                SELECT 1
                                FROM library_entries le
                                WHERE le.document_id = d.id
                                  AND le.deleted_at IS NULL
                            )
                            OR EXISTS (
                                SELECT 1
                                FROM feed_deliveries fd
                                WHERE fd.document_id = d.id
                                  AND fd.dismissed_at IS NULL
                                  AND fd.hidden_at IS NULL
                            )
                        )
                      AND NOT EXISTS (
                            SELECT 1
                            FROM search_documents sd
                            WHERE sd.document_id = d.id
                        )
                ) AS documents_missing_search_rows,
                (
                    SELECT COUNT(*)::BIGINT
                    FROM archive_assets aa
                    WHERE aa.document_id IS NOT NULL
                      AND aa.status IN ('failed', 'degraded')
                      AND aa.asset_kind IN (
                            'readable_html',
                            'monolith',
                            'pdf',
                            'screenshot',
                            'thumbnail',
                            'warc',
                            'epub',
                            'extracted_text'
                      )
                ) AS failed_derived_assets,
                (
                    SELECT COUNT(*)::BIGINT
                    FROM dead_letter_jobs
                    WHERE replayed_at IS NULL
                ) AS dead_letter_jobs
            "#,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|err| AppError::Repository(Box::new(err)))?;

        Ok(IntegrityStats {
            documents_missing_search_rows: row.documents_missing_search_rows.unwrap_or(0),
            failed_derived_assets: row.failed_derived_assets.unwrap_or(0),
            dead_letter_jobs: row.dead_letter_jobs.unwrap_or(0),
        })
    }
}

#[derive(sqlx::FromRow)]
struct IntegrityStatsRow {
    documents_missing_search_rows: Option<i64>,
    failed_derived_assets: Option<i64>,
    dead_letter_jobs: Option<i64>,
}
