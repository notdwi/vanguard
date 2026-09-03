use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

use vanguard_lib::capture::engine::Engine;
use vanguard_lib::events::{EventSink, NullSink};
use vanguard_lib::models::{CaptureConfig, ScopeMode, TimelineQuery};
use vanguard_lib::proxy::server;
use vanguard_lib::storage::{queries, sessions, Db};
use vanguard_lib::tls::ca;

/// Minimal origin server: answers every request with a JSON body and a cookie.
async fn spawn_origin() -> SocketAddr {
    let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind origin");
    let addr = listener.local_addr().expect("origin addr");

    tokio::spawn(async move {
        loop {
            let Ok((mut socket, _)) = listener.accept().await else { break };
            tokio::spawn(async move {
                let mut buf = vec![0u8; 8192];
                let n = socket.read(&mut buf).await.unwrap_or(0);
                let request = String::from_utf8_lossy(&buf[..n]).to_string();
                let path = request
                    .lines()
                    .next()
                    .and_then(|l| l.split_whitespace().nth(1))
                    .unwrap_or("/")
                    .to_string();

                let body = format!("{{\"path\":\"{path}\",\"products\":[{{\"id\":4242}}]}}");
                let response = format!(
                    "HTTP/1.1 200 OK\r\n\
                     content-type: application/json\r\n\
                     content-length: {}\r\n\
                     set-cookie: session_id=abc123def456; Path=/; HttpOnly\r\n\
                     connection: close\r\n\r\n{body}",
                    body.len()
                );
                let _ = socket.write_all(response.as_bytes()).await;
                let _ = socket.flush().await;
            });
        }
    });

    addr
}

async fn http_through_proxy(proxy: SocketAddr, origin: SocketAddr, path: &str) -> String {
    let mut stream = TcpStream::connect(proxy).await.expect("connect proxy");
    let request = format!(
        "GET http://{origin}{path} HTTP/1.1\r\nHost: {origin}\r\nAccept: application/json\r\n\
         Cookie: prior=zzz999\r\nConnection: close\r\n\r\n"
    );
    stream.write_all(request.as_bytes()).await.expect("write request");

    let mut response = Vec::new();
    let read = tokio::time::timeout(
        Duration::from_secs(10),
        stream.read_to_end(&mut response),
    )
    .await;
    assert!(read.is_ok(), "reading the proxied response timed out");
    String::from_utf8_lossy(&response).to_string()
}

async fn wait_for<F: Fn() -> bool>(check: F) -> bool {
    // Generous: the writer task is async, and CI machines are slow.
    for _ in 0..150 {
        if check() {
            return true;
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    false
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn captures_a_proxied_exchange_end_to_end() {
    let dir = std::env::temp_dir().join(format!("vanguard-test-{}", uuid::Uuid::new_v4()));
    let db = Db::open(&dir).expect("open db");

    let mut config = CaptureConfig::default();
    config.mode = ScopeMode::AllTraffic;
    let session = sessions::create(&db, "integration", &config).expect("create session");

    let sink: Arc<dyn EventSink> = Arc::new(NullSink);
    let engine = Engine::start(db.clone(), sink, session.id.clone(), config, 0);

    let ca_files = ca::load_or_generate(&dir.join("ca")).expect("generate ca");
    let proxy = server::spawn(
        SocketAddr::from(([127, 0, 0, 1], 0)),
        &ca_files,
        engine.clone(),
    )
    .await
    .expect("spawn proxy");

    let origin = spawn_origin().await;
    let response = http_through_proxy(proxy.addr, origin, "/api/product/4242?q=phone").await;
    assert!(response.contains("200 OK"), "unexpected response: {response}");
    assert!(response.contains("4242"), "body did not reach the client: {response}");

    let query = TimelineQuery {
        session_id: session.id.clone(),
        limit: 100,
        ..Default::default()
    };

    let db_for_wait = db.clone();
    let query_for_wait = query.clone();
    let arrived = wait_for(|| {
        queries::timeline(&db_for_wait, &query_for_wait)
            .map(|page| page.rows.iter().any(|r| r.status == Some(200)))
            .unwrap_or(false)
    })
    .await;
    assert!(arrived, "the exchange never reached the timeline");

    let page = queries::timeline(&db, &query).expect("timeline");
    let row = page.rows.first().expect("one row");
    assert_eq!(row.sequence_id, 1, "sequence numbering starts at 1");
    assert_eq!(row.method, "GET");
    assert_eq!(row.path, "/api/product/4242");
    assert_eq!(row.query.as_deref(), Some("q=phone"));
    assert_eq!(row.family.as_deref(), Some("json"));
    assert_eq!(row.importance.as_str(), "high", "a JSON API hit should rank high");

    let detail = queries::get_request(&db, &row.id).expect("detail");
    assert_eq!(detail.normalized_path, "/api/product/:id");
    assert!(detail.response.is_some());

    let body = queries::load_body(&db, &row.id, "response", 0).expect("response body");
    assert!(body.is_text);
    assert!(body.content.unwrap_or_default().contains("products"));

    let cookies = vanguard_lib::storage::cookies::usage(&db, &session.id).expect("cookies");
    assert!(
        cookies.iter().any(|c| c.name == "session_id" && !c.created_by.is_empty()),
        "the Set-Cookie should be recorded as created"
    );
    assert!(
        cookies.iter().any(|c| c.name == "prior" && !c.used_by.is_empty()),
        "the sent Cookie should be recorded as used"
    );

    let analysis = vanguard_lib::analyzer::run(&db, &session.id).expect("analysis");
    assert_eq!(analysis.overview.requests, 1);
    assert!(analysis.endpoints.iter().any(|e| e.normalized == "/api/product/:id"));

    drop(proxy);
    let _ = std::fs::remove_dir_all(&dir);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn out_of_scope_traffic_is_counted_but_not_stored() {
    let dir = std::env::temp_dir().join(format!("vanguard-scope-{}", uuid::Uuid::new_v4()));
    let db = Db::open(&dir).expect("open db");

    let mut config = CaptureConfig::default();
    config.mode = ScopeMode::ExactHost;
    config.include_domains = vec!["only-this-host.example".into()];
    let session = sessions::create(&db, "scoped", &config).expect("create session");

    let sink: Arc<dyn EventSink> = Arc::new(NullSink);
    let engine = Engine::start(db.clone(), sink, session.id.clone(), config, 0);

    let ca_files = ca::load_or_generate(&dir.join("ca")).expect("generate ca");
    let proxy = server::spawn(
        SocketAddr::from(([127, 0, 0, 1], 0)),
        &ca_files,
        engine.clone(),
    )
    .await
    .expect("spawn proxy");

    let origin = spawn_origin().await;
    let response = http_through_proxy(proxy.addr, origin, "/ignored").await;
    assert!(response.contains("200 OK"), "out-of-scope traffic must still be proxied");

    tokio::time::sleep(Duration::from_millis(400)).await;

    let page = queries::timeline(
        &db,
        &TimelineQuery { session_id: session.id.clone(), limit: 100, ..Default::default() },
    )
    .expect("timeline");
    assert!(page.rows.is_empty(), "out-of-scope traffic must stay out of the timeline");

    let (captured, ignored) = engine.counters();
    assert_eq!(captured, 0);
    assert_eq!(ignored, 1, "the skipped request should still be counted");

    drop(proxy);
    let _ = std::fs::remove_dir_all(&dir);
}
