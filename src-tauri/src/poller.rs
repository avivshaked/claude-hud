use crate::credentials;
use crate::error::HudError;
use crate::store::{Sample, Store};
use crate::usage_client::{self, UsageSnapshot};
use chrono::Utc;
use serde::Serialize;
use std::sync::Arc;
use tauri::{AppHandle, Emitter, Manager};
use tokio::sync::Notify;
use tokio::time::Duration;

const BACKOFF_FACTOR: u64 = 2;
const BACKOFF_CEILING_SECS: u64 = 30 * 60;

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum FetchStatus {
    Ok { at: String },
    Stale { at: String, reason: String },
    Error { reason: String },
    AuthRequired,
}

#[derive(Debug, Clone, Serialize)]
pub struct UsageUpdate {
    pub snapshot: UsageSnapshot,
    pub status: FetchStatus,
    pub sample: Sample,
}

pub fn spawn(app: AppHandle, store: Arc<Store>) {
    let wake = Arc::new(Notify::new());
    app.manage(wake.clone());
    tauri::async_runtime::spawn(async move {
        let client = usage_client::build_client();
        let base_interval = store.settings().poll_interval_secs.max(60);
        let mut current_interval = base_interval;

        // If we have a fresh-enough cached snapshot from a previous run, skip
        // the first immediate fetch so rapid dev restarts don't slam the API.
        let warm_delay = match store.snapshot() {
            Some(s) => {
                let age = chrono::DateTime::parse_from_rfc3339(&s.fetched_at)
                    .map(|t| (chrono::Utc::now() - t.with_timezone(&chrono::Utc)).num_seconds())
                    .unwrap_or(i64::MAX);
                if age >= 0 && (age as u64) < base_interval {
                    let wait = base_interval - age as u64;
                    log::info!("cached snapshot is {age}s old, deferring first poll {wait}s");
                    wait
                } else {
                    0
                }
            }
            None => 0,
        };
        if warm_delay > 0 {
            tokio::time::sleep(Duration::from_secs(warm_delay)).await;
        }

        loop {
            let result = tick(&app, &store, &client).await;
            match result {
                Ok(()) => {
                    if current_interval != base_interval {
                        log::info!(
                            "poll recovered; resetting interval {current_interval}s -> {base_interval}s"
                        );
                        current_interval = base_interval;
                    }
                }
                Err(HudError::RateLimited { retry_after }) => {
                    let next = if retry_after == 0 {
                        // Server isn't asking us to back off explicitly — keep base cadence.
                        base_interval
                    } else {
                        let backoff = (current_interval.saturating_mul(BACKOFF_FACTOR))
                            .min(BACKOFF_CEILING_SECS);
                        retry_after.max(backoff)
                    };
                    log::warn!(
                        "rate limited; retry-after={retry_after}s; next poll in {next}s (was {current_interval}s)"
                    );
                    current_interval = next;
                    let _ = app.emit(
                        "usage://status",
                        &FetchStatus::Error {
                            reason: format!("rate limited (retry in {next}s)"),
                        },
                    );
                }
                Err(HudError::AuthRequired) => {
                    let _ = app.emit("usage://status", &FetchStatus::AuthRequired);
                    log::warn!("auth required");
                    current_interval = base_interval;
                }
                Err(e) => {
                    let next = current_interval
                        .saturating_mul(BACKOFF_FACTOR)
                        .min(BACKOFF_CEILING_SECS);
                    log::warn!(
                        "poll tick failed: {e}; next poll in {next}s (was {current_interval}s)"
                    );
                    current_interval = next;
                    let _ = app.emit(
                        "usage://status",
                        &FetchStatus::Error {
                            reason: e.to_string(),
                        },
                    );
                }
            }
            tokio::select! {
                _ = tokio::time::sleep(Duration::from_secs(current_interval)) => {}
                _ = wake.notified() => {
                    if current_interval != base_interval {
                        log::info!(
                            "manual refresh succeeded; resetting interval {current_interval}s -> {base_interval}s"
                        );
                        current_interval = base_interval;
                    }
                }
            }
        }
    });
}

/// Trigger a one-off poll outside the regular interval (e.g. from a tray
/// "Refresh now" item). Runs in the background and emits events normally.
pub fn refresh_now(app: AppHandle, store: Arc<Store>) {
    let wake = app.try_state::<Arc<Notify>>().map(|s| s.inner().clone());
    tauri::async_runtime::spawn(async move {
        let client = usage_client::build_client();
        match tick(&app, &store, &client).await {
            Ok(()) => {
                if let Some(w) = wake {
                    w.notify_one();
                }
            }
            Err(e) => {
                let status = match &e {
                    HudError::AuthRequired => FetchStatus::AuthRequired,
                    HudError::RateLimited { retry_after } => FetchStatus::Error {
                        reason: format!("rate limited (retry in {retry_after}s)"),
                    },
                    _ => FetchStatus::Error { reason: e.to_string() },
                };
                let _ = app.emit("usage://status", &status);
            }
        }
    });
}

async fn tick(app: &AppHandle, store: &Arc<Store>, client: &reqwest::Client) -> Result<(), HudError> {
    let creds = credentials::load()?;
    let snap = usage_client::fetch_usage(&creds.access_token, client).await?;
    let sample = Sample {
        t: Utc::now().to_rfc3339(),
        five_hour: snap.five_hour.as_ref().map(|w| w.utilization),
        seven_day: snap.seven_day.as_ref().map(|w| w.utilization),
    };
    store.record(snap.clone(), sample.clone());
    let update = UsageUpdate {
        snapshot: snap,
        status: FetchStatus::Ok { at: Utc::now().to_rfc3339() },
        sample,
    };
    let _ = app.emit("usage://updated", &update);
    Ok(())
}
