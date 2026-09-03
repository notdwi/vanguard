use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
use url::Url;

use crate::models::QueryParam;

pub struct UrlParts {
    pub url: String,
    pub scheme: String,
    pub host: String,
    pub port: u16,
    pub path: String,
    pub query: Option<String>,
}

pub fn split(raw: &str) -> Option<UrlParts> {
    let parsed = Url::parse(raw).ok()?;
    let scheme = parsed.scheme().to_string();
    let host = parsed.host_str()?.to_string();
    let port = parsed.port_or_known_default().unwrap_or(if scheme == "https" { 443 } else { 80 });
    let path = if parsed.path().is_empty() { "/".to_string() } else { parsed.path().to_string() };
    Some(UrlParts {
        url: parsed.to_string(),
        scheme,
        host,
        port,
        path,
        query: parsed.query().map(|q| q.to_string()),
    })
}

pub fn parse_query(query: Option<&str>) -> Vec<QueryParam> {
    let Some(q) = query.filter(|q| !q.is_empty()) else { return Vec::new() };
    let full = format!("http://x/?{q}");
    match Url::parse(&full) {
        Ok(u) => u
            .query_pairs()
            .map(|(k, v)| QueryParam { name: k.into_owned(), value: v.into_owned() })
            .collect(),
        Err(_) => Vec::new(),
    }
}

pub fn build_query(params: &[QueryParam]) -> String {
    params
        .iter()
        .map(|p| {
            format!(
                "{}={}",
                utf8_percent_encode(&p.name, NON_ALPHANUMERIC),
                utf8_percent_encode(&p.value, NON_ALPHANUMERIC)
            )
        })
        .collect::<Vec<_>>()
        .join("&")
}

/// Replaces the query string of `base` with `params`, keeping everything else.
pub fn with_query(base: &str, params: &[QueryParam]) -> String {
    let Ok(mut parsed) = Url::parse(base) else { return base.to_string() };
    if params.is_empty() {
        parsed.set_query(None);
    } else {
        parsed.set_query(Some(&build_query(params)));
    }
    parsed.to_string()
}

pub fn strip_query(raw: &str) -> String {
    match raw.split_once('?') {
        Some((head, _)) => head.to_string(),
        None => raw.to_string(),
    }
}

/// Reconstructs an absolute URL from the pieces hudsucker gives us. Plain HTTP
/// proxying yields absolute URIs; tunnelled HTTPS yields origin-form paths.
pub fn absolute(uri: &http::Uri, host_header: Option<&str>, tls: bool) -> Option<String> {
    if uri.scheme().is_some() && uri.authority().is_some() {
        return Some(uri.to_string());
    }
    let authority = uri
        .authority()
        .map(|a| a.to_string())
        .or_else(|| host_header.map(|h| h.to_string()))?;
    let scheme = uri.scheme_str().unwrap_or(if tls { "https" } else { "http" });
    let path = uri.path_and_query().map(|p| p.as_str()).unwrap_or("/");
    Some(format!("{scheme}://{authority}{path}"))
}
