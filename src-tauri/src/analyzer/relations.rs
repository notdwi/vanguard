use std::collections::HashMap;

use crate::http::{cookies, headers, url as urlutil};
use crate::models::{LinkKind, Relationship};

use super::dataset::{AnalysisRow, Dataset};
use super::ids;

struct Producer {
    request_id: String,
    sequence_id: i64,
    path: String,
    json_path: Option<String>,
    kind: LinkKind,
}

const MAX_LINKS: usize = 600;
const MAX_VALUES_PER_RESPONSE: usize = 250;

/// Finds values that one exchange produced and a later request consumed.
/// A single forward pass guarantees the producer always came first.
pub fn detect(data: &Dataset) -> Vec<Relationship> {
    let mut produced: HashMap<String, Producer> = HashMap::new();
    let mut links: Vec<Relationship> = Vec::new();

    for row in &data.rows {
        if links.len() < MAX_LINKS {
            consume(row, &produced, &mut links, data);
        }
        produce(row, &mut produced, data);
    }

    links.sort_by(|a, b| {
        b.confidence
            .partial_cmp(&a.confidence)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.to_sequence_id.cmp(&b.to_sequence_id))
    });
    links
}

fn produce(row: &AnalysisRow, produced: &mut HashMap<String, Producer>, data: &Dataset) {
    for sc in cookies::parse_set_cookies(&row.response_headers) {
        if ids::is_candidate(&sc.value) {
            produced.insert(
                sc.value.clone(),
                Producer {
                    request_id: row.id.clone(),
                    sequence_id: row.sequence_id,
                    path: row.path.clone(),
                    json_path: Some(format!("Set-Cookie: {}", sc.name)),
                    kind: LinkKind::Cookie,
                },
            );
        }
    }

    for h in &row.response_headers {
        let lower = h.name.to_ascii_lowercase();
        if (lower.contains("token") || lower.contains("id") || lower == "location")
            && ids::is_candidate(&h.value)
        {
            produced.entry(h.value.clone()).or_insert(Producer {
                request_id: row.id.clone(),
                sequence_id: row.sequence_id,
                path: row.path.clone(),
                json_path: Some(format!("header: {}", h.name)),
                kind: LinkKind::HeaderValue,
            });
        }
    }

    let Some(body) = data.response_bodies.get(&row.id) else { return };
    let Ok(json) = serde_json::from_str::<serde_json::Value>(body) else { return };

    let mut found = Vec::new();
    ids::walk_json(&json, "$", &mut found, 0);
    for (json_path, value) in found.into_iter().take(MAX_VALUES_PER_RESPONSE) {
        produced.insert(
            value,
            Producer {
                request_id: row.id.clone(),
                sequence_id: row.sequence_id,
                path: row.path.clone(),
                json_path: Some(json_path),
                kind: LinkKind::JsonValue,
            },
        );
    }
}

fn consume(
    row: &AnalysisRow,
    produced: &HashMap<String, Producer>,
    links: &mut Vec<Relationship>,
    data: &Dataset,
) {
    let mut record = |value: &str, location: String, kind: LinkKind, confidence: f32| {
        let Some(producer) = produced.get(value) else { return };
        if producer.request_id == row.id {
            return;
        }
        links.push(Relationship {
            from_request_id: producer.request_id.clone(),
            from_sequence_id: producer.sequence_id,
            from_path: producer.path.clone(),
            to_request_id: row.id.clone(),
            to_sequence_id: row.sequence_id,
            to_path: row.path.clone(),
            kind: if matches!(producer.kind, LinkKind::Cookie) { LinkKind::Cookie } else { kind },
            value_preview: super::tokens::preview(value),
            source_json_path: producer.json_path.clone(),
            target_location: location,
            confidence,
        });
    };

    for segment in row.path.split('/').filter(|s| !s.is_empty()) {
        if ids::is_candidate(segment) || ids::is_path_identity(segment) {
            record(segment, format!("path segment `{segment}`"), LinkKind::PathValue, 0.9);
        }
    }

    for p in urlutil::parse_query(row.query.as_deref()) {
        if ids::is_candidate_named(&p.value, Some(&p.name)) {
            record(&p.value, format!("query `{}`", p.name), LinkKind::QueryValue, 0.8);
        }
    }

    for c in cookies::parse_request_cookies(&row.request_headers) {
        if ids::is_candidate(&c.value) {
            record(&c.value, format!("cookie `{}`", c.name), LinkKind::Cookie, 0.95);
        }
    }

    for h in &row.request_headers {
        if headers::is_hop_by_hop(&h.name) || h.name.eq_ignore_ascii_case("cookie") {
            continue;
        }
        let value = h.value.trim();
        let bare = value.strip_prefix("Bearer ").unwrap_or(value);
        if ids::is_candidate(bare) {
            record(bare, format!("header `{}`", h.name), LinkKind::HeaderValue, 0.85);
        }
    }

    if let Some(body) = data.request_bodies.get(&row.id) {
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(body) {
            let mut found = Vec::new();
            ids::walk_json(&json, "$", &mut found, 0);
            for (path, value) in found.into_iter().take(120) {
                record(&value, format!("body {path}"), LinkKind::BodyValue, 0.75);
            }
        }
    }
}

/// Counts how many links touch each request, feeding the importance pass.
pub fn link_counts(links: &[Relationship]) -> HashMap<String, i64> {
    let mut counts: HashMap<String, i64> = HashMap::new();
    for l in links {
        *counts.entry(l.to_request_id.clone()).or_insert(0) += 1;
        *counts.entry(l.from_request_id.clone()).or_insert(0) += 1;
    }
    counts
}
