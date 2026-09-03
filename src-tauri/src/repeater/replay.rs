use std::sync::Arc;
use std::time::Duration;

use futures::stream::{FuturesUnordered, StreamExt};

use crate::error::Result;
use crate::events::{self, EventSink};
use crate::models::{RepeaterDraft, ReplayMode, ReplayOptions, ReplayProgress, ReplayResult};
use crate::storage::{drafts, Db};

use super::execute;

const MAX_ITERATIONS: u32 = 500;
const MAX_CONCURRENCY: usize = 20;

/// Runs a draft one or more times. Concurrent mode is opt-in and capped, so a
/// stray value in the iterations box cannot turn into a flood.
pub async fn run(
    db: Db,
    sink: Arc<dyn EventSink>,
    draft: RepeaterDraft,
    options: ReplayOptions,
) -> Result<Vec<ReplayResult>> {
    let iterations = options.iterations.clamp(1, MAX_ITERATIONS);
    let client = execute::client(&options)?;
    let draft = Arc::new(draft);

    events::send(
        &sink,
        events::REPLAY_STARTED,
        ReplayProgress { draft_id: draft.id.clone(), completed: 0, total: iterations },
    );

    let results = match options.mode {
        ReplayMode::Sequential => {
            sequential(&db, &sink, &client, &draft, iterations, options.delay_ms).await
        }
        ReplayMode::Concurrent => concurrent(&db, &sink, &client, &draft, iterations).await,
    };

    events::send(
        &sink,
        events::REPLAY_COMPLETED,
        ReplayProgress {
            draft_id: draft.id.clone(),
            completed: results.len() as u32,
            total: iterations,
        },
    );
    Ok(results)
}

async fn sequential(
    db: &Db,
    sink: &Arc<dyn EventSink>,
    client: &reqwest::Client,
    draft: &Arc<RepeaterDraft>,
    iterations: u32,
    delay_ms: u64,
) -> Vec<ReplayResult> {
    let mut results = Vec::with_capacity(iterations as usize);
    for index in 1..=iterations {
        if index > 1 && delay_ms > 0 {
            tokio::time::sleep(Duration::from_millis(delay_ms.min(60_000))).await;
        }
        let result = execute::send(client, draft, index).await;
        publish(db, sink, &result, index, iterations);
        results.push(result);
    }
    results
}

async fn concurrent(
    db: &Db,
    sink: &Arc<dyn EventSink>,
    client: &reqwest::Client,
    draft: &Arc<RepeaterDraft>,
    iterations: u32,
) -> Vec<ReplayResult> {
    let mut results = Vec::with_capacity(iterations as usize);
    let mut pending = FuturesUnordered::new();
    let mut next = 1u32;
    let mut completed = 0u32;

    while next <= iterations || !pending.is_empty() {
        while next <= iterations && pending.len() < MAX_CONCURRENCY {
            let client = client.clone();
            let draft = Arc::clone(draft);
            let index = next;
            pending.push(async move { execute::send(&client, &draft, index).await });
            next += 1;
        }
        if let Some(result) = pending.next().await {
            completed += 1;
            publish(db, sink, &result, completed, iterations);
            results.push(result);
        }
    }

    results.sort_by_key(|r| r.index);
    results
}

fn publish(
    db: &Db,
    sink: &Arc<dyn EventSink>,
    result: &ReplayResult,
    completed: u32,
    total: u32,
) {
    if let Err(e) = drafts::insert_replay(db, result) {
        tracing::warn!("could not store replay result: {e}");
    }
    events::send(sink, events::REPLAY_RESULT, result.clone());
    events::send(
        sink,
        events::REPLAY_PROGRESS,
        ReplayProgress { draft_id: result.draft_id.clone(), completed, total },
    );
}
