use tauri::{AppHandle, State};

use crate::error::Result;
use crate::http::curl;
use crate::models::{
    Comparison, ComparisonSide, RepeaterDraft, ReplayOptions, ReplayResult,
};
use crate::repeater::{compare, draft as draft_builder, replay};
use crate::state::AppState;
use crate::storage::{drafts, queries};

#[tauri::command]
pub fn list_drafts(state: State<'_, AppState>, session_id: String) -> Result<Vec<RepeaterDraft>> {
    drafts::list(&state.db, &session_id)
}

#[tauri::command]
pub fn get_draft(state: State<'_, AppState>, draft_id: String) -> Result<RepeaterDraft> {
    drafts::get(&state.db, &draft_id)
}

#[tauri::command]
pub fn send_to_repeater(
    state: State<'_, AppState>,
    request_id: String,
) -> Result<RepeaterDraft> {
    let draft = draft_builder::from_capture(&state.db, &request_id)?;
    drafts::upsert(&state.db, &draft)?;
    Ok(draft)
}

#[tauri::command]
pub fn new_draft(state: State<'_, AppState>, session_id: String) -> Result<RepeaterDraft> {
    let draft = draft_builder::blank(&session_id);
    drafts::upsert(&state.db, &draft)?;
    Ok(draft)
}

#[tauri::command]
pub fn save_draft(state: State<'_, AppState>, draft: RepeaterDraft) -> Result<RepeaterDraft> {
    let mut draft = draft;
    draft.updated_at = crate::models::now_millis();
    drafts::upsert(&state.db, &draft)?;
    Ok(draft)
}

#[tauri::command]
pub fn delete_draft(state: State<'_, AppState>, draft_id: String) -> Result<()> {
    drafts::delete(&state.db, &draft_id)
}

#[tauri::command]
pub async fn run_replay(
    app: AppHandle,
    state: State<'_, AppState>,
    draft: RepeaterDraft,
    options: ReplayOptions,
) -> Result<Vec<ReplayResult>> {
    let mut draft = draft;
    draft.updated_at = crate::models::now_millis();
    drafts::upsert(&state.db, &draft)?;
    let sink: std::sync::Arc<dyn crate::events::EventSink> = std::sync::Arc::new(app);
    replay::run(state.db.clone(), sink, draft, options).await
}

#[tauri::command]
pub fn list_replays(
    state: State<'_, AppState>,
    draft_id: String,
    limit: Option<i64>,
) -> Result<Vec<ReplayResult>> {
    drafts::list_replays(&state.db, &draft_id, limit.unwrap_or(200))
}

#[tauri::command]
pub fn clear_replays(state: State<'_, AppState>, draft_id: String) -> Result<()> {
    drafts::clear_replays(&state.db, &draft_id)
}

#[tauri::command]
pub fn draft_as_curl(draft: RepeaterDraft, mask_secrets: Option<bool>) -> String {
    curl::build(
        &draft.method,
        &draft_builder::effective_url(&draft),
        &draft_builder::effective_headers(&draft),
        Some(&draft.body),
        &curl::CurlOptions { mask_secrets: mask_secrets.unwrap_or(false), multiline: true },
    )
}

/// Compares two sides, each identified as `original:<request_id>` or
/// `replay:<replay_id>`, so a capture can be diffed against any replay.
#[tauri::command]
pub fn compare_responses(
    state: State<'_, AppState>,
    left: String,
    right: String,
) -> Result<Comparison> {
    let l = resolve_side(&state, &left)?;
    let r = resolve_side(&state, &right)?;
    Ok(compare::compare(l, r))
}

fn resolve_side(state: &State<'_, AppState>, spec: &str) -> Result<ComparisonSide> {
    let (kind, id) = spec.split_once(':').unwrap_or(("original", spec));
    match kind {
        "replay" => {
            let r = drafts::get_replay(&state.db, id)?;
            Ok(ComparisonSide {
                label: format!("Replay #{}", r.index),
                status: r.status,
                duration_ms: r.duration_ms,
                size: r.body_size,
                content_type: r.content_type,
                headers: r.headers,
                body: r.body.filter(|_| r.body_is_text),
            })
        }
        _ => {
            let request = queries::get_request(&state.db, id)?;
            let response = request.response.clone();
            let body = queries::load_body(&state.db, id, "response", 4 * 1024 * 1024)
                .ok()
                .and_then(|p| p.content.filter(|_| p.is_text));
            Ok(ComparisonSide {
                label: format!("Original #{:03}", request.sequence_id),
                status: response.as_ref().map(|r| r.status),
                duration_ms: response.as_ref().map(|r| r.duration_ms).unwrap_or(0),
                size: response.as_ref().map(|r| r.body.size).unwrap_or(0),
                content_type: response.as_ref().and_then(|r| r.content_type.clone()),
                headers: response.map(|r| r.headers).unwrap_or_default(),
                body,
            })
        }
    }
}
