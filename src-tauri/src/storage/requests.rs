use rusqlite::params;

use crate::error::Result;
use crate::models::{now_millis, ContentFamily, Header, Importance};

use super::{blobs, Db};

/// Everything the capture engine knows about one exchange, with bodies still
/// held as raw bytes.
pub struct NewRequest {
    pub id: String,
    pub session_id: String,
    pub sequence_id: i64,
    pub timestamp: i64,
    pub method: String,
    pub url: String,
    pub scheme: String,
    pub host: String,
    pub port: u16,
    pub path: String,
    pub query: Option<String>,
    pub normalized_path: String,
    pub protocol: String,
    pub client_addr: Option<String>,
    pub remote_ip: Option<String>,
    pub headers: Vec<Header>,
    pub body: Vec<u8>,
    pub content_type: Option<String>,
    pub has_cookies: bool,
    pub has_auth: bool,
    pub is_api: bool,
    pub importance: Importance,
    pub importance_reasons: Vec<String>,
    pub max_body_bytes: i64,
}

pub struct NewResponse {
    pub request_id: String,
    pub session_id: String,
    pub status: u16,
    pub status_text: String,
    pub protocol: String,
    pub headers: Vec<Header>,
    pub body: Vec<u8>,
    pub content_type: Option<String>,
    pub timestamp: i64,
    pub duration_ms: i64,
    pub max_body_bytes: i64,
    /// Recomputed once the response is known, so the timeline settles on a
    /// verdict that accounts for status and content type.
    pub final_importance: Importance,
    pub final_reasons: Vec<String>,
}

pub fn insert_request(db: &Db, req: NewRequest) -> Result<()> {
    let stored = blobs::store(
        &db.blob_root(),
        &req.session_id,
        &req.id,
        "req",
        &req.body,
        req.max_body_bytes,
        req.content_type.as_deref(),
    )?;
    let body_ref = serde_json::to_string(&stored.reference)?;
    let headers = serde_json::to_string(&req.headers)?;
    let reasons = serde_json::to_string(&req.importance_reasons)?;

    db.with(|c| {
        c.execute(
            "INSERT INTO requests(
                id, session_id, sequence_id, timestamp, method, url, scheme, host, port, path,
                query, normalized_path, protocol, client_addr, remote_ip, request_headers,
                request_body_ref, request_body_blob, request_size, request_content_type,
                has_request_body, has_cookies, has_auth, is_api, importance, importance_reasons)
             VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17,?18,?19,?20,?21,?22,?23,?24,?25,?26)",
            params![
                req.id,
                req.session_id,
                req.sequence_id,
                req.timestamp,
                req.method,
                req.url,
                req.scheme,
                req.host,
                req.port,
                req.path,
                req.query,
                req.normalized_path,
                req.protocol,
                req.client_addr,
                req.remote_ip,
                headers,
                body_ref,
                stored.inline,
                stored.reference.size,
                req.content_type,
                (stored.reference.size > 0) as i32,
                req.has_cookies as i32,
                req.has_auth as i32,
                req.is_api as i32,
                req.importance.as_str(),
                reasons,
            ],
        )?;
        Ok(())
    })
}

pub fn insert_response(db: &Db, res: NewResponse) -> Result<()> {
    let stored = blobs::store(
        &db.blob_root(),
        &res.session_id,
        &res.request_id,
        "res",
        &res.body,
        res.max_body_bytes,
        res.content_type.as_deref(),
    )?;
    let body_ref = serde_json::to_string(&stored.reference)?;
    let headers = serde_json::to_string(&res.headers)?;
    let family = ContentFamily::from_content_type(res.content_type.as_deref());

    db.with(|c| {
        c.execute(
            "INSERT INTO responses(
                request_id, session_id, status, status_text, protocol, headers, body_ref,
                body_blob, body_size, content_type, family, timestamp, duration_ms)
             VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13)
             ON CONFLICT(request_id) DO NOTHING",
            params![
                res.request_id,
                res.session_id,
                res.status,
                res.status_text,
                res.protocol,
                headers,
                body_ref,
                stored.inline,
                stored.reference.size,
                res.content_type,
                family.as_str(),
                res.timestamp,
                res.duration_ms,
            ],
        )?;
        Ok(())
    })
}

pub fn mark_failed(db: &Db, request_id: &str, message: &str) -> Result<()> {
    db.with(|c| {
        c.execute("UPDATE requests SET error = ?2 WHERE id = ?1", params![request_id, message])?;
        Ok(())
    })
}

pub fn set_importance(
    db: &Db,
    request_id: &str,
    importance: Importance,
    reasons: &[String],
) -> Result<()> {
    let reasons = serde_json::to_string(reasons)?;
    db.with(|c| {
        c.execute(
            "UPDATE requests SET importance = ?2, importance_reasons = ?3 WHERE id = ?1",
            params![request_id, importance.as_str(), reasons],
        )?;
        Ok(())
    })
}

pub struct NewCookieEvent {
    pub session_id: String,
    pub request_id: String,
    pub sequence_id: i64,
    pub direction: &'static str,
    pub name: String,
    pub value: String,
    pub domain: String,
    pub path: String,
    pub expires_at: Option<i64>,
    pub secure: bool,
    pub http_only: bool,
    pub same_site: Option<String>,
}

pub fn insert_cookie_events(db: &Db, events: &[NewCookieEvent]) -> Result<()> {
    if events.is_empty() {
        return Ok(());
    }
    let ts = now_millis();
    db.with_tx(|tx| {
        let mut stmt = tx.prepare(
            "INSERT INTO cookie_events(session_id, request_id, sequence_id, direction, name,
                value, domain, path, expires_at, secure, http_only, same_site, timestamp)
             VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13)",
        )?;
        for e in events {
            stmt.execute(params![
                e.session_id,
                e.request_id,
                e.sequence_id,
                e.direction,
                e.name,
                e.value,
                e.domain,
                e.path,
                e.expires_at,
                e.secure as i32,
                e.http_only as i32,
                e.same_site,
                ts,
            ])?;
        }
        Ok(())
    })
}
