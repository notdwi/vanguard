use std::sync::Arc;

use hudsucker::{Body, HttpContext, HttpHandler, RequestOrResponse};
use hudsucker::hyper::{Method, Request, Response};

use crate::capture::engine::{Engine, Job};
use crate::events;
use crate::http::{cookies, headers, url as urlutil};
use crate::models::{new_id, now_millis, ContentFamily, Header};
use crate::storage::requests::{NewRequest, NewResponse};

use super::collect;

#[derive(Clone)]
struct Pending {
    request_id: String,
    sequence_id: i64,
    started_at: i64,
    host: String,
    path: String,
    query: Option<String>,
    method: String,
    has_auth: bool,
    has_cookies: bool,
    request_content_type: Option<String>,
    has_request_body: bool,
}

#[derive(Clone)]
pub struct CaptureHandler {
    engine: Arc<Engine>,
    pending: Option<Pending>,
}

impl CaptureHandler {
    pub fn new(engine: Arc<Engine>) -> Self {
        Self { engine, pending: None }
    }
}

impl HttpHandler for CaptureHandler {
    async fn handle_request(
        &mut self,
        ctx: &HttpContext,
        req: Request<Body>,
    ) -> RequestOrResponse {
        if req.method() == Method::CONNECT {
            return req.into();
        }
        if self.engine.is_paused() {
            return req.into();
        }

        let (parts, body) = req.into_parts();
        let hs = headers::from_map(&parts.headers);
        let host_header = headers::get(&hs, "host").map(|s| s.to_string());
        let tls = parts.uri.scheme_str() == Some("https") || parts.uri.port_u16() == Some(443);

        let Some(absolute) = urlutil::absolute(&parts.uri, host_header.as_deref(), tls) else {
            return Request::from_parts(parts, body).into();
        };
        let Some(parsed) = urlutil::split(&absolute) else {
            return Request::from_parts(parts, body).into();
        };

        let method = parts.method.as_str().to_string();
        if !self.engine.scope().allows(&parsed.host, &parsed.path, &method) {
            self.engine.note_ignored();
            return Request::from_parts(parts, body).into();
        }

        let cfg = self.engine.config();
        let content_type = headers::content_type(&hs);
        let capture_body = cfg.capture_request_bodies && parts.method != Method::GET;
        let (body_bytes, body) = if capture_body {
            collect::buffer(body, cfg.max_body_bytes).await
        } else {
            (Vec::new(), body)
        };

        let sequence_id = self.engine.next_sequence();
        let request_id = new_id("req");
        let started_at = now_millis();

        let has_auth = headers::get(&hs, "authorization").is_some()
            || headers::get(&hs, "x-api-key").is_some();
        let request_cookies = cookies::parse_request_cookies(&hs);
        let has_cookies = !request_cookies.is_empty();
        let normalized_path = crate::analyzer::endpoints::normalize_path(&parsed.path);
        let is_api = crate::analyzer::endpoints::is_api_like(&parsed.path, content_type.as_deref());
        let protocol = headers::protocol_label(parts.version);

        let verdict = crate::analyzer::importance::score(&crate::analyzer::importance::Signals {
            method: &method,
            host: &parsed.host,
            path: &parsed.path,
            query: parsed.query.as_deref(),
            request_content_type: content_type.as_deref(),
            response_content_type: None,
            status: None,
            has_auth,
            has_cookies,
            has_request_body: !body_bytes.is_empty(),
            response_size: 0,
            has_path_id: normalized_path.contains(":id")
                || normalized_path.contains(":uuid")
                || normalized_path.contains(":hash"),
            relationship_count: 0,
            repeat_count: 1,
        });

        let event = events::RequestStarted {
            session_id: self.engine.session_id().to_string(),
            request_id: request_id.clone(),
            sequence_id,
            timestamp: started_at,
            method: method.clone(),
            scheme: parsed.scheme.clone(),
            host: parsed.host.clone(),
            path: parsed.path.clone(),
            query: parsed.query.clone(),
            protocol: protocol.clone(),
            importance: verdict.importance.as_str().to_string(),
        };

        let record = NewRequest {
            id: request_id.clone(),
            session_id: self.engine.session_id().to_string(),
            sequence_id,
            timestamp: started_at,
            method: method.clone(),
            url: parsed.url.clone(),
            scheme: parsed.scheme.clone(),
            host: parsed.host.clone(),
            port: parsed.port,
            path: parsed.path.clone(),
            query: parsed.query.clone(),
            normalized_path,
            protocol,
            client_addr: Some(ctx.client_addr.to_string()),
            remote_ip: None,
            headers: hs.clone(),
            body: body_bytes.clone(),
            content_type: content_type.clone(),
            has_cookies,
            has_auth,
            is_api,
            importance: verdict.importance,
            importance_reasons: verdict.reasons,
            max_body_bytes: cfg.max_body_bytes,
        };

        self.engine.note_captured();
        self.engine.submit(Job::Request(Box::new(record), event));

        if has_cookies {
            let batch = request_cookies
                .iter()
                .map(|c| crate::storage::requests::NewCookieEvent {
                    session_id: self.engine.session_id().to_string(),
                    request_id: request_id.clone(),
                    sequence_id,
                    direction: "sent",
                    name: c.name.clone(),
                    value: c.value.clone(),
                    domain: parsed.host.clone(),
                    path: "/".into(),
                    expires_at: None,
                    secure: false,
                    http_only: false,
                    same_site: None,
                })
                .collect::<Vec<_>>();
            self.engine.submit(Job::Cookies(batch));
        }

        self.pending = Some(Pending {
            request_id,
            sequence_id,
            started_at,
            host: parsed.host,
            path: parsed.path,
            query: parsed.query,
            method,
            has_auth,
            has_cookies,
            request_content_type: content_type,
            has_request_body: !body_bytes.is_empty(),
        });

        Request::from_parts(parts, body).into()
    }

    async fn handle_response(&mut self, _ctx: &HttpContext, res: Response<Body>) -> Response<Body> {
        let Some(pending) = self.pending.take() else { return res };

        let res = collect::decode_if_supported(res);

        let (parts, body) = res.into_parts();
        let hs = headers::from_map(&parts.headers);
        let content_type = headers::content_type(&hs);
        let cfg = self.engine.config();

        let (body_bytes, body) = if cfg.capture_response_bodies
            && self.engine.scope().content_type_allowed(content_type.as_deref())
        {
            collect::buffer(body, cfg.max_body_bytes).await
        } else {
            (Vec::new(), body)
        };

        let finished = now_millis();
        let status = parts.status.as_u16();
        let family = ContentFamily::from_content_type(content_type.as_deref());
        let protocol = headers::protocol_label(parts.version);

        let verdict = crate::analyzer::importance::score(&crate::analyzer::importance::Signals {
            method: &pending.method,
            host: &pending.host,
            path: &pending.path,
            query: pending.query.as_deref(),
            request_content_type: pending.request_content_type.as_deref(),
            response_content_type: content_type.as_deref(),
            status: Some(status),
            has_auth: pending.has_auth,
            has_cookies: pending.has_cookies,
            has_request_body: pending.has_request_body,
            response_size: body_bytes.len() as i64,
            has_path_id: false,
            relationship_count: 0,
            repeat_count: 1,
        });

        self.submit_set_cookies(&pending, &hs);

        let event = events::ResponseReceived {
            session_id: self.engine.session_id().to_string(),
            request_id: pending.request_id.clone(),
            sequence_id: pending.sequence_id,
            status,
            content_type: content_type.clone(),
            family: family.as_str().to_string(),
            body_size: body_bytes.len() as i64,
            duration_ms: finished - pending.started_at,
            importance: verdict.importance.as_str().to_string(),
        };

        let record = NewResponse {
            request_id: pending.request_id.clone(),
            session_id: self.engine.session_id().to_string(),
            status,
            status_text: parts.status.canonical_reason().unwrap_or("").to_string(),
            protocol,
            headers: hs,
            body: body_bytes,
            content_type,
            timestamp: finished,
            duration_ms: finished - pending.started_at,
            max_body_bytes: cfg.max_body_bytes,
            final_importance: verdict.importance,
            final_reasons: verdict.reasons,
        };

        self.engine.submit(Job::Response(Box::new(record), event));

        Response::from_parts(parts, body)
    }

    async fn handle_error(
        &mut self,
        _ctx: &HttpContext,
        err: hudsucker::hyper_util::client::legacy::Error,
    ) -> Response<Body> {
        if let Some(pending) = self.pending.take() {
            self.engine.submit(Job::Failed {
                request_id: pending.request_id,
                sequence_id: pending.sequence_id,
                message: err.to_string(),
            });
        }
        Response::builder()
            .status(hudsucker::hyper::StatusCode::BAD_GATEWAY)
            .body(Body::from(format!("vanguard: upstream request failed: {err}")))
            .expect("static response is valid")
    }
}

impl CaptureHandler {
    fn submit_set_cookies(&self, pending: &Pending, hs: &[Header]) {
        let set = cookies::parse_set_cookies(hs);
        if set.is_empty() {
            return;
        }
        let batch = set
            .into_iter()
            .map(|c| crate::storage::requests::NewCookieEvent {
                session_id: self.engine.session_id().to_string(),
                request_id: pending.request_id.clone(),
                sequence_id: pending.sequence_id,
                direction: "set",
                name: c.name,
                value: c.value,
                domain: c.domain.unwrap_or_else(|| pending.host.clone()),
                path: c.path,
                expires_at: c.expires_at,
                secure: c.secure,
                http_only: c.http_only,
                same_site: c.same_site,
            })
            .collect::<Vec<_>>();
        self.engine.submit(Job::Cookies(batch));
    }
}
