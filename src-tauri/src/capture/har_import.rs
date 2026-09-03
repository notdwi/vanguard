use crate::analyzer::{endpoints, importance};
use crate::error::Result;
use crate::http::{cookies, har, headers, url as urlutil};
use crate::models::{new_id, CaptureConfig, Header};
use crate::storage::requests::{NewCookieEvent, NewRequest, NewResponse};
use crate::storage::{requests as store, sessions, Db};

pub struct ImportOutcome {
    pub session_id: String,
    pub imported: i64,
    pub skipped: i64,
}

/// Rebuilds a session from a HAR file, keeping the file order as the timeline
/// order so sequence numbers still mean "what happened first".
pub fn import(db: &Db, name: &str, file: &har::HarFile) -> Result<ImportOutcome> {
    let session = sessions::create(db, name, &CaptureConfig::default())?;
    let max_body = session.config.max_body_bytes;
    let mut imported = 0i64;
    let mut skipped = 0i64;

    for (index, entry) in file.log.entries.iter().enumerate() {
        match import_entry(db, &session.id, index as i64 + 1, entry, max_body) {
            Ok(()) => imported += 1,
            Err(e) => {
                tracing::debug!("skipped HAR entry {index}: {e}");
                skipped += 1;
            }
        }
    }

    sessions::bump_counters(db, &session.id, imported, 0)?;
    sessions::set_status(db, &session.id, crate::models::CaptureState::Stopped)?;

    Ok(ImportOutcome { session_id: session.id, imported, skipped })
}

fn import_entry(
    db: &Db,
    session_id: &str,
    sequence_id: i64,
    entry: &har::HarEntry,
    max_body: i64,
) -> Result<()> {
    let parsed = urlutil::split(&entry.request.url)
        .ok_or_else(|| crate::error::AppError::Invalid("unparseable url".into()))?;

    let request_id = new_id("req");
    let started = har::parse_iso(&entry.started_date_time);
    let req_headers = har::from_pairs(&entry.request.headers);
    let res_headers = har::from_pairs(&entry.response.headers);

    let req_content_type = headers::content_type(&req_headers);
    let res_content_type = headers::content_type(&res_headers)
        .or_else(|| non_empty(&entry.response.content.mime_type));

    let req_body = entry
        .request
        .post_data
        .as_ref()
        .map(|p| p.text.as_bytes().to_vec())
        .unwrap_or_default();
    let res_body = decode_content(&entry.response.content);

    let has_auth = headers::get(&req_headers, "authorization").is_some();
    let request_cookies = cookies::parse_request_cookies(&req_headers);
    let has_cookies = !request_cookies.is_empty() || !entry.request.cookies.is_empty();
    let normalized_path = endpoints::normalize_path(&parsed.path);
    let is_api = endpoints::is_api_like(&parsed.path, res_content_type.as_deref());

    let verdict = importance::score(&importance::Signals {
        method: &entry.request.method,
        host: &parsed.host,
        path: &parsed.path,
        query: parsed.query.as_deref(),
        request_content_type: req_content_type.as_deref(),
        response_content_type: res_content_type.as_deref(),
        status: Some(entry.response.status),
        has_auth,
        has_cookies,
        has_request_body: !req_body.is_empty(),
        response_size: res_body.len() as i64,
        has_path_id: normalized_path.contains(':'),
        relationship_count: 0,
        repeat_count: 1,
    });

    store::insert_request(
        db,
        NewRequest {
            id: request_id.clone(),
            session_id: session_id.to_string(),
            sequence_id,
            timestamp: started,
            method: entry.request.method.to_ascii_uppercase(),
            url: parsed.url.clone(),
            scheme: parsed.scheme.clone(),
            host: parsed.host.clone(),
            port: parsed.port,
            path: parsed.path.clone(),
            query: parsed.query.clone(),
            normalized_path,
            protocol: non_empty(&entry.request.http_version).unwrap_or_else(|| "HTTP/1.1".into()),
            client_addr: None,
            remote_ip: entry.server_ip_address.clone(),
            headers: req_headers,
            body: req_body,
            content_type: req_content_type,
            has_cookies,
            has_auth,
            is_api,
            importance: verdict.importance,
            importance_reasons: verdict.reasons.clone(),
            max_body_bytes: max_body,
        },
    )?;

    let duration = entry.time.max(0.0) as i64;
    store::insert_response(
        db,
        NewResponse {
            request_id: request_id.clone(),
            session_id: session_id.to_string(),
            status: entry.response.status,
            status_text: entry.response.status_text.clone(),
            protocol: non_empty(&entry.response.http_version).unwrap_or_else(|| "HTTP/1.1".into()),
            headers: res_headers.clone(),
            body: res_body,
            content_type: res_content_type,
            timestamp: started + duration,
            duration_ms: duration,
            max_body_bytes: max_body,
            final_importance: verdict.importance,
            final_reasons: verdict.reasons,
        },
    )?;

    let mut events: Vec<NewCookieEvent> = request_cookies
        .iter()
        .map(|c| cookie_event(session_id, &request_id, sequence_id, "sent", &c.name, &c.value, &parsed.host))
        .collect();
    for sc in cookies::parse_set_cookies(&res_headers) {
        events.push(cookie_event(
            session_id,
            &request_id,
            sequence_id,
            "set",
            &sc.name,
            &sc.value,
            sc.domain.as_deref().unwrap_or(&parsed.host),
        ));
    }
    store::insert_cookie_events(db, &events)?;
    Ok(())
}

fn cookie_event(
    session_id: &str,
    request_id: &str,
    sequence_id: i64,
    direction: &'static str,
    name: &str,
    value: &str,
    domain: &str,
) -> NewCookieEvent {
    NewCookieEvent {
        session_id: session_id.to_string(),
        request_id: request_id.to_string(),
        sequence_id,
        direction,
        name: name.to_string(),
        value: value.to_string(),
        domain: domain.to_string(),
        path: "/".into(),
        expires_at: None,
        secure: false,
        http_only: false,
        same_site: None,
    }
}

fn decode_content(content: &har::HarContent) -> Vec<u8> {
    let Some(text) = content.text.as_ref() else { return Vec::new() };
    if content.encoding.as_deref() == Some("base64") {
        use base64::Engine;
        base64::engine::general_purpose::STANDARD
            .decode(text)
            .unwrap_or_else(|_| text.as_bytes().to_vec())
    } else {
        text.as_bytes().to_vec()
    }
}

fn non_empty(value: &str) -> Option<String> {
    let t = value.trim();
    if t.is_empty() {
        None
    } else {
        Some(t.to_string())
    }
}

pub fn header_pairs(hs: &[Header]) -> Vec<har::HarNameValue> {
    har::to_pairs(hs)
}
