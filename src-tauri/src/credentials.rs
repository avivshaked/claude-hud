use crate::error::{HudError, Result};
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug, Deserialize)]
struct CredsFile {
    #[serde(rename = "claudeAiOauth")]
    claude_ai_oauth: ClaudeAiOauth,
}

#[derive(Debug, Deserialize)]
struct ClaudeAiOauth {
    #[serde(rename = "accessToken")]
    access_token: String,
    #[serde(rename = "refreshToken")]
    #[allow(dead_code)]
    refresh_token: Option<String>,
    #[serde(rename = "expiresAt")]
    #[allow(dead_code)]
    expires_at: Option<i64>,
}

pub struct Creds {
    pub access_token: String,
}

fn credentials_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".claude")
        .join(".credentials.json")
}

pub fn load() -> Result<Creds> {
    let path = credentials_path();
    if !path.exists() {
        return Err(HudError::CredsMissing(path.display().to_string()));
    }
    let raw = std::fs::read_to_string(&path)?;
    let parsed: CredsFile = serde_json::from_str(&raw)
        .map_err(|e| HudError::CredsInvalid(e.to_string()))?;
    Ok(Creds {
        access_token: parsed.claude_ai_oauth.access_token,
    })
}
