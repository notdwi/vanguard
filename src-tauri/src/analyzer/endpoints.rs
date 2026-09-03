use std::sync::OnceLock;

use regex::Regex;

/// Collapses volatile path segments into `:id` style placeholders so that
/// /api/product/123 and /api/product/456 group together. The original URL is
/// always kept alongside this.
pub fn normalize_path(path: &str) -> String {
    if path == "/" || path.is_empty() {
        return "/".to_string();
    }
    let normalized: Vec<String> = path
        .split('/')
        .map(|segment| match classify_segment(segment) {
            Some(kind) => format!(":{kind}"),
            None => segment.to_string(),
        })
        .collect();
    let joined = normalized.join("/");
    if joined.is_empty() {
        "/".to_string()
    } else {
        joined
    }
}

/// Returns the placeholder name when a segment looks like an identifier.
pub fn classify_segment(segment: &str) -> Option<&'static str> {
    if segment.is_empty() {
        return None;
    }
    if uuid_re().is_match(segment) {
        return Some("uuid");
    }
    // Record ids are often small and sequential, so a bare numeric segment
    // counts regardless of length: /posts/1 and /posts/1234 both group.
    if segment.chars().all(|c| c.is_ascii_digit()) {
        return Some("id");
    }
    if hash_re().is_match(segment) {
        return Some("hash");
    }
    if date_re().is_match(segment) {
        return Some("date");
    }
    if mixed_id_re().is_match(segment) && has_digit(segment) && segment.len() >= 8 {
        return Some("id");
    }
    None
}

/// Heuristic for "this looks like a data endpoint rather than an asset".
pub fn is_api_like(path: &str, content_type: Option<&str>) -> bool {
    let lower = path.to_ascii_lowercase();
    if is_static_asset(&lower) {
        return false;
    }
    let ct = content_type.unwrap_or("").to_ascii_lowercase();
    if ct.contains("json") || ct.contains("graphql") || ct.contains("grpc") {
        return true;
    }
    const MARKERS: &[&str] = &[
        "/api/", "/api.", "/v1/", "/v2/", "/v3/", "/graphql", "/rest/", "/rpc/", "/gql",
        "/query", "/services/", "/service/", "/ajax/", "/_next/data/", "/wp-json/",
    ];
    MARKERS.iter().any(|m| lower.contains(m)) || lower.starts_with("/api")
}

pub fn is_static_asset(path: &str) -> bool {
    const EXT: &[&str] = &[
        ".js", ".mjs", ".css", ".png", ".jpg", ".jpeg", ".gif", ".webp", ".avif", ".svg", ".ico",
        ".woff", ".woff2", ".ttf", ".otf", ".eot", ".map", ".mp4", ".webm", ".mp3", ".wav",
        ".pdf", ".zip", ".gz",
    ];
    let clean = path.split('?').next().unwrap_or(path);
    EXT.iter().any(|e| clean.ends_with(e))
}

/// Third-party analytics and ad endpoints that rarely matter to a crawler.
pub fn is_noise_host(host: &str) -> bool {
    const NOISE: &[&str] = &[
        "google-analytics.com",
        "googletagmanager.com",
        "doubleclick.net",
        "facebook.net",
        "facebook.com",
        "hotjar.com",
        "segment.io",
        "sentry.io",
        "newrelic.com",
        "clarity.ms",
        "amplitude.com",
        "mixpanel.com",
        "intercom.io",
        "detectportal.firefox.com",
        "push.services.mozilla.com",
        "incoming.telemetry.mozilla.org",
        "safebrowsing.googleapis.com",
        "optimizationguide-pa.googleapis.com",
    ];
    let h = host.to_ascii_lowercase();
    NOISE.iter().any(|n| h == *n || h.ends_with(&format!(".{n}")))
}

fn has_digit(s: &str) -> bool {
    s.chars().any(|c| c.is_ascii_digit())
}

fn uuid_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"(?i)^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$").unwrap()
    })
}

fn hash_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"(?i)^[0-9a-f]{16,}$").unwrap())
}

fn date_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"^\d{4}-\d{2}(-\d{2})?$").unwrap())
}

fn mixed_id_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"^[A-Za-z0-9_-]+$").unwrap())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn collapses_short_and_long_ids_alike() {
        assert_eq!(normalize_path("/posts/1"), "/posts/:id");
        assert_eq!(normalize_path("/posts/2/comments"), "/posts/:id/comments");
        assert_eq!(normalize_path("/albums/1/photos"), "/albums/:id/photos");
    }

    #[test]
    fn collapses_numeric_and_uuid_segments() {
        assert_eq!(normalize_path("/api/product/123"), "/api/product/:id");
        assert_eq!(
            normalize_path("/api/u/550e8400-e29b-41d4-a716-446655440000"),
            "/api/u/:uuid"
        );
        assert_eq!(normalize_path("/api/search"), "/api/search");
        assert_eq!(normalize_path("/"), "/");
    }

    #[test]
    fn assets_are_not_api() {
        assert!(!is_api_like("/assets/app.js", None));
        assert!(is_api_like("/api/search", None));
        assert!(is_api_like("/anything", Some("application/json")));
    }
}
