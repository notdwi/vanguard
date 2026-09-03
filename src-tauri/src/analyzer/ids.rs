use std::sync::OnceLock;

use regex::Regex;
use serde_json::Value;

use crate::models::DetectedId;

/// Values worth following: long enough to be meaningful, short enough not to
/// be a payload, and not obviously a word.
pub fn is_candidate(value: &str) -> bool {
    let v = value.trim();
    let len = v.len();
    if !(4..=128).contains(&len) {
        return false;
    }
    if v.contains(' ') || v.contains('\n') {
        return false;
    }
    if v.chars().all(|c| c.is_ascii_digit()) {
        return len >= 4;
    }
    if uuid_re().is_match(v) {
        return true;
    }
    let has_digit = v.chars().any(|c| c.is_ascii_digit());
    let alnum = v.chars().all(|c| c.is_ascii_alphanumeric() || "-_.:".contains(c));
    alnum && (has_digit || len >= 16)
}

/// Field names that mean "this is an identifier". Short numeric values are
/// only worth following when the name says they identify something, otherwise
/// every `page=1` would look like a link.
pub fn is_identity_key(key: &str) -> bool {
    let leaf = key.rsplit('.').next().unwrap_or(key);
    let leaf = leaf.split('[').next().unwrap_or(leaf);
    if leaf.is_empty() {
        return false;
    }
    if leaf.ends_with("_id") || leaf.ends_with("Id") || leaf.ends_with("ID") {
        return true;
    }
    let lower = leaf.to_ascii_lowercase();
    matches!(
        lower.as_str(),
        "id" | "uuid" | "guid" | "slug" | "code" | "key" | "sku" | "ref" | "hash" | "token"
    )
}

/// Candidate test that takes the surrounding field name into account.
pub fn is_candidate_named(value: &str, key: Option<&str>) -> bool {
    if is_candidate(value) {
        return true;
    }
    let Some(key) = key else { return false };
    if !is_identity_key(key) {
        return false;
    }
    let v = value.trim();
    !v.is_empty() && v.len() <= 128 && !v.contains(' ') && v != "0" && v != "false" && v != "true"
}

/// A bare numeric path segment is almost always a record id, even when short.
pub fn is_path_identity(segment: &str) -> bool {
    let s = segment.trim();
    (1..=12).contains(&s.len())
        && s.chars().all(|c| c.is_ascii_digit())
        && s != "0"
        && !s.starts_with('0')
}

pub fn kind_of(value: &str) -> &'static str {
    if uuid_re().is_match(value) {
        "uuid"
    } else if value.chars().all(|c| c.is_ascii_digit()) {
        "numeric"
    } else if jwt_re().is_match(value) {
        "jwt"
    } else if value.len() >= 32 {
        "opaque"
    } else {
        "slug"
    }
}

pub fn from_request(path: &str, query: Option<&str>, body: Option<&str>) -> Vec<DetectedId> {
    let mut out = Vec::new();

    for segment in path.split('/').filter(|s| !s.is_empty()) {
        if is_candidate(segment) && super::endpoints::classify_segment(segment).is_some() {
            out.push(DetectedId {
                value: segment.to_string(),
                location: format!("path:{segment}"),
                kind: kind_of(segment).to_string(),
            });
        }
    }

    for p in crate::http::url::parse_query(query) {
        if is_candidate(&p.value) {
            out.push(DetectedId {
                value: p.value.clone(),
                location: format!("query:{}", p.name),
                kind: kind_of(&p.value).to_string(),
            });
        }
    }

    if let Some(body) = body {
        if let Ok(json) = serde_json::from_str::<Value>(body) {
            let mut found = Vec::new();
            walk_json(&json, "$", &mut found, 0);
            for (path, value) in found.into_iter().take(64) {
                out.push(DetectedId {
                    value: value.clone(),
                    location: format!("body:{path}"),
                    kind: kind_of(&value).to_string(),
                });
            }
        }
    }

    dedupe(out)
}

/// Collects scalar values from a JSON document along with their JSONPath.
pub fn walk_json(value: &Value, path: &str, out: &mut Vec<(String, String)>, depth: usize) {
    if depth > 8 || out.len() > 4000 {
        return;
    }
    match value {
        Value::Object(map) => {
            for (k, v) in map {
                walk_json(v, &format!("{path}.{k}"), out, depth + 1);
            }
        }
        Value::Array(items) => {
            for (i, v) in items.iter().enumerate().take(200) {
                walk_json(v, &format!("{path}[{i}]"), out, depth + 1);
            }
        }
        Value::String(s) => {
            if is_candidate_named(s, Some(path)) {
                out.push((path.to_string(), s.clone()));
            }
        }
        Value::Number(n) => {
            let s = n.to_string();
            if is_candidate_named(&s, Some(path)) {
                out.push((path.to_string(), s));
            }
        }
        _ => {}
    }
}

fn dedupe(mut ids: Vec<DetectedId>) -> Vec<DetectedId> {
    ids.sort_by(|a, b| a.value.cmp(&b.value).then_with(|| a.location.cmp(&b.location)));
    ids.dedup_by(|a, b| a.value == b.value && a.location == b.location);
    ids
}

pub fn uuid_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"(?i)^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$").unwrap()
    })
}

pub fn jwt_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"^eyJ[A-Za-z0-9_-]+\.[A-Za-z0-9_-]+\.[A-Za-z0-9_-]*$").unwrap())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identity_keys_admit_short_numeric_ids() {
        assert!(is_identity_key("$.products[0].id"));
        assert!(is_identity_key("$.user_id"));
        assert!(is_identity_key("postId"));
        assert!(!is_identity_key("$.page"));
        assert!(!is_identity_key("$.paid"));

        assert!(is_candidate_named("1", Some("$.products[0].id")));
        assert!(!is_candidate_named("1", Some("$.page")));
        assert!(!is_candidate_named("0", Some("$.id")));
    }

    #[test]
    fn numeric_path_segments_are_identities() {
        assert!(is_path_identity("1"));
        assert!(is_path_identity("4242"));
        assert!(!is_path_identity("0"));
        assert!(!is_path_identity("007"));
        assert!(!is_path_identity("posts"));
    }

    #[test]
    fn candidates_exclude_words_and_short_values() {
        assert!(!is_candidate("phone"));
        assert!(!is_candidate("12"));
        assert!(is_candidate("1234"));
        assert!(is_candidate("550e8400-e29b-41d4-a716-446655440000"));
        assert!(!is_candidate("hello world"));
    }
}
