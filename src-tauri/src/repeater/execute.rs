use std::time::Duration;

use reqwest::redirect::Policy;
use reqwest::Client;

use crate::error::{AppError, Result};
use crate::models::{
    new_id, now_millis, Header, RepeaterDraft, RepeaterSnapshot, ReplayOptions, ReplayResult,
};

use super::draft;

const MAX_KEPT_BODY: usize = 4 * 1024 * 1024;

pub fn client(options: &ReplayOptions) -> Result<Client> {
    let policy = if options.follow_redirects { Policy::limited(10) } else { Policy::none() };
    Client::builder()
        .redirect(policy)
        .timeout(Duration::from_millis(options.timeout_ms.clamp(1_000, 300_000)))
        .user_agent("vanguard-repeater")
        .build()
        .map_err(|e| AppError::Http(format!("could not build the HTTP client: {e}")))
}

/// Sends one copy of the draft and records exactly what went out.
pub async fn send(
    client: &Client,
    d: &RepeaterDraft,
    index: u32,
) -> ReplayResult {
    let url = draft::effective_url(d);
    let hs = draft::effective_headers(d);
    let snapshot = RepeaterSnapshot {
        method: d.method.to_ascii_uppercase(),
        url: url.clone(),
        headers: hs.clone(),
        body: d.body.clone(),
    };

    let started_at = now_millis();
    let mut result = ReplayResult {
        id: new_id("rpl"),
        draft_id: d.id.clone(),
        session_id: d.session_id.clone(),
        index,
        started_at,
        duration_ms: 0,
        status: None,
        status_text: String::new(),
        protocol: String::new(),
        headers: Vec::new(),
        body: None,
        body_size: 0,
        body_is_text: false,
        content_type: None,
        error: None,
        sent: snapshot,
    };

    let method = match reqwest::Method::from_bytes(d.method.to_ascii_uppercase().as_bytes()) {
        Ok(m) => m,
        Err(_) => {
            result.error = Some(format!("unsupported method `{}`", d.method));
            return result;
        }
    };

    let mut builder = client.request(method, &url);
    for h in &hs {
        builder = builder.header(&h.name, &h.value);
    }
    if !d.body.is_empty() {
        builder = builder.body(d.body.clone());
    }

    match builder.send().await {
        Ok(response) => fill(&mut result, response, started_at).await,
        Err(e) => {
            result.duration_ms = now_millis() - started_at;
            result.error = Some(e.to_string());
        }
    }
    result
}

async fn fill(result: &mut ReplayResult, response: reqwest::Response, started_at: i64) {
    result.status = Some(response.status().as_u16());
    result.status_text = response.status().canonical_reason().unwrap_or("").to_string();
    result.protocol = format!("{:?}", response.version());
    result.headers = response
        .headers()
        .iter()
        .map(|(k, v)| Header {
            name: k.as_str().to_string(),
            value: String::from_utf8_lossy(v.as_bytes()).into_owned(),
        })
        .collect();
    result.content_type = result
        .headers
        .iter()
        .find(|h| h.name.eq_ignore_ascii_case("content-type"))
        .map(|h| h.value.clone());

    match response.bytes().await {
        Ok(bytes) => {
            result.duration_ms = now_millis() - started_at;
            result.body_size = bytes.len() as i64;
            let slice = &bytes[..bytes.len().min(MAX_KEPT_BODY)];
            match std::str::from_utf8(slice) {
                Ok(text) => {
                    result.body_is_text = true;
                    result.body = Some(text.to_string());
                }
                Err(_) => {
                    use base64::Engine;
                    result.body_is_text = false;
                    result.body =
                        Some(base64::engine::general_purpose::STANDARD.encode(slice));
                }
            }
        }
        Err(e) => {
            result.duration_ms = now_millis() - started_at;
            result.error = Some(format!("could not read the response body: {e}"));
        }
    }
}
