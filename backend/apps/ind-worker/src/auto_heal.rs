use std::sync::Arc;
use std::time::Duration;

use chrono::Utc;
use ind_application::AppError;
use ind_application::repos::integrity::{IntegrityStats, IntegrityStatsRepository};
use ind_application::repos::maintenance::MaintenanceTaskLease;

use crate::context::RecoveryJobDeps;

const INTEGRITY_TASK: &str = "integrity.check";
const MAINTENANCE_FAILURE_RETRY_SECS: u64 = 60;

pub async fn run_auto_heal_loop(ctx: Arc<RecoveryJobDeps>) {
    run_auto_heal_once(&ctx).await;

    let mut interval = tokio::time::interval(Duration::from_secs(ctx.auto_heal_interval_secs));
    interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
    interval.tick().await;

    loop {
        interval.tick().await;
        run_auto_heal_once(&ctx).await;
    }
}

pub async fn run_auto_heal_once(ctx: &RecoveryJobDeps) {
    let now = Utc::now();

    crate::recovery_sweeper::sweep_background_recoveries(
        &ctx.background_recovery_repo,
        &ctx.worker_id,
        ctx.job_recovery_max_attempts,
        ctx.job_recovery_batch_size,
        ctx.auto_heal_lease_secs,
        now,
    )
    .await;

    run_integrity_check_if_due(ctx).await;
}

pub async fn sweep_integrity_stats(
    repo: &dyn IntegrityStatsRepository,
) -> Result<IntegrityStats, AppError> {
    repo.stats().await
}

async fn run_integrity_check_if_due(ctx: &RecoveryJobDeps) {
    let Some(_) = acquire_maintenance(ctx, INTEGRITY_TASK, Utc::now()).await else {
        return;
    };
    match sweep_integrity_stats(ctx.integrity_stats_repo.as_ref()).await {
        Ok(stats) => {
            log_integrity_stats(&stats);
            let completed_at = Utc::now();
            complete_maintenance(
                ctx,
                INTEGRITY_TASK,
                schedule_after(completed_at, ctx.integrity_interval_secs),
                None,
                completed_at,
            )
            .await;
        }
        Err(error) => {
            tracing::warn!(%error, "integrity stats sweep failed");
            fail_maintenance(ctx, INTEGRITY_TASK, &error).await;
        }
    }
}

async fn acquire_maintenance(
    ctx: &RecoveryJobDeps,
    task_name: &str,
    now: chrono::DateTime<Utc>,
) -> Option<MaintenanceTaskLease> {
    match ctx
        .maintenance_task_repo
        .try_acquire(
            task_name,
            &ctx.worker_id,
            now,
            now + chrono::Duration::seconds(ctx.maintenance_lease_secs.max(1)),
        )
        .await
    {
        Ok(lease) => lease,
        Err(error) => {
            tracing::warn!(%error, task_name, "maintenance task lease acquisition failed");
            None
        }
    }
}

async fn complete_maintenance(
    ctx: &RecoveryJobDeps,
    task_name: &str,
    next_run_at: chrono::DateTime<Utc>,
    continuation_cursor: Option<&str>,
    now: chrono::DateTime<Utc>,
) {
    if let Err(error) = ctx
        .maintenance_task_repo
        .complete(
            task_name,
            &ctx.worker_id,
            next_run_at,
            continuation_cursor,
            now,
        )
        .await
    {
        tracing::warn!(%error, task_name, "maintenance task completion failed");
    }
}

async fn fail_maintenance(ctx: &RecoveryJobDeps, task_name: &str, task_error: &AppError) {
    let failed_at = Utc::now();
    if let Err(error) = ctx
        .maintenance_task_repo
        .fail(
            task_name,
            &ctx.worker_id,
            schedule_after(failed_at, MAINTENANCE_FAILURE_RETRY_SECS),
            &task_error.to_string(),
            failed_at,
        )
        .await
    {
        tracing::warn!(%error, task_name, "maintenance task failure release failed");
    }
}

fn schedule_after(now: chrono::DateTime<Utc>, seconds: u64) -> chrono::DateTime<Utc> {
    now + chrono::Duration::seconds(i64::try_from(seconds).unwrap_or(i64::MAX))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IntegrityStatsLogSeverity {
    Info,
    Warn,
}

pub fn integrity_stats_log_severity(stats: &IntegrityStats) -> IntegrityStatsLogSeverity {
    if stats.documents_missing_search_rows > 0
        || stats.failed_derived_assets > 0
        || stats.dead_letter_jobs > 0
    {
        IntegrityStatsLogSeverity::Warn
    } else {
        IntegrityStatsLogSeverity::Info
    }
}

fn log_integrity_stats(stats: &IntegrityStats) {
    match integrity_stats_log_severity(stats) {
        IntegrityStatsLogSeverity::Info => tracing::info!(
            documents_missing_search_rows = stats.documents_missing_search_rows,
            failed_derived_assets = stats.failed_derived_assets,
            dead_letter_jobs = stats.dead_letter_jobs,
            "integrity stats sweep finished"
        ),
        IntegrityStatsLogSeverity::Warn => tracing::warn!(
            documents_missing_search_rows = stats.documents_missing_search_rows,
            failed_derived_assets = stats.failed_derived_assets,
            dead_letter_jobs = stats.dead_letter_jobs,
            "integrity stats sweep found issues"
        ),
    }
}
