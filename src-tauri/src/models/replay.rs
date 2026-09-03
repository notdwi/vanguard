use super::{CookiePair, Header, QueryParam};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ReplayMode {
    Sequential,
    Concurrent,
}

/// Editable copy of a captured request. The capture itself is never touched.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepeaterDraft {
    pub id: String,
    pub session_id: String,
    pub source_request_id: Option<String>,
    pub source_sequence_id: Option<i64>,
    pub label: String,
    pub method: String,
    pub url: String,
    pub query: Vec<QueryParam>,
    pub headers: Vec<Header>,
    pub cookies: Vec<CookiePair>,
    pub body: String,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplayOptions {
    pub iterations: u32,
    pub mode: ReplayMode,
    pub delay_ms: u64,
    pub follow_redirects: bool,
    pub timeout_ms: u64,
}

impl Default for ReplayOptions {
    fn default() -> Self {
        Self {
            iterations: 1,
            mode: ReplayMode::Sequential,
            delay_ms: 0,
            follow_redirects: false,
            timeout_ms: 30_000,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplayResult {
    pub id: String,
    pub draft_id: String,
    pub session_id: String,
    pub index: u32,
    pub started_at: i64,
    pub duration_ms: i64,
    pub status: Option<u16>,
    pub status_text: String,
    pub protocol: String,
    pub headers: Vec<Header>,
    pub body: Option<String>,
    pub body_size: i64,
    pub body_is_text: bool,
    pub content_type: Option<String>,
    pub error: Option<String>,
    /// Snapshot of exactly what was sent, for audit.
    pub sent: RepeaterSnapshot,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepeaterSnapshot {
    pub method: String,
    pub url: String,
    pub headers: Vec<Header>,
    pub body: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplayProgress {
    pub draft_id: String,
    pub completed: u32,
    pub total: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComparisonSide {
    pub label: String,
    pub status: Option<u16>,
    pub duration_ms: i64,
    pub size: i64,
    pub content_type: Option<String>,
    pub headers: Vec<Header>,
    pub body: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum DiffKind {
    Added,
    Removed,
    Changed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffEntry {
    pub kind: DiffKind,
    pub path: String,
    pub left: Option<String>,
    pub right: Option<String>,
    /// Flagged when the key name suggests a value that changes on every call.
    pub volatile: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Comparison {
    pub left: ComparisonSide,
    pub right: ComparisonSide,
    pub header_diff: Vec<DiffEntry>,
    pub body_diff: Vec<DiffEntry>,
    pub body_comparable: bool,
}
