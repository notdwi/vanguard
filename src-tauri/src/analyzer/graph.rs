use std::collections::{HashMap, HashSet};

use crate::models::{FlowEdge, FlowGraph, FlowNode, LinkKind, Relationship};

use super::dataset::Dataset;

/// Collapses per-request links into an endpoint-level flow, then assigns each
/// node a depth so the UI can lay it out top to bottom.
pub fn build(data: &Dataset, links: &[Relationship]) -> FlowGraph {
    let mut node_of: HashMap<String, String> = HashMap::new();
    let mut nodes: HashMap<String, FlowNode> = HashMap::new();

    for row in &data.rows {
        let key = format!("{} {}{}", row.method, row.host, row.normalized_path);
        node_of.insert(row.id.clone(), key.clone());
        nodes
            .entry(key.clone())
            .and_modify(|n| {
                n.count += 1;
                if rank(&row.importance) > rank(&n.importance) {
                    n.importance = row.importance.clone();
                }
            })
            .or_insert(FlowNode {
                id: key,
                label: row.normalized_path.clone(),
                host: row.host.clone(),
                method: row.method.clone(),
                count: 1,
                importance: row.importance.clone(),
                depth: 0,
                sample_request_id: row.id.clone(),
            });
    }

    let mut edges: HashMap<(String, String, String), FlowEdge> = HashMap::new();
    for link in links {
        let (Some(from), Some(to)) =
            (node_of.get(&link.from_request_id), node_of.get(&link.to_request_id))
        else {
            continue;
        };
        if from == to {
            continue;
        }
        let label = edge_label(link);
        let key = (from.clone(), to.clone(), label.clone());
        edges
            .entry(key)
            .and_modify(|e| e.weight += 1)
            .or_insert(FlowEdge {
                from: from.clone(),
                to: to.clone(),
                kind: link.kind,
                label,
                weight: 1,
            });
    }

    let mut edge_list: Vec<FlowEdge> = edges.into_values().collect();
    edge_list.sort_by(|a, b| b.weight.cmp(&a.weight));

    let connected: HashSet<String> = edge_list
        .iter()
        .flat_map(|e| [e.from.clone(), e.to.clone()])
        .collect();

    let mut node_list: Vec<FlowNode> = nodes
        .into_values()
        .filter(|n| connected.contains(&n.id) || n.importance == "high")
        .collect();

    assign_depth(&mut node_list, &edge_list);
    node_list.sort_by(|a, b| a.depth.cmp(&b.depth).then_with(|| b.count.cmp(&a.count)));

    let kept: HashSet<String> = node_list.iter().map(|n| n.id.clone()).collect();
    edge_list.retain(|e| kept.contains(&e.from) && kept.contains(&e.to));

    FlowGraph { nodes: node_list, edges: edge_list }
}

fn edge_label(link: &Relationship) -> String {
    match link.kind {
        LinkKind::Cookie => "cookie".into(),
        LinkKind::HeaderValue => "header".into(),
        LinkKind::QueryValue => "query".into(),
        LinkKind::BodyValue => "body".into(),
        LinkKind::JsonValue => "json value".into(),
        LinkKind::PathValue => "path id".into(),
    }
}

fn rank(importance: &str) -> u8 {
    match importance {
        "high" => 3,
        "medium" => 2,
        _ => 1,
    }
}

/// Longest-path depth with a visit cap, so cycles cannot spin forever.
fn assign_depth(nodes: &mut [FlowNode], edges: &[FlowEdge]) {
    let incoming: HashMap<&str, Vec<&str>> = edges.iter().fold(HashMap::new(), |mut acc, e| {
        acc.entry(e.to.as_str()).or_default().push(e.from.as_str());
        acc
    });

    let ids: Vec<String> = nodes.iter().map(|n| n.id.clone()).collect();
    let mut depths: HashMap<String, i32> = ids.iter().map(|id| (id.clone(), 0)).collect();

    for _ in 0..8 {
        let mut changed = false;
        for id in &ids {
            let Some(parents) = incoming.get(id.as_str()) else { continue };
            let best = parents
                .iter()
                .filter_map(|p| depths.get(*p).copied())
                .max()
                .unwrap_or(-1);
            let candidate = best + 1;
            if candidate > *depths.get(id).unwrap_or(&0) {
                depths.insert(id.clone(), candidate);
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }

    for node in nodes.iter_mut() {
        node.depth = depths.get(&node.id).copied().unwrap_or(0);
    }
}
