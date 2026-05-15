use crate::credentials;
use crate::error::HudError;
use crate::poller::FetchStatus;
use crate::store::{HudMode, Sample, Settings, Store};
use crate::tray;
use crate::usage_client::{self, UsageSnapshot};
use chrono::Utc;
use serde::Serialize;
use std::sync::Arc;
use tauri::{AppHandle, Emitter};

#[derive(Debug, Serialize)]
pub struct Bootstrap {
    pub snapshot: Option<UsageSnapshot>,
    pub history: Vec<Sample>,
    pub status: FetchStatus,
    pub settings: Settings,
}

#[tauri::command]
pub async fn bootstrap(store: tauri::State<'_, Arc<Store>>) -> Result<Bootstrap, String> {
    let cached = store.snapshot();
    if let Some(s) = cached {
        return Ok(Bootstrap {
            status: FetchStatus::Stale {
                at: s.fetched_at.clone(),
                reason: "rehydrated".into(),
            },
            snapshot: Some(s),
            history: store.history(),
            settings: store.settings(),
        });
    }

    let (snapshot, status) = match fetch_once().await {
        Ok(s) => {
            let sample = Sample {
                t: Utc::now().to_rfc3339(),
                five_hour: s.five_hour.as_ref().map(|w| w.utilization),
                seven_day: s.seven_day.as_ref().map(|w| w.utilization),
            };
            store.record(s.clone(), sample);
            (
                Some(s),
                FetchStatus::Ok {
                    at: Utc::now().to_rfc3339(),
                },
            )
        }
        Err(HudError::AuthRequired) => (None, FetchStatus::AuthRequired),
        Err(e) => (
            None,
            FetchStatus::Error {
                reason: e.to_string(),
            },
        ),
    };

    Ok(Bootstrap {
        snapshot,
        status,
        history: store.history(),
        settings: store.settings(),
    })
}

async fn fetch_once() -> Result<UsageSnapshot, HudError> {
    let creds = credentials::load()?;
    let client = usage_client::build_client();
    usage_client::fetch_usage(&creds.access_token, &client).await
}

#[tauri::command]
pub fn set_mode(app: AppHandle, store: tauri::State<'_, Arc<Store>>, mode: HudMode) {
    store.set_mode(mode);
    tray::resize_for_mode(&app, mode);
    let _ = app.emit("settings://mode", &mode);
}
