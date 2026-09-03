use crate::error::Result;
use crate::http::{cookies, headers, url as urlutil};
use crate::models::{
    new_id, now_millis, BodyStorage, CapturedRequest, CookiePair, Header, RepeaterDraft,
};
use crate::storage::{queries, Db};

/// Builds an editable copy of a capture. The captured row is read-only and is
/// never touched by anything in this module.
pub fn from_capture(db: &Db, request_id: &str) -> Result<RepeaterDraft> {
    let captured: CapturedRequest = queries::get_request(db, request_id)?;

    let body = if captured.request_body.storage == BodyStorage::None {
        String::new()
    } else {
        queries::load_body(db, request_id, "request", 2 * 1024 * 1024)
            .ok()
            .and_then(|p| if p.is_text { p.content } else { None })
            .unwrap_or_default()
    };

    let cookie_list = cookies::parse_request_cookies(&captured.request_headers);
    let editable_headers: Vec<Header> = captured
        .request_headers
        .iter()
        .filter(|h| !h.name.eq_ignore_ascii_case("cookie") && !headers::is_hop_by_hop(&h.name))
        .cloned()
        .collect();

    let now = now_millis();
    Ok(RepeaterDraft {
        id: new_id("dft"),
        session_id: captured.session_id,
        source_request_id: Some(captured.id),
        source_sequence_id: Some(captured.sequence_id),
        label: format!("#{:03} {} {}", captured.sequence_id, captured.method, captured.path),
        method: captured.method,
        url: urlutil::strip_query(&captured.url),
        query: urlutil::parse_query(captured.query.as_deref()),
        headers: editable_headers,
        cookies: cookie_list,
        body,
        created_at: now,
        updated_at: now,
    })
}

pub fn blank(session_id: &str) -> RepeaterDraft {
    let now = now_millis();
    RepeaterDraft {
        id: new_id("dft"),
        session_id: session_id.to_string(),
        source_request_id: None,
        source_sequence_id: None,
        label: "New request".into(),
        method: "GET".into(),
        url: "https://".into(),
        query: Vec::new(),
        headers: vec![Header { name: "accept".into(), value: "*/*".into() }],
        cookies: Vec::new(),
        body: String::new(),
        created_at: now,
        updated_at: now,
    }
}

pub fn effective_url(draft: &RepeaterDraft) -> String {
    urlutil::with_query(&draft.url, &draft.query)
}

/// Merges the editable cookie list back into a single Cookie header.
pub fn effective_headers(draft: &RepeaterDraft) -> Vec<Header> {
    let mut out: Vec<Header> = draft
        .headers
        .iter()
        .filter(|h| !h.name.trim().is_empty() && !h.name.eq_ignore_ascii_case("cookie"))
        .cloned()
        .collect();

    let active: Vec<CookiePair> =
        draft.cookies.iter().filter(|c| !c.name.trim().is_empty()).cloned().collect();
    if !active.is_empty() {
        out.push(Header {
            name: "cookie".into(),
            value: cookies::serialize_request_cookies(&active),
        });
    }
    out
}
