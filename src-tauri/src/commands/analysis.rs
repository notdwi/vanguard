use serde::Serialize;
use tauri::{AppHandle, State};

use crate::analyzer::{self, AnalysisBundle};
use crate::error::Result;
use crate::events;
use crate::models::{DetectedId, RequestAnalysis, Relationship};
use crate::state::AppState;
use crate::storage::queries;

#[tauri::command]
pub fn analyse_session(
    app: AppHandle,
    state: State<'_, AppState>,
    session_id: String,
    refresh: Option<bool>,
) -> Result<AnalysisBundle> {
    if !refresh.unwrap_or(false) {
        if let Some(cached) = state.cached_analysis(&session_id) {
            return Ok(cached);
        }
    }
    let bundle = analyzer::run(&state.db, &session_id)?;
    state.cache_analysis(bundle.clone());
    events::emit(&app, events::ANALYSIS_UPDATED, session_id);
    Ok(bundle)
}

#[tauri::command]
pub fn analyse_request(
    state: State<'_, AppState>,
    session_id: String,
    request_id: String,
) -> Result<RequestAnalysis> {
    let bundle = match state.cached_analysis(&session_id) {
        Some(b) => b,
        None => {
            let b = analyzer::run(&state.db, &session_id)?;
            state.cache_analysis(b.clone());
            b
        }
    };

    let request = queries::get_request(&state.db, &request_id)?;
    let body = queries::load_body(&state.db, &request_id, "request", 512 * 1024)
        .ok()
        .and_then(|p| p.content.filter(|_| p.is_text));

    let detected_ids: Vec<DetectedId> = analyzer::ids::from_request(
        &request.path,
        request.query.as_deref(),
        body.as_deref(),
    );

    let tokens = analyzer::tokens::merge(analyzer::tokens::scan(&analyzer::tokens::TokenScan {
        request_id: &request.id,
        sequence_id: request.sequence_id,
        headers: &request.request_headers,
        query: request.query.as_deref(),
    }));

    let inbound: Vec<Relationship> = bundle
        .relationships
        .iter()
        .filter(|l| l.to_request_id == request_id)
        .cloned()
        .collect();
    let outbound: Vec<Relationship> = bundle
        .relationships
        .iter()
        .filter(|l| l.from_request_id == request_id)
        .cloned()
        .collect();

    let key = format!("{}{}", request.host, request.normalized_path);
    let repeat_count = bundle
        .endpoints
        .iter()
        .find(|e| format!("{}{}", e.host, e.normalized) == key)
        .map(|e| e.count)
        .unwrap_or(1);

    Ok(RequestAnalysis {
        importance: request.importance.as_str().to_string(),
        reasons: request.importance_reasons,
        normalized_endpoint: request.normalized_path,
        is_api: analyzer::endpoints::is_api_like(
            &request.path,
            request.response.as_ref().and_then(|r| r.content_type.as_deref()),
        ),
        detected_ids,
        tokens,
        inbound,
        outbound,
        repeat_count,
    })
}

#[derive(Serialize)]
pub struct EndpointRequests {
    pub request_ids: Vec<String>,
}

#[tauri::command]
pub fn endpoint_requests(
    state: State<'_, AppState>,
    session_id: String,
    sequence_ids: Vec<i64>,
) -> Result<EndpointRequests> {
    Ok(EndpointRequests {
        request_ids: queries::request_ids_for_sequences(&state.db, &session_id, &sequence_ids)?,
    })
}
