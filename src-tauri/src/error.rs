use thiserror::Error;

#[derive(Debug, Error)]
pub enum HudError {
    #[error("credentials file not found at {0}")]
    CredsMissing(String),
    #[error("credentials file invalid: {0}")]
    CredsInvalid(String),
    #[error("auth required (token rejected by server)")]
    AuthRequired,
    #[error("rate limited; retry after {retry_after}s")]
    RateLimited { retry_after: u64 },
    #[error("http error: {0}")]
    Http(String),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
}

impl From<reqwest::Error> for HudError {
    fn from(e: reqwest::Error) -> Self {
        let mut msg = e.to_string();
        let mut src: &dyn std::error::Error = &e;
        while let Some(cause) = src.source() {
            msg.push_str(" — caused by: ");
            msg.push_str(&cause.to_string());
            src = cause;
        }
        HudError::Http(msg)
    }
}

impl From<HudError> for String {
    fn from(e: HudError) -> Self {
        e.to_string()
    }
}

pub type Result<T> = std::result::Result<T, HudError>;
