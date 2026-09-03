use super::CaptureConfig;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum CaptureState {
    Idle,
    Capturing,
    Paused,
    Stopped,
}

impl CaptureState {
    pub fn as_str(&self) -> &'static str {
        match self {
            CaptureState::Idle => "idle",
            CaptureState::Capturing => "capturing",
            CaptureState::Paused => "paused",
            CaptureState::Stopped => "stopped",
        }
    }

    pub fn parse(s: &str) -> Self {
        match s {
            "capturing" => CaptureState::Capturing,
            "paused" => CaptureState::Paused,
            "stopped" => CaptureState::Stopped,
            _ => CaptureState::Idle,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: String,
    pub name: String,
    pub created_at: i64,
    pub updated_at: i64,
    pub status: CaptureState,
    pub config: CaptureConfig,
    pub request_count: i64,
    pub ignored_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionSummary {
    pub id: String,
    pub name: String,
    pub created_at: i64,
    pub updated_at: i64,
    pub status: CaptureState,
    pub request_count: i64,
    pub ignored_count: i64,
    pub domains: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaptureStatus {
    pub state: CaptureState,
    pub session_id: Option<String>,
    pub session_name: Option<String>,
    pub proxy_addr: Option<String>,
    pub captured: i64,
    pub ignored: i64,
}
