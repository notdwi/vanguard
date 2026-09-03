use std::sync::atomic::{AtomicBool, AtomicI64, Ordering};
use std::sync::Arc;

use tokio::sync::mpsc::{unbounded_channel, UnboundedSender};

use crate::events::EventSink;
use crate::models::{CaptureConfig, CaptureState};
use crate::proxy::scope::Scope;
use crate::storage::requests::{NewCookieEvent, NewRequest, NewResponse};
use crate::storage::{sessions, Db};

use super::writer;

pub enum Job {
    Request(Box<NewRequest>, crate::events::RequestStarted),
    Response(Box<NewResponse>, crate::events::ResponseReceived),
    Cookies(Vec<NewCookieEvent>),
    Failed { request_id: String, sequence_id: i64, message: String },
}

/// Owns the live capture: scope, sequence numbering and the write queue.
/// The proxy handler only ever pushes into the queue.
pub struct Engine {
    db: Db,
    session_id: String,
    scope: Scope,
    sequence: AtomicI64,
    captured: AtomicI64,
    ignored: AtomicI64,
    paused: AtomicBool,
    tx: UnboundedSender<Job>,
}

impl Engine {
    pub fn start(
        db: Db,
        sink: Arc<dyn EventSink>,
        session_id: String,
        config: CaptureConfig,
        start_sequence: i64,
    ) -> Arc<Self> {
        let (tx, rx) = unbounded_channel();
        let engine = Arc::new(Self {
            db: db.clone(),
            session_id: session_id.clone(),
            scope: Scope::new(config),
            sequence: AtomicI64::new(start_sequence),
            captured: AtomicI64::new(0),
            ignored: AtomicI64::new(0),
            paused: AtomicBool::new(false),
            tx,
        });
        tokio::spawn(writer::run(db, sink, session_id, rx));
        engine
    }

    pub fn session_id(&self) -> &str {
        &self.session_id
    }

    pub fn scope(&self) -> &Scope {
        &self.scope
    }

    pub fn config(&self) -> &CaptureConfig {
        self.scope.config()
    }

    pub fn db(&self) -> &Db {
        &self.db
    }

    pub fn set_paused(&self, paused: bool) {
        self.paused.store(paused, Ordering::Relaxed);
    }

    pub fn is_paused(&self) -> bool {
        self.paused.load(Ordering::Relaxed)
    }

    pub fn state(&self) -> CaptureState {
        if self.is_paused() {
            CaptureState::Paused
        } else {
            CaptureState::Capturing
        }
    }

    pub fn next_sequence(&self) -> i64 {
        self.sequence.fetch_add(1, Ordering::SeqCst) + 1
    }

    pub fn counters(&self) -> (i64, i64) {
        (self.captured.load(Ordering::Relaxed), self.ignored.load(Ordering::Relaxed))
    }

    pub fn note_ignored(&self) {
        self.ignored.fetch_add(1, Ordering::Relaxed);
    }

    pub fn note_captured(&self) {
        self.captured.fetch_add(1, Ordering::Relaxed);
    }

    pub fn submit(&self, job: Job) {
        let _ = self.tx.send(job);
    }

    /// Moves the in-memory counters onto the session row and resets them.
    pub fn persist_counters(&self) {
        let (captured, ignored) = self.counters();
        if captured == 0 && ignored == 0 {
            return;
        }
        let _ = sessions::bump_counters(&self.db, &self.session_id, captured, ignored);
        self.captured.fetch_sub(captured, Ordering::Relaxed);
        self.ignored.fetch_sub(ignored, Ordering::Relaxed);
    }
}
