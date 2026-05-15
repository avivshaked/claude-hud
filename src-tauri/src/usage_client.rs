use crate::error::{HudError, Result};
use chrono::Utc;
use serde::{Deserialize, Serialize};

const ENDPOINT: &str = "https://api.anthropic.com/api/oauth/usage";
const USER_AGENT: &str = "claude-cli/2.0.42 (external, cli)";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowUsage {
    pub utilization: f64,
    pub resets_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UsageSnapshot {
    pub five_hour: Option<WindowUsage>,
    pub seven_day: Option<WindowUsage>,
    pub seven_day_opus: Option<WindowUsage>,
    pub seven_day_sonnet: Option<WindowUsage>,
    pub fetched_at: String,
}

#[derive(Debug, Deserialize)]
struct RawUsage {
    five_hour: Option<RawWindow>,
    seven_day: Option<RawWindow>,
    seven_day_opus: Option<RawWindow>,
    seven_day_sonnet: Option<RawWindow>,
}

#[derive(Debug, Deserialize)]
struct RawWindow {
    utilization: Option<f64>,
    resets_at: Option<String>,
}

fn to_window(raw: Option<RawWindow>) -> Option<WindowUsage> {
    raw.and_then(|w| w.utilization.map(|u| WindowUsage {
        utilization: u,
        resets_at: w.resets_at,
    }))
}

pub async fn fetch_usage(token: &str, client: &reqwest::Client) -> Result<UsageSnapshot> {
    let resp = client
        .get(ENDPOINT)
        .header("Authorization", format!("Bearer {token}"))
        .header("User-Agent", USER_AGENT)
        .header("Accept", "application/json")
        .send()
        .await?;

    let status = resp.status();
    if status == reqwest::StatusCode::UNAUTHORIZED {
        return Err(HudError::AuthRequired);
    }
    if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
        let retry_after = resp
            .headers()
            .get("retry-after")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(60);
        return Err(HudError::RateLimited { retry_after });
    }
    if !status.is_success() {
        let body = resp.text().await.unwrap_or_default();
        return Err(HudError::Http(format!("status {status}: {body}")));
    }
    let raw: RawUsage = resp.json().await?;
    Ok(UsageSnapshot {
        five_hour: to_window(raw.five_hour),
        seven_day: to_window(raw.seven_day),
        seven_day_opus: to_window(raw.seven_day_opus),
        seven_day_sonnet: to_window(raw.seven_day_sonnet),
        fetched_at: Utc::now().to_rfc3339(),
    })
}

pub fn build_client() -> reqwest::Client {
    reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .expect("reqwest client")
}
