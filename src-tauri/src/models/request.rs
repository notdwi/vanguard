use super::{BodyRef, Header};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Importance {
    High,
    Medium,
    Low,
}

impl Importance {
    pub fn as_str(&self) -> &'static str {
        match self {
            Importance::High => "high",
            Importance::Medium => "medium",
            Importance::Low => "low",
        }
    }

    pub fn parse(s: &str) -> Self {
        match s {
            "high" => Importance::High,
            "medium" => Importance::Medium,
            _ => Importance::Low,
        }
    }
}

/// Full record of one captured exchange. Never mutated after insert.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapturedRequest {
    pub id: String,
    pub session_id: String,
    pub sequence_id: i64,
    pub timestamp: i64,
    pub method: String,
    pub url: String,
    pub scheme: String,
    pub host: String,
    pub port: u16,
    pub path: String,
    pub query: Option<String>,
    pub normalized_path: String,
    pub protocol: String,
    pub client_addr: Option<String>,
    pub remote_ip: Option<String>,
    pub request_headers: Vec<Header>,
    pub request_body: BodyRef,
    pub request_size: i64,
    pub request_content_type: Option<String>,
    pub response: Option<CapturedResponse>,
    pub error: Option<String>,
    pub importance: Importance,
    pub importance_reasons: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapturedResponse {
    pub status: u16,
    pub status_text: String,
    pub protocol: String,
    pub headers: Vec<Header>,
    pub body: BodyRef,
    pub content_type: Option<String>,
    pub family: String,
    pub timestamp: i64,
    pub duration_ms: i64,
}

/// Compact row rendered by the virtualised timeline.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineRow {
    pub id: String,
    pub sequence_id: i64,
    pub timestamp: i64,
    pub method: String,
    pub scheme: String,
    pub host: String,
    pub path: String,
    pub query: Option<String>,
    pub status: Option<u16>,
    pub duration_ms: Option<i64>,
    pub response_size: i64,
    pub family: Option<String>,
    pub importance: Importance,
    pub has_error: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelinePage {
    pub rows: Vec<TimelineRow>,
    pub total: i64,
    pub offset: i64,
}
