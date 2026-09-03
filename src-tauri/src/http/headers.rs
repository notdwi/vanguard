use crate::models::Header;

pub const SENSITIVE: &[&str] = &[
    "authorization",
    "proxy-authorization",
    "cookie",
    "set-cookie",
    "x-api-key",
    "x-auth-token",
    "x-csrf-token",
    "x-xsrf-token",
    "api-key",
];

pub fn from_map(map: &http::HeaderMap) -> Vec<Header> {
    map.iter()
        .map(|(name, value)| Header {
            name: name.as_str().to_string(),
            value: String::from_utf8_lossy(value.as_bytes()).into_owned(),
        })
        .collect()
}

pub fn get<'a>(headers: &'a [Header], name: &str) -> Option<&'a str> {
    headers
        .iter()
        .find(|h| h.name.eq_ignore_ascii_case(name))
        .map(|h| h.value.as_str())
}

pub fn get_all<'a>(headers: &'a [Header], name: &str) -> Vec<&'a str> {
    headers
        .iter()
        .filter(|h| h.name.eq_ignore_ascii_case(name))
        .map(|h| h.value.as_str())
        .collect()
}

pub fn is_sensitive(name: &str) -> bool {
    let lower = name.to_ascii_lowercase();
    SENSITIVE.iter().any(|s| *s == lower)
        || lower.contains("token")
        || lower.contains("secret")
        || lower.contains("password")
}

/// Replaces the meaningful part of a secret while keeping its shape readable.
pub fn mask(value: &str) -> String {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return String::new();
    }
    if let Some((scheme, rest)) = trimmed.split_once(' ') {
        let known = ["bearer", "basic", "digest", "token"];
        if known.contains(&scheme.to_ascii_lowercase().as_str()) && !rest.trim().is_empty() {
            return format!("{scheme} {}", stars(rest.trim()));
        }
    }
    stars(trimmed)
}

fn stars(value: &str) -> String {
    let n = value.chars().count();
    let keep = if n > 12 { 4 } else { 0 };
    let head: String = value.chars().take(keep).collect();
    format!("{head}{}", "*".repeat(8))
}

pub fn content_type(headers: &[Header]) -> Option<String> {
    get(headers, "content-type").map(|v| v.to_string())
}

pub fn protocol_label(version: http::Version) -> String {
    match version {
        http::Version::HTTP_09 => "HTTP/0.9".into(),
        http::Version::HTTP_10 => "HTTP/1.0".into(),
        http::Version::HTTP_11 => "HTTP/1.1".into(),
        http::Version::HTTP_2 => "HTTP/2".into(),
        http::Version::HTTP_3 => "HTTP/3".into(),
        _ => "HTTP".into(),
    }
}

/// Headers the repeater must not copy verbatim: they describe the original
/// transport, not the request being replayed.
pub fn is_hop_by_hop(name: &str) -> bool {
    const HOP: &[&str] = &[
        "connection",
        "keep-alive",
        "proxy-authenticate",
        "proxy-connection",
        "te",
        "trailer",
        "transfer-encoding",
        "upgrade",
        "content-length",
        "host",
        "accept-encoding",
    ];
    let lower = name.to_ascii_lowercase();
    HOP.contains(&lower.as_str()) || lower.starts_with(':')
}
