use std::net::SocketAddr;
use std::sync::Arc;

use hudsucker::certificate_authority::RcgenAuthority;
use hudsucker::rustls::crypto::aws_lc_rs;
use hudsucker::{Proxy, WebSocketHandler};
use tokio::sync::oneshot;

use crate::capture::engine::Engine;
use crate::error::{AppError, Result};
use crate::tls::ca::CaFiles;

use super::handler::CaptureHandler;

#[derive(Clone)]
struct PassthroughWebSockets;

impl WebSocketHandler for PassthroughWebSockets {}

pub struct ProxyHandle {
    pub addr: SocketAddr,
    shutdown: Option<oneshot::Sender<()>>,
}

impl ProxyHandle {
    pub fn stop(&mut self) {
        if let Some(tx) = self.shutdown.take() {
            let _ = tx.send(());
        }
    }
}

impl Drop for ProxyHandle {
    fn drop(&mut self) {
        self.stop();
    }
}

pub async fn spawn(addr: SocketAddr, ca: &CaFiles, engine: Arc<Engine>) -> Result<ProxyHandle> {
    let listener = std::net::TcpListener::bind(addr)
        .map_err(|e| AppError::Proxy(format!("could not bind {addr}: {e}")))?;
    let bound = listener
        .local_addr()
        .map_err(|e| AppError::Proxy(format!("could not read the bound address: {e}")))?;
    drop(listener);

    let issuer = crate::tls::ca::issuer(ca)?;
    let authority = RcgenAuthority::new(issuer, 1_000, aws_lc_rs::default_provider());

    let (tx, rx) = oneshot::channel::<()>();

    let proxy = Proxy::builder()
        .with_addr(bound)
        .with_ca(authority)
        .with_rustls_connector(aws_lc_rs::default_provider())
        .with_http_handler(CaptureHandler::new(engine))
        .with_websocket_handler(PassthroughWebSockets)
        .with_graceful_shutdown(async move {
            let _ = rx.await;
        })
        .build()
        .map_err(|e| AppError::Proxy(format!("could not build the proxy: {e}")))?;

    tokio::spawn(async move {
        if let Err(e) = proxy.start().await {
            tracing::error!("proxy stopped: {e}");
        }
    });

    Ok(ProxyHandle { addr: bound, shutdown: Some(tx) })
}
