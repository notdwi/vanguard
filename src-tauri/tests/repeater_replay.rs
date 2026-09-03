use std::net::SocketAddr;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

use vanguard_lib::events::{EventSink, NullSink};
use vanguard_lib::models::{
    CaptureConfig, Header, QueryParam, RepeaterDraft, ReplayMode, ReplayOptions,
};
use vanguard_lib::repeater::{compare, draft as draft_builder, replay};
use vanguard_lib::storage::{drafts, sessions, Db};

/// Answers 200 for the first two calls and 403 afterwards, so a replay run
/// shows a state change the way a rate-limited endpoint would.
async fn spawn_flaky_origin() -> (SocketAddr, Arc<AtomicUsize>) {
    let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind");
    let addr = listener.local_addr().expect("addr");
    let hits = Arc::new(AtomicUsize::new(0));
    let counter = Arc::clone(&hits);

    tokio::spawn(async move {
        loop {
            let Ok((mut socket, _)) = listener.accept().await else { break };
            let counter = Arc::clone(&counter);
            tokio::spawn(async move {
                let mut buf = vec![0u8; 4096];
                let _ = socket.read(&mut buf).await;
                let n = counter.fetch_add(1, Ordering::SeqCst);

                let (status, body) = if n < 2 {
                    (200, format!("{{\"products\":[1,2],\"requestId\":\"r{n}\"}}"))
                } else {
                    (403, format!("{{\"error\":\"denied\",\"requestId\":\"r{n}\"}}"))
                };
                let response = format!(
                    "HTTP/1.1 {status} OK\r\ncontent-type: application/json\r\n\
                     content-length: {}\r\nconnection: close\r\n\r\n{body}",
                    body.len()
                );
                let _ = socket.write_all(response.as_bytes()).await;
                let _ = socket.flush().await;
            });
        }
    });

    (addr, hits)
}

fn draft_for(session_id: &str, url: String) -> RepeaterDraft {
    let mut draft = draft_builder::blank(session_id);
    draft.method = "GET".into();
    draft.url = url;
    draft.query = vec![QueryParam { name: "page".into(), value: "1".into() }];
    draft.headers = vec![Header { name: "accept".into(), value: "application/json".into() }];
    draft
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn sequential_replay_records_every_iteration() {
    let dir = std::env::temp_dir().join(format!("vanguard-replay-{}", uuid::Uuid::new_v4()));
    let db = Db::open(&dir).expect("open db");
    let session = sessions::create(&db, "replay", &CaptureConfig::default()).expect("session");

    let (origin, _hits) = spawn_flaky_origin().await;
    let draft = draft_for(&session.id, format!("http://{origin}/api/search"));
    drafts::upsert(&db, &draft).expect("save draft");

    let sink: Arc<dyn EventSink> = Arc::new(NullSink);
    let results = replay::run(
        db.clone(),
        sink,
        draft.clone(),
        ReplayOptions {
            iterations: 4,
            mode: ReplayMode::Sequential,
            delay_ms: 0,
            follow_redirects: false,
            timeout_ms: 10_000,
        },
    )
    .await
    .expect("replay");

    assert_eq!(results.len(), 4);
    assert_eq!(results[0].index, 1);
    assert_eq!(results[0].status, Some(200));
    assert_eq!(results[3].status, Some(403), "later calls should show the state change");
    assert!(results[0].sent.url.contains("page=1"), "query edits must be sent");

    let stored = drafts::list_replays(&db, &draft.id, 50).expect("history");
    assert_eq!(stored.len(), 4, "every replay is kept in history");

    let left = results[0].clone();
    let right = results[3].clone();
    let comparison = compare::compare(
        vanguard_lib::models::ComparisonSide {
            label: "a".into(),
            status: left.status,
            duration_ms: left.duration_ms,
            size: left.body_size,
            content_type: left.content_type.clone(),
            headers: left.headers.clone(),
            body: left.body.clone(),
        },
        vanguard_lib::models::ComparisonSide {
            label: "b".into(),
            status: right.status,
            duration_ms: right.duration_ms,
            size: right.body_size,
            content_type: right.content_type.clone(),
            headers: right.headers.clone(),
            body: right.body.clone(),
        },
    );

    assert!(comparison.body_comparable);
    assert!(
        comparison.body_diff.iter().any(|d| d.path == "$.products"),
        "the removed array should show up in the diff"
    );
    assert!(
        comparison.body_diff.iter().any(|d| d.path == "$.requestId" && d.volatile),
        "requestId should be flagged volatile"
    );

    let _ = std::fs::remove_dir_all(&dir);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn concurrent_replay_runs_every_iteration() {
    let dir = std::env::temp_dir().join(format!("vanguard-conc-{}", uuid::Uuid::new_v4()));
    let db = Db::open(&dir).expect("open db");
    let session = sessions::create(&db, "conc", &CaptureConfig::default()).expect("session");

    let (origin, hits) = spawn_flaky_origin().await;
    let draft = draft_for(&session.id, format!("http://{origin}/api/search"));
    drafts::upsert(&db, &draft).expect("save draft");

    let sink: Arc<dyn EventSink> = Arc::new(NullSink);
    let results = replay::run(
        db.clone(),
        sink,
        draft,
        ReplayOptions {
            iterations: 8,
            mode: ReplayMode::Concurrent,
            delay_ms: 0,
            follow_redirects: false,
            timeout_ms: 10_000,
        },
    )
    .await
    .expect("replay");

    assert_eq!(results.len(), 8);
    assert_eq!(hits.load(Ordering::SeqCst), 8, "every iteration must reach the origin");
    let indexes: Vec<u32> = results.iter().map(|r| r.index).collect();
    assert_eq!(indexes, (1..=8).collect::<Vec<_>>(), "results come back ordered");

    let _ = std::fs::remove_dir_all(&dir);
}
