use std::collections::HashMap;

use crate::http::{cookies, url as urlutil};
use crate::models::{DetectedToken, Header, TokenKind, TokenSource};

use super::ids;

pub struct TokenScan<'a> {
    pub request_id: &'a str,
    pub sequence_id: i64,
    pub headers: &'a [Header],
    pub query: Option<&'a str>,
}

/// Collects credentials and correlation values from one request.
pub fn scan(input: &TokenScan) -> Vec<DetectedToken> {
    let mut out = Vec::new();

    for h in input.headers {
        let lower = h.name.to_ascii_lowercase();
        if lower == "cookie" {
            for c in cookies::parse_request_cookies(std::slice::from_ref(h)) {
                if let Some(kind) = cookie_kind(&c.name, &c.value) {
                    out.push(token(kind, TokenSource::Cookie, &c.name, &c.value, input));
                }
            }
            continue;
        }
        if let Some(kind) = header_kind(&lower, &h.value) {
            out.push(token(kind, TokenSource::Header, &h.name, &h.value, input));
        }
    }

    for p in urlutil::parse_query(input.query) {
        if let Some(kind) = query_kind(&p.name, &p.value) {
            out.push(token(kind, TokenSource::Query, &p.name, &p.value, input));
        }
    }

    out
}

/// Folds per-request detections into one entry per distinct value.
pub fn merge(detections: Vec<DetectedToken>) -> Vec<DetectedToken> {
    let mut grouped: HashMap<(String, String), DetectedToken> = HashMap::new();

    for d in detections {
        let key = (d.name.to_ascii_lowercase(), d.value_hash.clone());
        grouped
            .entry(key)
            .and_modify(|existing| {
                for seq in &d.used_by {
                    if !existing.used_by.contains(seq) {
                        existing.used_by.push(*seq);
                    }
                }
            })
            .or_insert(d);
    }

    let mut out: Vec<DetectedToken> = grouped.into_values().collect();
    for t in out.iter_mut() {
        t.used_by.sort_unstable();
    }
    out.sort_by(|a, b| b.used_by.len().cmp(&a.used_by.len()).then_with(|| a.name.cmp(&b.name)));
    out
}

fn header_kind(lower: &str, value: &str) -> Option<TokenKind> {
    match lower {
        "authorization" | "proxy-authorization" => {
            let v = value.trim();
            let lower_v = v.to_ascii_lowercase();
            if lower_v.starts_with("bearer ") {
                let raw = v[7..].trim();
                Some(if ids::jwt_re().is_match(raw) { TokenKind::Jwt } else { TokenKind::Bearer })
            } else if lower_v.starts_with("basic ") {
                Some(TokenKind::Basic)
            } else {
                Some(TokenKind::Unknown)
            }
        }
        "x-api-key" | "api-key" | "x-app-key" => Some(TokenKind::ApiKey),
        "x-csrf-token" | "x-xsrf-token" | "csrf-token" => Some(TokenKind::Csrf),
        "x-request-id" | "x-correlation-id" | "x-trace-id" | "traceparent" => {
            Some(TokenKind::RequestId)
        }
        _ => {
            if lower.contains("token") && ids::is_candidate(value) {
                Some(TokenKind::Unknown)
            } else {
                None
            }
        }
    }
}

fn cookie_kind(name: &str, value: &str) -> Option<TokenKind> {
    let lower = name.to_ascii_lowercase();
    if !ids::is_candidate(value) {
        return None;
    }
    if lower.contains("csrf") || lower.contains("xsrf") {
        Some(TokenKind::Csrf)
    } else if lower.contains("sess") || lower.contains("sid") || lower.contains("auth") {
        Some(TokenKind::SessionId)
    } else if lower.contains("token") || lower.contains("jwt") {
        Some(TokenKind::Jwt)
    } else {
        None
    }
}

fn query_kind(name: &str, value: &str) -> Option<TokenKind> {
    let lower = name.to_ascii_lowercase();
    if !ids::is_candidate(value) {
        return None;
    }
    if lower.contains("api_key") || lower.contains("apikey") || lower == "key" {
        Some(TokenKind::ApiKey)
    } else if lower.contains("token") || lower.contains("access_token") {
        Some(TokenKind::Bearer)
    } else if lower.contains("csrf") {
        Some(TokenKind::Csrf)
    } else {
        None
    }
}

fn token(
    kind: TokenKind,
    source: TokenSource,
    name: &str,
    value: &str,
    input: &TokenScan,
) -> DetectedToken {
    DetectedToken {
        kind,
        source,
        name: name.to_string(),
        value_preview: preview(value),
        value_hash: short_hash(value),
        used_by: vec![input.sequence_id],
        first_seen_request_id: input.request_id.to_string(),
    }
}

pub fn preview(value: &str) -> String {
    let v = value.trim();
    if v.chars().count() <= 24 {
        return v.to_string();
    }
    let head: String = v.chars().take(20).collect();
    format!("{head}...")
}

/// Short digest so identical secrets group without the full value travelling
/// around the app more than necessary.
pub fn short_hash(value: &str) -> String {
    use sha2::{Digest, Sha256};
    let digest = Sha256::digest(value.trim().as_bytes());
    digest.iter().take(6).map(|b| format!("{b:02x}")).collect()
}
