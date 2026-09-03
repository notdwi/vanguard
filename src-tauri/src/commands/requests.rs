use serde::Serialize;
use tauri::State;

use crate::error::Result;
use crate::http::{cookies as cookie_util, curl, headers as header_util, url as urlutil};
use crate::models::{
    BodyPayload, CapturedRequest, CookiePair, QueryParam, TimelinePage, TimelineQuery,
};
use crate::state::AppState;
use crate::storage::{cookies as cookie_store, queries};

const PREVIEW_BODY_LIMIT: i64 = 512 * 1024;

#[derive(Serialize)]
pub struct RequestDetail {
    pub request: CapturedRequest,
    pub query: Vec<QueryParam>,
    pub request_cookies: Vec<CookiePair>,
    pub response_cookies: Vec<ResponseCookie>,
    pub cookie_origins: Vec<CookieOrigin>,
}

#[derive(Serialize)]
pub struct ResponseCookie {
    pub name: String,
    pub value: String,
    pub domain: Option<String>,
    pub path: String,
    pub secure: bool,
    pub http_only: bool,
    pub same_site: Option<String>,
}

#[derive(Serialize)]
pub struct CookieOrigin {
    pub name: String,
    pub direction: String,
    pub sequence_id: i64,
    pub value_preview: String,
}

#[tauri::command]
pub fn timeline(state: State<'_, AppState>, query: TimelineQuery) -> Result<TimelinePage> {
    let page = queries::timeline(&state.db, &query)?;
    if !query.search_bodies {
        return Ok(page);
    }
    Ok(filter_by_body(&state, &query, page))
}

#[tauri::command]
pub fn request_detail(state: State<'_, AppState>, request_id: String) -> Result<RequestDetail> {
    let request = queries::get_request(&state.db, &request_id)?;
    let query = urlutil::parse_query(request.query.as_deref());
    let request_cookies = cookie_util::parse_request_cookies(&request.request_headers);

    let response_cookies = request
        .response
        .as_ref()
        .map(|r| {
            cookie_util::parse_set_cookies(&r.headers)
                .into_iter()
                .map(|c| ResponseCookie {
                    name: c.name,
                    value: c.value,
                    domain: c.domain,
                    path: c.path,
                    secure: c.secure,
                    http_only: c.http_only,
                    same_site: c.same_site,
                })
                .collect()
        })
        .unwrap_or_default();

    let cookie_origins = cookie_store::for_request(&state.db, &request_id)?
        .into_iter()
        .map(|e| CookieOrigin {
            name: e.name,
            direction: e.direction,
            sequence_id: e.sequence_id,
            value_preview: cookie_store::preview(&e.value),
        })
        .collect();

    Ok(RequestDetail { request, query, request_cookies, response_cookies, cookie_origins })
}

#[tauri::command]
pub fn load_body(
    state: State<'_, AppState>,
    request_id: String,
    side: String,
    full: Option<bool>,
) -> Result<BodyPayload> {
    let limit = if full.unwrap_or(false) { 0 } else { PREVIEW_BODY_LIMIT };
    queries::load_body(&state.db, &request_id, &side, limit)
}

#[tauri::command]
pub fn copy_as_curl(
    state: State<'_, AppState>,
    request_id: String,
    mask_secrets: Option<bool>,
) -> Result<String> {
    let request = queries::get_request(&state.db, &request_id)?;
    let body = queries::load_body(&state.db, &request_id, "request", 1024 * 1024)
        .ok()
        .and_then(|p| if p.is_text { p.content } else { None });

    Ok(curl::build(
        &request.method,
        &request.url,
        &request.request_headers,
        body.as_deref(),
        &curl::CurlOptions {
            mask_secrets: mask_secrets.unwrap_or(false),
            multiline: true,
        },
    ))
}

#[tauri::command]
pub fn mask_header_value(name: String, value: String) -> String {
    if header_util::is_sensitive(&name) {
        header_util::mask(&value)
    } else {
        value
    }
}

/// Second pass for "search inside bodies": the SQL filter has already narrowed
/// the set, so this only reads bodies for rows that survived it.
fn filter_by_body(
    state: &State<'_, AppState>,
    query: &TimelineQuery,
    page: TimelinePage,
) -> TimelinePage {
    let Some(term) = query.search.as_ref().map(|s| s.trim().to_lowercase()) else { return page };
    if term.is_empty() {
        return page;
    }

    let mut rows = page.rows;
    let matches_meta = |row: &crate::models::TimelineRow| {
        row.path.to_lowercase().contains(&term)
            || row.host.to_lowercase().contains(&term)
            || row.query.as_deref().unwrap_or("").to_lowercase().contains(&term)
    };

    rows.retain(|row| {
        if matches_meta(row) {
            return true;
        }
        ["response", "request"].iter().any(|side| {
            queries::load_body(&state.db, &row.id, side, 512 * 1024)
                .ok()
                .and_then(|p| p.content.filter(|_| p.is_text))
                .map(|c| c.to_lowercase().contains(&term))
                .unwrap_or(false)
        })
    });

    let total = rows.len() as i64;
    TimelinePage { rows, total, offset: page.offset }
}
