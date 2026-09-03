use std::collections::BTreeMap;

use serde_json::Value;

use crate::models::{Comparison, ComparisonSide, DiffEntry, DiffKind, Header};

const VOLATILE_KEYS: &[&str] = &[
    "timestamp", "time", "date", "requestid", "request_id", "traceid", "trace_id", "nonce",
    "etag", "expires", "age", "server-timing", "x-request-id", "x-amz-cf-id", "set-cookie",
    "signature", "csrf", "sessionid", "session_id", "correlationid",
];

pub fn compare(left: ComparisonSide, right: ComparisonSide) -> Comparison {
    let header_diff = diff_headers(&left.headers, &right.headers);
    let (body_diff, body_comparable) = diff_bodies(left.body.as_deref(), right.body.as_deref());
    Comparison { left, right, header_diff, body_diff, body_comparable }
}

fn diff_headers(left: &[Header], right: &[Header]) -> Vec<DiffEntry> {
    let l = index(left);
    let r = index(right);
    let mut out = Vec::new();

    for (name, lv) in &l {
        match r.get(name) {
            Some(rv) if rv == lv => {}
            Some(rv) => out.push(entry(DiffKind::Changed, name, Some(lv), Some(rv))),
            None => out.push(entry(DiffKind::Removed, name, Some(lv), None)),
        }
    }
    for (name, rv) in &r {
        if !l.contains_key(name) {
            out.push(entry(DiffKind::Added, name, None, Some(rv)));
        }
    }
    out
}

fn index(hs: &[Header]) -> BTreeMap<String, String> {
    let mut map = BTreeMap::new();
    for h in hs {
        map.entry(h.name.to_ascii_lowercase())
            .and_modify(|v: &mut String| {
                v.push_str(", ");
                v.push_str(&h.value);
            })
            .or_insert_with(|| h.value.clone());
    }
    map
}

/// JSON-aware when both sides parse; otherwise the caller shows raw text.
fn diff_bodies(left: Option<&str>, right: Option<&str>) -> (Vec<DiffEntry>, bool) {
    let (Some(l), Some(r)) = (left, right) else { return (Vec::new(), false) };
    let (Ok(lv), Ok(rv)) = (
        serde_json::from_str::<Value>(l),
        serde_json::from_str::<Value>(r),
    ) else {
        return (Vec::new(), false);
    };

    let mut out = Vec::new();
    walk(&lv, &rv, "$", &mut out, 0);
    out.sort_by(|a, b| a.path.cmp(&b.path));
    out.truncate(400);
    (out, true)
}

fn walk(left: &Value, right: &Value, path: &str, out: &mut Vec<DiffEntry>, depth: usize) {
    if depth > 12 || out.len() > 400 {
        return;
    }
    match (left, right) {
        (Value::Object(a), Value::Object(b)) => {
            for (k, av) in a {
                let child = format!("{path}.{k}");
                match b.get(k) {
                    Some(bv) => walk(av, bv, &child, out, depth + 1),
                    None => out.push(entry(DiffKind::Removed, &child, Some(&render(av)), None)),
                }
            }
            for (k, bv) in b {
                if !a.contains_key(k) {
                    let child = format!("{path}.{k}");
                    out.push(entry(DiffKind::Added, &child, None, Some(&render(bv))));
                }
            }
        }
        (Value::Array(a), Value::Array(b)) => {
            if a.len() != b.len() {
                out.push(entry(
                    DiffKind::Changed,
                    &format!("{path}.length"),
                    Some(&a.len().to_string()),
                    Some(&b.len().to_string()),
                ));
            }
            for i in 0..a.len().min(b.len()).min(100) {
                walk(&a[i], &b[i], &format!("{path}[{i}]"), out, depth + 1);
            }
        }
        (a, b) if a != b => {
            out.push(entry(DiffKind::Changed, path, Some(&render(a)), Some(&render(b))));
        }
        _ => {}
    }
}

fn render(value: &Value) -> String {
    match value {
        Value::String(s) => s.clone(),
        other => other.to_string(),
    }
}

fn entry(kind: DiffKind, path: &str, left: Option<&String>, right: Option<&String>) -> DiffEntry {
    DiffEntry {
        kind,
        path: path.to_string(),
        left: left.map(|v| truncate(v)),
        right: right.map(|v| truncate(v)),
        volatile: is_volatile(path),
    }
}

/// Marks paths whose value is expected to change on every call, so a diff full
/// of timestamps does not read as a real behavioural difference.
fn is_volatile(path: &str) -> bool {
    let leaf = path
        .rsplit(['.', '[', ']'])
        .find(|s| !s.is_empty())
        .unwrap_or(path)
        .to_ascii_lowercase()
        .replace(['-', '_'], "");
    VOLATILE_KEYS.iter().any(|k| leaf == k.replace(['-', '_'], ""))
}

fn truncate(value: &str) -> String {
    if value.chars().count() <= 200 {
        return value.to_string();
    }
    let head: String = value.chars().take(197).collect();
    format!("{head}...")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_added_removed_and_changed() {
        let (diff, comparable) = diff_bodies(
            Some(r#"{"products":[1],"requestId":"a"}"#),
            Some(r#"{"error":"denied","requestId":"b"}"#),
        );
        assert!(comparable);
        assert!(diff.iter().any(|d| d.path == "$.products" && d.kind == DiffKind::Removed));
        assert!(diff.iter().any(|d| d.path == "$.error" && d.kind == DiffKind::Added));
        assert!(diff.iter().any(|d| d.path == "$.requestId" && d.volatile));
    }
}
