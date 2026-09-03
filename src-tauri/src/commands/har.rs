use serde::Serialize;
use tauri::{AppHandle, State};

use crate::capture::har_import;
use crate::error::Result;
use crate::events;
use crate::http::{har, url as urlutil};
use crate::state::AppState;
use crate::storage::{queries, sessions};

#[derive(Serialize)]
pub struct ImportReport {
    pub session_id: String,
    pub imported: i64,
    pub skipped: i64,
}

#[tauri::command]
pub fn import_har(
    app: AppHandle,
    state: State<'_, AppState>,
    path: String,
    name: Option<String>,
) -> Result<ImportReport> {
    let raw = std::fs::read_to_string(&path)?;
    let file: har::HarFile = serde_json::from_str(&raw)?;
    let label = name.unwrap_or_else(|| {
        std::path::Path::new(&path)
            .file_stem()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_else(|| "Imported HAR".into())
    });

    let outcome = har_import::import(&state.db, &label, &file)?;
    state.invalidate_analysis();
    events::sessions_changed(&app);

    Ok(ImportReport {
        session_id: outcome.session_id,
        imported: outcome.imported,
        skipped: outcome.skipped,
    })
}

#[tauri::command]
pub fn export_har(
    state: State<'_, AppState>,
    session_id: String,
    path: String,
) -> Result<String> {
    let session = sessions::get(&state.db, &session_id)?;
    let rows = crate::analyzer::dataset::load_rows(&state.db, &session_id, 100_000)?;
    let mut entries = Vec::with_capacity(rows.len());

    for row in rows {
        let request = queries::get_request(&state.db, &row.id)?;
        let req_body = queries::load_body(&state.db, &row.id, "request", 0).ok();
        let res_body = queries::load_body(&state.db, &row.id, "response", 0).ok();

        let post_data = req_body.as_ref().filter(|b| b.size > 0).map(|b| har::HarPostData {
            mime_type: request.request_content_type.clone().unwrap_or_default(),
            text: b.content.clone().unwrap_or_default(),
            params: Vec::new(),
        });

        let (content_text, encoding) = match &res_body {
            Some(b) if b.is_text => (b.content.clone(), None),
            Some(b) => (b.content.clone(), Some("base64".to_string())),
            None => (None, None),
        };

        let response = request.response.clone();
        entries.push(har::HarEntry {
            started_date_time: har::iso_time(request.timestamp),
            time: response.as_ref().map(|r| r.duration_ms as f64).unwrap_or(0.0),
            request: har::HarRequest {
                method: request.method.clone(),
                url: request.url.clone(),
                http_version: request.protocol.clone(),
                headers: har::to_pairs(&request.request_headers),
                query_string: urlutil::parse_query(request.query.as_deref())
                    .into_iter()
                    .map(|p| har::HarNameValue { name: p.name, value: p.value })
                    .collect(),
                cookies: Vec::new(),
                post_data,
                headers_size: -1,
                body_size: request.request_size,
            },
            response: har::HarResponse {
                status: response.as_ref().map(|r| r.status).unwrap_or(0),
                status_text: response
                    .as_ref()
                    .map(|r| r.status_text.clone())
                    .unwrap_or_default(),
                http_version: response
                    .as_ref()
                    .map(|r| r.protocol.clone())
                    .unwrap_or_else(|| "HTTP/1.1".into()),
                headers: response.as_ref().map(|r| har::to_pairs(&r.headers)).unwrap_or_default(),
                cookies: Vec::new(),
                content: har::HarContent {
                    size: res_body.as_ref().map(|b| b.size).unwrap_or(0),
                    mime_type: response
                        .as_ref()
                        .and_then(|r| r.content_type.clone())
                        .unwrap_or_default(),
                    text: content_text,
                    encoding,
                },
                redirect_url: String::new(),
                headers_size: -1,
                body_size: res_body.as_ref().map(|b| b.size).unwrap_or(0),
            },
            cache: serde_json::json!({}),
            timings: har::HarTimings::from_total(
                response.as_ref().map(|r| r.duration_ms).unwrap_or(0),
            ),
            server_ip_address: request.remote_ip.clone(),
            sequence_id: Some(request.sequence_id),
        });
    }

    let file = har::HarFile {
        log: har::HarLog { version: "1.2".into(), creator: har::creator(), entries },
    };
    std::fs::write(&path, serde_json::to_string_pretty(&file)?)?;
    let _ = session;
    Ok(path)
}
