use rusqlite::types::Value;

use crate::models::TimelineQuery;

/// Translates a TimelineQuery into a WHERE clause plus positional arguments.
/// Ordering is never touched here: the timeline is always sequence order.
pub fn build_where(q: &TimelineQuery) -> (String, Vec<Value>) {
    let mut clauses: Vec<String> = vec!["r.session_id = ?".into()];
    let mut args: Vec<Value> = vec![Value::Text(q.session_id.clone())];

    if let Some(term) = q.search.as_ref().map(|s| s.trim()).filter(|s| !s.is_empty()) {
        let like = format!("%{}%", term.trim_matches('"'));
        clauses.push(
            "(r.url LIKE ? OR r.path LIKE ? OR r.host LIKE ? OR r.query LIKE ?
              OR r.request_headers LIKE ? OR s.headers LIKE ?)"
                .into(),
        );
        for _ in 0..6 {
            args.push(Value::Text(like.clone()));
        }
    }

    push_in(&mut clauses, &mut args, "r.method", &upper(&q.methods));
    push_in(&mut clauses, &mut args, "r.host", &q.hosts);
    push_in(&mut clauses, &mut args, "s.family", &q.families);
    push_in(&mut clauses, &mut args, "r.importance", &lower(&q.importance));

    if !q.status_classes.is_empty() {
        let parts: Vec<String> = q
            .status_classes
            .iter()
            .filter(|c| (1..=5).contains(*c))
            .map(|c| format!("(s.status >= {} AND s.status < {})", c * 100, (c + 1) * 100))
            .collect();
        if !parts.is_empty() {
            clauses.push(format!("({})", parts.join(" OR ")));
        }
    }

    if q.only_api {
        clauses.push("r.is_api = 1".into());
    }
    if q.only_errors {
        clauses.push("(s.status >= 400 OR r.error IS NOT NULL)".into());
    }
    if q.only_json {
        clauses.push("s.family = 'json'".into());
    }
    if q.only_with_cookies {
        clauses.push("r.has_cookies = 1".into());
    }
    if q.only_with_body {
        clauses.push("r.has_request_body = 1".into());
    }
    if q.only_authenticated {
        clauses.push("r.has_auth = 1".into());
    }

    (clauses.join(" AND "), args)
}

fn push_in(clauses: &mut Vec<String>, args: &mut Vec<Value>, column: &str, values: &[String]) {
    if values.is_empty() {
        return;
    }
    let holes = vec!["?"; values.len()].join(",");
    clauses.push(format!("{column} IN ({holes})"));
    for v in values {
        args.push(Value::Text(v.clone()));
    }
}

fn upper(values: &[String]) -> Vec<String> {
    values.iter().map(|v| v.to_ascii_uppercase()).collect()
}

fn lower(values: &[String]) -> Vec<String> {
    values.iter().map(|v| v.to_ascii_lowercase()).collect()
}
