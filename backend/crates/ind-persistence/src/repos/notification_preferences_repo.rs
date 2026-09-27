use chrono::{DateTime, Utc};
use ind_application::AppError;
use ind_application::repos::notification_preferences::NotificationPreferencesRepository;
use ind_domain::{NotificationPreferences, UserId};
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

pub struct PgNotificationPreferencesRepository {
    pool: PgPool,
}

impl PgNotificationPreferencesRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(FromRow)]
struct NotificationPreferencesRow {
    user_id: Uuid,
    daily_review_reminder_enabled: bool,
    daily_review_reminder_time: String,
    weekly_digest_enabled: bool,
    new_highlights_sync: bool,
    feed_updates: bool,
    updated_at: DateTime<Utc>,
}

impl From<NotificationPreferencesRow> for NotificationPreferences {
    fn from(row: NotificationPreferencesRow) -> Self {
        Self {
            user_id: UserId::from_uuid(row.user_id),
            daily_review_reminder_enabled: row.daily_review_reminder_enabled,
            daily_review_reminder_time: row.daily_review_reminder_time,
            weekly_digest_enabled: row.weekly_digest_enabled,
            new_highlights_sync: row.new_highlights_sync,
            feed_updates: row.feed_updates,
            updated_at: row.updated_at,
        }
    }
}

fn map_sqlx_error(err: sqlx::Error) -> AppError {
    super::map_sqlx_error(
        "notification_preferences",
        "notification preferences already exist",
        err,
    )
}

#[async_trait::async_trait]
impl NotificationPreferencesRepository for PgNotificationPreferencesRepository {
    async fn get_by_user(
        &self,
        user_id: UserId,
    ) -> Result<Option<NotificationPreferences>, AppError> {
        let row = sqlx::query_as::<_, NotificationPreferencesRow>(
            "SELECT user_id, daily_review_reminder_enabled, daily_review_reminder_time, \
             weekly_digest_enabled, new_highlights_sync, feed_updates, \
             updated_at \
             FROM notification_preferences WHERE user_id = $1",
        )
        .bind(user_id.into_uuid())
        .fetch_optional(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        Ok(row.map(Into::into))
    }

    async fn upsert(
        &self,
        preferences: &NotificationPreferences,
    ) -> Result<NotificationPreferences, AppError> {
        let row = sqlx::query_as::<_, NotificationPreferencesRow>(
            "INSERT INTO notification_preferences \
             (user_id, daily_review_reminder_enabled, daily_review_reminder_time, \
              weekly_digest_enabled, new_highlights_sync, feed_updates, updated_at) \
             VALUES ($1, $2, $3, $4, $5, $6, $7) \
             ON CONFLICT (user_id) DO UPDATE SET \
               daily_review_reminder_enabled = EXCLUDED.daily_review_reminder_enabled, \
               daily_review_reminder_time = EXCLUDED.daily_review_reminder_time, \
               weekly_digest_enabled = EXCLUDED.weekly_digest_enabled, \
               new_highlights_sync = EXCLUDED.new_highlights_sync, \
               feed_updates = EXCLUDED.feed_updates, \
               updated_at = EXCLUDED.updated_at \
             RETURNING user_id, daily_review_reminder_enabled, daily_review_reminder_time, \
             weekly_digest_enabled, new_highlights_sync, feed_updates, updated_at",
        )
        .bind(preferences.user_id.into_uuid())
        .bind(preferences.daily_review_reminder_enabled)
        .bind(&preferences.daily_review_reminder_time)
        .bind(preferences.weekly_digest_enabled)
        .bind(preferences.new_highlights_sync)
        .bind(preferences.feed_updates)
        .bind(preferences.updated_at)
        .fetch_one(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        Ok(row.into())
    }
}
