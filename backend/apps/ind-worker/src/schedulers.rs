use std::sync::Arc;
use std::time::Duration;

use ind_application::handlers::feed::FeedPollScheduleConfig;
use ind_domain::FeedPollJob;

use crate::config::WorkerConfig;
use crate::context::{FeedJobDeps, WebhookJobDeps};
use crate::jobs;

pub async fn run_feed_scheduler_loop(ctx: Arc<FeedJobDeps>, config: WorkerConfig) {
    let mut interval = tokio::time::interval(Duration::from_secs(
        config.feed.scheduler_interval_secs.max(1),
    ));

    loop {
        interval.tick().await;

        let claimed = match ctx
            .feed_repo
            .claim_due_sources(
                chrono::Utc::now(),
                &ctx.worker_id,
                config.feed.batch_size,
                chrono::Duration::seconds(config.feed.lease_secs),
            )
            .await
        {
            Ok(claimed) => claimed,
            Err(err) => {
                tracing::error!(error = %err, "feed scheduler claim failed");
                continue;
            }
        };

        for source in claimed {
            let payload = match serde_json::to_value(FeedPollJob {
                source_id: source.id,
            }) {
                Ok(payload) => payload,
                Err(err) => {
                    tracing::error!(error = %err, source_id = %source.id, "failed to serialize feed poll job");
                    let _ = ctx.feed_repo.clear_source_lease(source.id).await;
                    continue;
                }
            };

            if let Err(err) = ctx
                .outbox_repo
                .enqueue(
                    "feed.poll",
                    payload,
                    Some(format!("feed.poll:{}", source.id)),
                    chrono::Utc::now(),
                )
                .await
            {
                tracing::error!(error = %err, source_id = %source.id, "failed to enqueue feed poll job");
                let _ = ctx.feed_repo.clear_source_lease(source.id).await;
            }
        }
    }
}

pub async fn run_webhook_projector_loop(webhooks: WebhookJobDeps) {
    let mut interval = tokio::time::interval(Duration::from_secs(5));

    loop {
        interval.tick().await;
        match jobs::webhooks::project_due_webhooks(&webhooks, 100).await {
            Ok(count) if count > 0 => {
                tracing::info!(count, "webhook projector enqueued dispatches");
            }
            Ok(_) => {}
            Err(err) => {
                tracing::error!(error = %err, "webhook projector failed");
            }
        }
    }
}

pub fn feed_poll_schedule(config: &WorkerConfig) -> FeedPollScheduleConfig {
    FeedPollScheduleConfig {
        default_public_poll_interval_minutes: config.feed.default_poll_interval_minutes,
        min_public_poll_interval_minutes: config.feed.min_poll_interval_minutes,
    }
    .normalized()
}
