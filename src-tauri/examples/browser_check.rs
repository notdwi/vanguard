//! Manual harness: starts a capture proxy, launches a browser at it, and
//! prints whatever it records. Run with `cargo run --example browser_check --
//! <browser-id> [seconds]`.

use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

use vanguard_lib::capture::engine::Engine;
use vanguard_lib::events::{EventSink, NullSink};
use vanguard_lib::models::{CaptureConfig, TimelineQuery};
use vanguard_lib::proxy::{launcher, server};
use vanguard_lib::storage::{queries, sessions, Db};
use vanguard_lib::tls::ca;

async fn spawn_origin() -> SocketAddr {
    let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind");
    let addr = listener.local_addr().expect("addr");
    tokio::spawn(async move {
        loop {
            let Ok((mut socket, _)) = listener.accept().await else { break };
            tokio::spawn(async move {
                let mut buf = vec![0u8; 8192];
                let _ = socket.read(&mut buf).await;
                let body = "<html><body><h1>Vanguard proxy check</h1>\
                            <script>fetch('/api/ping').then(r=>r.json())</script>\
                            </body></html>";
                let response = format!(
                    "HTTP/1.1 200 OK\r\ncontent-type: text/html\r\ncontent-length: {}\r\n\
                     connection: close\r\n\r\n{body}",
                    body.len()
                );
                let _ = socket.write_all(response.as_bytes()).await;
            });
        }
    });
    addr
}

#[tokio::main]
async fn main() {
    let mut args = std::env::args().skip(1);
    let browser_id = args.next().unwrap_or_else(|| "firefox".into());
    let seconds: u64 = args.next().and_then(|s| s.parse().ok()).unwrap_or(25);

    let dir = std::env::temp_dir().join("vanguard-browser-check");
    let _ = std::fs::remove_dir_all(&dir);
    let db = Db::open(&dir).expect("db");
    let session = sessions::create(&db, "browser check", &CaptureConfig::default()).expect("session");

    let sink: Arc<dyn EventSink> = Arc::new(NullSink);
    let engine = Engine::start(db.clone(), sink, session.id.clone(), CaptureConfig::default(), 0);

    let ca_files = ca::load_or_generate(&dir.join("ca")).expect("ca");
    let proxy = server::spawn(SocketAddr::from(([127, 0, 0, 1], 0)), &ca_files, engine.clone())
        .await
        .expect("proxy");

    let origin = spawn_origin().await;

    println!("detected browsers:");
    for b in launcher::available() {
        println!("  {} ({:?}) uses_system_trust={} -> {}", b.id, b.kind, b.uses_system_trust, b.path);
    }

    println!("\nproxy listening on {}", proxy.addr);
    println!("local origin on http://{origin}/");
    println!("ca cert at {}", ca::cert_path(&dir.join("ca")).display());

    if browser_id == "serve" {
        println!("\nserve mode: point a client at the proxy yourself");
        watch(&db, &session.id, &engine, seconds).await;
        return;
    }

    println!("launching `{browser_id}` ...");

    match launcher::launch(
        &browser_id,
        &proxy.addr.to_string(),
        &dir,
        &ca::cert_path(&dir.join("ca")),
        &format!("http://{origin}/"),
    ) {
        Ok(()) => println!("launch command issued"),
        Err(e) => {
            eprintln!("launch failed: {e}");
            return;
        }
    }

    watch(&db, &session.id, &engine, seconds).await;
    drop(proxy);
}

async fn watch(db: &Db, session_id: &str, engine: &Arc<Engine>, seconds: u64) {
    println!("watching for {seconds}s ...\n");
    let deadline = std::time::Instant::now() + Duration::from_secs(seconds);
    let mut seen = 0usize;

    while std::time::Instant::now() < deadline {
        tokio::time::sleep(Duration::from_millis(700)).await;
        let page = queries::timeline(
            db,
            &TimelineQuery { session_id: session_id.to_string(), limit: 500, ..Default::default() },
        )
        .expect("timeline");
        while seen < page.rows.len() {
            let r = &page.rows[seen];
            println!(
                "#{:03} {:<6} {:>3} {}://{}{}",
                r.sequence_id,
                r.method,
                r.status.map(|s| s.to_string()).unwrap_or_else(|| "···".into()),
                r.scheme,
                r.host,
                r.path
            );
            seen += 1;
        }
    }

    let (captured, ignored) = engine.counters();
    println!("\ncaptured={captured} ignored={ignored}");
    if captured == 0 {
        println!("nothing was captured: the client did not use the proxy");
    }
}
