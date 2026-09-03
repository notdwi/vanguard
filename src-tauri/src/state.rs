use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;

use parking_lot::Mutex;
use tauri::{AppHandle, Manager};

use crate::analyzer::AnalysisBundle;
use crate::capture::engine::Engine;
use crate::error::{AppError, Result};
use crate::models::{CaptureState, CaptureStatus};
use crate::proxy::server::ProxyHandle;
use crate::storage::{sessions, Db};

pub const DEFAULT_PROXY_PORT: u16 = 8080;

pub struct Active {
    pub engine: Arc<Engine>,
    pub proxy: ProxyHandle,
}

pub struct AppState {
    pub db: Db,
    pub root: PathBuf,
    active: Mutex<Option<Active>>,
    analysis: Mutex<Option<AnalysisBundle>>,
}

impl AppState {
    pub fn new(app: &AppHandle) -> Result<Self> {
        let root = app
            .path()
            .app_data_dir()
            .map_err(|e| AppError::Other(format!("could not resolve the data directory: {e}")))?;
        let db = Db::open(&root)?;
        Ok(Self { db, root, active: Mutex::new(None), analysis: Mutex::new(None) })
    }

    pub fn ca_root(&self) -> PathBuf {
        self.root.join("ca")
    }

    pub fn set_active(&self, active: Active) {
        *self.active.lock() = Some(active);
    }

    pub fn engine(&self) -> Option<Arc<Engine>> {
        self.active.lock().as_ref().map(|a| Arc::clone(&a.engine))
    }

    pub fn proxy_addr(&self) -> Option<SocketAddr> {
        self.active.lock().as_ref().map(|a| a.proxy.addr)
    }

    pub fn require_engine(&self) -> Result<Arc<Engine>> {
        self.engine().ok_or(AppError::NoActiveSession)
    }

    /// Stops the proxy, flushes counters and marks the session stopped.
    pub fn stop_active(&self) -> Option<String> {
        let mut guard = self.active.lock();
        let Some(mut active) = guard.take() else { return None };
        active.proxy.stop();
        active.engine.persist_counters();
        let session_id = active.engine.session_id().to_string();
        let _ = sessions::set_status(&self.db, &session_id, CaptureState::Stopped);
        Some(session_id)
    }

    pub fn status(&self) -> CaptureStatus {
        let guard = self.active.lock();
        match guard.as_ref() {
            Some(active) => {
                let (captured, ignored) = active.engine.counters();
                let session_id = active.engine.session_id().to_string();
                let name = sessions::get(&self.db, &session_id).ok().map(|s| s.name);
                let stored = sessions::get(&self.db, &session_id)
                    .map(|s| (s.request_count, s.ignored_count))
                    .unwrap_or((0, 0));
                CaptureStatus {
                    state: active.engine.state(),
                    session_id: Some(session_id),
                    session_name: name,
                    proxy_addr: Some(active.proxy.addr.to_string()),
                    captured: stored.0 + captured,
                    ignored: stored.1 + ignored,
                }
            }
            None => CaptureStatus {
                state: CaptureState::Idle,
                session_id: None,
                session_name: None,
                proxy_addr: None,
                captured: 0,
                ignored: 0,
            },
        }
    }

    pub fn cache_analysis(&self, bundle: AnalysisBundle) {
        *self.analysis.lock() = Some(bundle);
    }

    pub fn cached_analysis(&self, session_id: &str) -> Option<AnalysisBundle> {
        self.analysis
            .lock()
            .as_ref()
            .filter(|b| b.session_id == session_id)
            .cloned()
    }

    pub fn invalidate_analysis(&self) {
        *self.analysis.lock() = None;
    }
}
