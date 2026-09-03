use crate::models::{CookiePair, Header};

use super::headers;

#[derive(Debug, Clone)]
pub struct SetCookie {
    pub name: String,
    pub value: String,
    pub domain: Option<String>,
    pub path: String,
    pub expires_at: Option<i64>,
    pub max_age: Option<i64>,
    pub secure: bool,
    pub http_only: bool,
    pub same_site: Option<String>,
}

/// Parses a request `Cookie:` header into pairs.
pub fn parse_request_cookies(hs: &[Header]) -> Vec<CookiePair> {
    let mut out = Vec::new();
    for raw in headers::get_all(hs, "cookie") {
        for part in raw.split(';') {
            let part = part.trim();
            if part.is_empty() {
                continue;
            }
            let (name, value) = match part.split_once('=') {
                Some((n, v)) => (n.trim(), v.trim()),
                None => (part, ""),
            };
            if name.is_empty() {
                continue;
            }
            out.push(CookiePair { name: name.to_string(), value: value.to_string() });
        }
    }
    out
}

pub fn serialize_request_cookies(cookies: &[CookiePair]) -> String {
    cookies
        .iter()
        .map(|c| format!("{}={}", c.name, c.value))
        .collect::<Vec<_>>()
        .join("; ")
}

pub fn parse_set_cookies(hs: &[Header]) -> Vec<SetCookie> {
    headers::get_all(hs, "set-cookie").iter().filter_map(|v| parse_set_cookie(v)).collect()
}

pub fn parse_set_cookie(raw: &str) -> Option<SetCookie> {
    let mut parts = raw.split(';');
    let first = parts.next()?.trim();
    let (name, value) = first.split_once('=')?;
    let name = name.trim();
    if name.is_empty() {
        return None;
    }

    let mut cookie = SetCookie {
        name: name.to_string(),
        value: value.trim().to_string(),
        domain: None,
        path: "/".to_string(),
        expires_at: None,
        max_age: None,
        secure: false,
        http_only: false,
        same_site: None,
    };

    for attr in parts {
        let attr = attr.trim();
        let (key, val) = match attr.split_once('=') {
            Some((k, v)) => (k.trim().to_ascii_lowercase(), v.trim().to_string()),
            None => (attr.to_ascii_lowercase(), String::new()),
        };
        match key.as_str() {
            "domain" => cookie.domain = Some(val.trim_start_matches('.').to_string()),
            "path" if !val.is_empty() => cookie.path = val,
            "secure" => cookie.secure = true,
            "httponly" => cookie.http_only = true,
            "samesite" => cookie.same_site = Some(val),
            "max-age" => cookie.max_age = val.parse::<i64>().ok(),
            "expires" => cookie.expires_at = parse_http_date(&val),
            _ => {}
        }
    }
    Some(cookie)
}

fn parse_http_date(value: &str) -> Option<i64> {
    let formats = ["%a, %d %b %Y %H:%M:%S GMT", "%A, %d-%b-%y %H:%M:%S GMT", "%a %b %e %H:%M:%S %Y"];
    for f in formats {
        if let Ok(dt) = chrono::NaiveDateTime::parse_from_str(value, f) {
            return Some(dt.and_utc().timestamp_millis());
        }
    }
    None
}
