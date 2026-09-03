use std::collections::{HashMap, HashSet};

use crate::models::{DetectedToken, EndpointGroup, Relationship, SessionAnalysis};

use super::dataset::Dataset;
use super::{endpoints, tokens};

pub fn overview(data: &Dataset, token_list: &[DetectedToken]) -> SessionAnalysis {
    let mut domains = HashSet::new();
    let mut unique = HashSet::new();
    let mut api_endpoints = HashSet::new();
    let mut json_responses = 0i64;
    let mut post_requests = 0i64;
    let mut with_cookies = 0i64;
    let mut high = 0i64;
    let mut errors = 0i64;
    let mut total_bytes = 0i64;

    for row in &data.rows {
        domains.insert(row.host.clone());
        let key = format!("{}{}", row.host, row.normalized_path);
        unique.insert(key.clone());
        if row.is_api {
            api_endpoints.insert(key);
        }
        if row.family.as_deref() == Some("json") {
            json_responses += 1;
        }
        if row.method.eq_ignore_ascii_case("POST") {
            post_requests += 1;
        }
        if row.has_cookies {
            with_cookies += 1;
        }
        if row.importance == "high" {
            high += 1;
        }
        if row.status.map(|s| s >= 400).unwrap_or(false) {
            errors += 1;
        }
        total_bytes += row.response_size;
    }

    SessionAnalysis {
        requests: data.rows.len() as i64,
        domains: domains.len() as i64,
        api_endpoints: api_endpoints.len() as i64,
        unique_endpoints: unique.len() as i64,
        json_responses,
        post_requests,
        with_cookies,
        possible_tokens: token_list.len() as i64,
        high_importance: high,
        errors,
        total_bytes,
    }
}

pub fn endpoint_groups(data: &Dataset) -> Vec<EndpointGroup> {
    let mut groups: HashMap<String, EndpointGroup> = HashMap::new();
    let mut durations: HashMap<String, (i64, i64)> = HashMap::new();

    for row in &data.rows {
        let key = format!("{}|{}", row.host, row.normalized_path);
        let entry = groups.entry(key.clone()).or_insert_with(|| EndpointGroup {
            normalized: row.normalized_path.clone(),
            host: row.host.clone(),
            methods: Vec::new(),
            count: 0,
            is_api: row.is_api
                || endpoints::is_api_like(&row.path, row.content_type.as_deref()),
            sample_request_id: row.id.clone(),
            sequence_ids: Vec::new(),
            status_codes: Vec::new(),
            avg_duration_ms: 0,
        });

        entry.count += 1;
        if !entry.methods.contains(&row.method) {
            entry.methods.push(row.method.clone());
        }
        if entry.sequence_ids.len() < 500 {
            entry.sequence_ids.push(row.sequence_id);
        }
        if let Some(status) = row.status {
            if !entry.status_codes.contains(&status) {
                entry.status_codes.push(status);
            }
        }

        let d = durations.entry(key).or_insert((0, 0));
        d.0 += row.duration_ms;
        d.1 += 1;
    }

    let mut out: Vec<EndpointGroup> = groups
        .into_iter()
        .map(|(key, mut g)| {
            if let Some((total, n)) = durations.get(&key) {
                if *n > 0 {
                    g.avg_duration_ms = total / n;
                }
            }
            g.methods.sort();
            g.status_codes.sort_unstable();
            g
        })
        .collect();

    out.sort_by(|a, b| {
        b.is_api
            .cmp(&a.is_api)
            .then_with(|| b.count.cmp(&a.count))
            .then_with(|| a.normalized.cmp(&b.normalized))
    });
    out
}

pub fn collect_tokens(data: &Dataset) -> Vec<DetectedToken> {
    let mut all = Vec::new();
    for row in &data.rows {
        all.extend(tokens::scan(&tokens::TokenScan {
            request_id: &row.id,
            sequence_id: row.sequence_id,
            headers: &row.request_headers,
            query: row.query.as_deref(),
        }));
    }
    tokens::merge(all)
}

pub fn links_for_request<'a>(
    links: &'a [Relationship],
    request_id: &str,
) -> (Vec<&'a Relationship>, Vec<&'a Relationship>) {
    let inbound = links.iter().filter(|l| l.to_request_id == request_id).collect();
    let outbound = links.iter().filter(|l| l.from_request_id == request_id).collect();
    (inbound, outbound)
}
