use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use vanguard_lib::capture::engine::Engine;
use vanguard_lib::events::{EventSink, NullSink};
use vanguard_lib::models::{CaptureConfig, TimelineQuery};
use vanguard_lib::proxy::server;
use vanguard_lib::storage::{queries, sessions, Db};
use vanguard_lib::tls::ca;

/// Exercises the full CONNECT + TLS interception path against a real host.
/// Ignored by default because it needs outbound network access; run with
/// `cargo test --test https_mitm -- --ignored`.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[ignore]
async fn intercepts_real_https_traffic() {
    let dir = std::env::temp_dir().join(format!("vanguard-https-{}", uuid::Uuid::new_v4()));
    let db = Db::open(&dir).expect("open db");

    let config = CaptureConfig::default();
    let session = sessions::create(&db, "https", &config).expect("create session");

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

    // Trust *only* the Vanguard CA. Leaving the platform verifier on would
    // make the client fall back to the OS trust store, which does not contain
    // this authority, so the assertion below really does prove interception.
    let root = reqwest::Certificate::from_pem(ca_files.cert_pem.as_bytes()).expect("ca pem");
    let client = reqwest::Client::builder()
        .proxy(reqwest::Proxy::all(format!("http://{}", proxy.addr)).expect("proxy"))
        .tls_backend_rustls()
        .tls_certs_only([root])
        .timeout(Duration::from_secs(30))
        .build()
        .expect("client");

    let response = client
        .get("https://example.com/")
        .send()
        .await
        .expect("request through the intercepting proxy");
    assert!(response.status().is_success(), "status {}", response.status());
    let body = response.text().await.expect("body");
    assert!(body.contains("Example"), "unexpected body");

    let mut captured = None;
    for _ in 0..60 {
        let page = queries::timeline(
            &db,
            &TimelineQuery { session_id: session.id.clone(), limit: 50, ..Default::default() },
        )
        .expect("timeline");
        if let Some(row) = page.rows.iter().find(|r| r.status.is_some()) {
            captured = Some(row.clone());
            break;
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    let row = captured.expect("the HTTPS exchange should have been captured");
    assert_eq!(row.scheme, "https");
    assert_eq!(row.host, "example.com");
    assert_eq!(row.status, Some(200));

    let stored = queries::load_body(&db, &row.id, "response", 0).expect("body");
    assert!(stored.is_text, "the decoded HTML body should be stored as text");
    assert!(stored.content.unwrap_or_default().contains("Example"));

    drop(proxy);
    let _ = std::fs::remove_dir_all(&dir);
}
