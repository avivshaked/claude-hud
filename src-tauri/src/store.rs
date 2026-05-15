use crate::usage_client::UsageSnapshot;
use serde::{Deserialize, Deserializer, Serialize};
use std::path::PathBuf;
use std::sync::Mutex;

const HISTORY_CAP: usize = 1440; // 24h @ 1m

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum HudMode {
    Minimal,
    Full,
}

// Custom Deserialize so an older state.json with `"mode":"tray"` migrates
// transparently to Minimal instead of failing the whole load.
impl<'de> Deserialize<'de> for HudMode {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let s = String::deserialize(d)?;
        match s.as_str() {
            "minimal" | "tray" => Ok(HudMode::Minimal),
            "full" => Ok(HudMode::Full),
            other => Err(serde::de::Error::custom(format!("unknown mode: {other}"))),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub mode: HudMode,
    pub poll_interval_secs: u64,
    pub hotkey: String,
    #[serde(default)]
    pub window_pos: Option<(i32, i32)>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            mode: HudMode::Minimal,
            poll_interval_secs: 90,
            hotkey: "Ctrl+Alt+H".to_string(),
            window_pos: None,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Sample {
    pub t: String,
    pub five_hour: Option<f64>,
    pub seven_day: Option<f64>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct Persisted {
    #[serde(default)]
    settings: Settings,
    #[serde(default)]
    snapshot: Option<UsageSnapshot>,
    #[serde(default)]
    history: Vec<Sample>,
}

pub struct Store {
    inner: Mutex<Persisted>,
    path: PathBuf,
}

fn state_path() -> PathBuf {
    let base = dirs::data_dir().unwrap_or_else(|| PathBuf::from("."));
    base.join("claude-hud").join("state.json")
}

impl Store {
    pub fn load() -> Self {
        let path = state_path();
        let inner: Persisted = std::fs::read_to_string(&path)
            .ok()
            .and_then(|s| serde_json::from_str::<Persisted>(&s).ok())
            .unwrap_or_default();
        Self {
            inner: Mutex::new(inner),
            path,
        }
    }

    fn persist_locked(&self, p: &Persisted) {
        if let Some(parent) = self.path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        if let Ok(s) = serde_json::to_string(p) {
            let _ = std::fs::write(&self.path, s);
        }
    }

    pub fn settings(&self) -> Settings {
        self.inner.lock().unwrap().settings.clone()
    }

    pub fn snapshot(&self) -> Option<UsageSnapshot> {
        self.inner.lock().unwrap().snapshot.clone()
    }

    pub fn history(&self) -> Vec<Sample> {
        self.inner.lock().unwrap().history.clone()
    }

    pub fn set_mode(&self, mode: HudMode) -> Settings {
        let mut g = self.inner.lock().unwrap();
        g.settings.mode = mode;
        let s = g.settings.clone();
        self.persist_locked(&g);
        s
    }

    pub fn set_window_pos(&self, x: i32, y: i32) {
        let mut g = self.inner.lock().unwrap();
        if g.settings.window_pos == Some((x, y)) {
            return;
        }
        g.settings.window_pos = Some((x, y));
        self.persist_locked(&g);
    }

    pub fn record(&self, snap: UsageSnapshot, sample: Sample) {
        let mut g = self.inner.lock().unwrap();
        g.snapshot = Some(snap);
        g.history.push(sample);
        if g.history.len() > HISTORY_CAP {
            let drop_n = g.history.len() - HISTORY_CAP;
            g.history.drain(..drop_n);
        }
        self.persist_locked(&g);
    }
}
