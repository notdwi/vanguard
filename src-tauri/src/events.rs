use std::sync::Arc;

use serde::Serialize;
use tauri::{AppHandle, Emitter};

use crate::models::{CaptureStatus, ReplayProgress, ReplayResult};

pub const CAPTURE_STATUS: &str = "capture:status";
pub const REQUEST_STARTED: &str = "request:started";
pub const RESPONSE_RECEIVED: &str = "response:received";
pub const REQUEST_FAILED: &str = "request:failed";
pub const COUNTERS: &str = "capture:counters";
pub const REPLAY_STARTED: &str = "replay:started";
pub const REPLAY_RESULT: &str = "replay:result";
pub const REPLAY_PROGRESS: &str = "replay:progress";
pub const REPLAY_COMPLETED: &str = "replay:completed";
pub const ANALYSIS_UPDATED: &str = "analysis:updated";
pub const SESSIONS_CHANGED: &str = "sessions:changed";

/// Decouples the capture pipeline from Tauri so the proxy can be exercised
/// headlessly in tests.
pub trait EventSink: Send + Sync + 'static {
    fn send(&self, event: &str, payload: serde_json::Value);
}

impl EventSink for AppHandle {
    fn send(&self, event: &str, payload: serde_json::Value) {
        if let Err(e) = self.emit(event, payload) {
            tracing::warn!("failed to emit {event}: {e}");
        }
    }
}

pub struct NullSink;

impl EventSink for NullSink {
    fn send(&self, _event: &str, _payload: serde_json::Value) {}
}

pub fn send<T: Serialize>(sink: &Arc<dyn EventSink>, event: &str, payload: T) {
    match serde_json::to_value(payload) {
        Ok(value) => sink.send(event, value),
        Err(e) => tracing::warn!("failed to serialise {event}: {e}"),
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct RequestStarted {
    pub session_id: String,
    pub request_id: String,
    pub sequence_id: i64,
    pub timestamp: i64,
    pub method: String,
    pub scheme: String,
    pub host: String,
    pub path: String,
    pub query: Option<String>,
    pub protocol: String,
    pub importance: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ResponseReceived {
    pub session_id: String,
    pub request_id: String,
    pub sequence_id: i64,
    pub status: u16,
    pub content_type: Option<String>,
    pub family: String,
    pub body_size: i64,
    pub duration_ms: i64,
    pub importance: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct RequestFailed {
    pub session_id: String,
    pub request_id: String,
    pub sequence_id: i64,
    pub message: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct Counters {
    pub session_id: String,
    pub captured: i64,
    pub ignored: i64,
}

pub fn emit<T: Serialize + Clone>(app: &AppHandle, event: &str, payload: T) {
    if let Err(e) = app.emit(event, payload) {
        tracing::warn!("failed to emit {event}: {e}");
    }
}

pub fn capture_status(app: &AppHandle, status: &CaptureStatus) {
    emit(app, CAPTURE_STATUS, status.clone());
}

pub fn replay_result(app: &AppHandle, result: &ReplayResult) {
    emit(app, REPLAY_RESULT, result.clone());
}

pub fn replay_progress(app: &AppHandle, progress: &ReplayProgress) {
    emit(app, REPLAY_PROGRESS, progress.clone());
}

pub fn sessions_changed(app: &AppHandle) {
    emit(app, SESSIONS_CHANGED, ());
}
