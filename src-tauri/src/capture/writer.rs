use std::sync::Arc;

use tokio::sync::mpsc::UnboundedReceiver;

use crate::events::{self, EventSink};
use crate::storage::{requests as store, Db};

use super::engine::Job;

/// Single consumer of the capture queue. Persisting happens here so the proxy
/// path never waits on the database, and events fire only once a row exists.
pub async fn run(
    db: Db,
    sink: Arc<dyn EventSink>,
    session_id: String,
    mut rx: UnboundedReceiver<Job>,
) {
    while let Some(job) = rx.recv().await {
        let db = db.clone();
        let sink = Arc::clone(&sink);
        let session_id = session_id.clone();

        let handled =
            tokio::task::spawn_blocking(move || apply(&db, &sink, &session_id, job)).await;
        if let Err(e) = handled {
            tracing::warn!("capture writer task failed: {e}");
        }
    }
}

fn apply(db: &Db, sink: &Arc<dyn EventSink>, session_id: &str, job: Job) {
    match job {
        Job::Request(req, event) => {
            if let Err(e) = store::insert_request(db, *req) {
                tracing::warn!("failed to persist request: {e}");
                return;
            }
            events::send(sink, events::REQUEST_STARTED, event);
        }
        Job::Response(res, event) => {
            let request_id = res.request_id.clone();
            let importance = res.final_importance;
            let reasons = res.final_reasons.clone();
            if let Err(e) = store::insert_response(db, *res) {
                tracing::warn!("failed to persist response: {e}");
                return;
            }
            let _ = store::set_importance(db, &request_id, importance, &reasons);
            events::send(sink, events::RESPONSE_RECEIVED, event);
        }
        Job::Cookies(batch) => {
            if let Err(e) = store::insert_cookie_events(db, &batch) {
                tracing::warn!("failed to persist cookie events: {e}");
            }
        }
        Job::Failed { request_id, sequence_id, message } => {
            let _ = store::mark_failed(db, &request_id, &message);
            events::send(
                sink,
                events::REQUEST_FAILED,
                events::RequestFailed {
                    session_id: session_id.to_string(),
                    request_id,
                    sequence_id,
                    message,
                },
            );
        }
    }
}
