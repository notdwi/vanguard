pub mod dataset;
pub mod endpoints;
pub mod graph;
pub mod ids;
pub mod importance;
pub mod relations;
pub mod summary;
pub mod tokens;

use serde::Serialize;

use crate::error::Result;
use crate::models::{
    CookieUsage, DetectedToken, EndpointGroup, FlowGraph, Relationship, SessionAnalysis,
};
use crate::storage::{cookies as cookie_store, Db};

#[derive(Debug, Clone, Serialize)]
pub struct AnalysisBundle {
    pub session_id: String,
    pub generated_at: i64,
    pub overview: SessionAnalysis,
    pub endpoints: Vec<EndpointGroup>,
    pub tokens: Vec<DetectedToken>,
    pub relationships: Vec<Relationship>,
    pub graph: FlowGraph,
    pub cookies: Vec<CookieUsage>,
    pub truncated: bool,
}

/// One pass over a session: endpoints, tokens, links, graph and cookies.
/// Importance is rewritten afterwards so the timeline reflects what the
/// relationship pass learned.
pub fn run(db: &Db, session_id: &str) -> Result<AnalysisBundle> {
    let data = dataset::load(db, session_id)?;
    let truncated = data.rows.len() as i64 >= dataset::ROW_LIMIT;

    let token_list = summary::collect_tokens(&data);
    let relationships = relations::detect(&data);
    let graph = graph::build(&data, &relationships);
    let endpoint_groups = summary::endpoint_groups(&data);
    let overview = summary::overview(&data, &token_list);
    let cookies = cookie_store::usage(db, session_id)?;

    refine_importance(db, &data, &relationships)?;

    Ok(AnalysisBundle {
        session_id: session_id.to_string(),
        generated_at: crate::models::now_millis(),
        overview,
        endpoints: endpoint_groups,
        tokens: token_list,
        relationships,
        graph,
        cookies,
        truncated,
    })
}

fn refine_importance(
    db: &Db,
    data: &dataset::Dataset,
    relationships: &[Relationship],
) -> Result<()> {
    let link_counts = relations::link_counts(relationships);
    let endpoint_counts = data.endpoint_counts();

    for row in &data.rows {
        let key = format!("{}{}", row.host, row.normalized_path);
        let verdict = importance::score(&importance::Signals {
            method: &row.method,
            host: &row.host,
            path: &row.path,
            query: row.query.as_deref(),
            request_content_type: None,
            response_content_type: row.content_type.as_deref(),
            status: row.status,
            has_auth: row.has_auth,
            has_cookies: row.has_cookies,
            has_request_body: row.has_request_body,
            response_size: row.response_size,
            has_path_id: row.normalized_path.contains(':'),
            relationship_count: link_counts.get(&row.id).copied().unwrap_or(0),
            repeat_count: endpoint_counts.get(&key).copied().unwrap_or(1),
        });

        if verdict.importance.as_str() != row.importance {
            crate::storage::requests::set_importance(
                db,
                &row.id,
                verdict.importance,
                &verdict.reasons,
            )?;
        }
    }
    Ok(())
}
