use std::collections::HashMap;

use crate::error::Result;
use crate::models::Header;
use crate::storage::{queries, Db};

/// Everything the analysis passes need, loaded once per run.
pub struct AnalysisRow {
    pub id: String,
    pub sequence_id: i64,
    pub method: String,
    pub host: String,
    pub path: String,
    pub normalized_path: String,
    pub query: Option<String>,
    pub is_api: bool,
    pub has_auth: bool,
    pub has_cookies: bool,
    pub has_request_body: bool,
    pub importance: String,
    pub status: Option<u16>,
    pub duration_ms: i64,
    pub family: Option<String>,
    pub content_type: Option<String>,
    pub response_size: i64,
    pub request_headers: Vec<Header>,
    pub response_headers: Vec<Header>,
}

pub struct Dataset {
    pub rows: Vec<AnalysisRow>,
    pub request_bodies: HashMap<String, String>,
    pub response_bodies: HashMap<String, String>,
}

/// Caps keep a 100k-request session responsive; the newest traffic wins.
pub const ROW_LIMIT: i64 = 20_000;
pub const BODY_LIMIT: i64 = 4_000;

pub fn load(db: &Db, session_id: &str) -> Result<Dataset> {
    let rows = load_rows(db, session_id, ROW_LIMIT)?;
    let request_bodies = queries::iter_text_bodies(db, session_id, "request", BODY_LIMIT)?
        .into_iter()
        .map(|(id, _, text)| (id, text))
        .collect();
    let response_bodies = queries::iter_text_bodies(db, session_id, "response", BODY_LIMIT)?
        .into_iter()
        .map(|(id, _, text)| (id, text))
        .collect();
    Ok(Dataset { rows, request_bodies, response_bodies })
}

pub fn load_rows(db: &Db, session_id: &str, limit: i64) -> Result<Vec<AnalysisRow>> {
    db.with(|c| {
        let mut stmt = c.prepare(
            "SELECT r.id, r.sequence_id, r.method, r.host, r.path, r.normalized_path, r.query,
                    r.is_api, r.has_auth, r.has_cookies, r.has_request_body, r.importance,
                    r.request_headers,
                    s.status, s.duration_ms, s.family, s.content_type, s.body_size, s.headers
             FROM requests r LEFT JOIN responses s ON s.request_id = r.id
             WHERE r.session_id = ?1
             ORDER BY r.sequence_id ASC
             LIMIT ?2",
        )?;
        let rows = stmt
            .query_map(rusqlite::params![session_id, limit], |r| {
                let req_headers: String = r.get(12)?;
                let res_headers: Option<String> = r.get(18)?;
                Ok(AnalysisRow {
                    id: r.get(0)?,
                    sequence_id: r.get(1)?,
                    method: r.get(2)?,
                    host: r.get(3)?,
                    path: r.get(4)?,
                    normalized_path: r.get(5)?,
                    query: r.get(6)?,
                    is_api: r.get::<_, i64>(7)? != 0,
                    has_auth: r.get::<_, i64>(8)? != 0,
                    has_cookies: r.get::<_, i64>(9)? != 0,
                    has_request_body: r.get::<_, i64>(10)? != 0,
                    importance: r.get(11)?,
                    status: r.get::<_, Option<i64>>(13)?.map(|v| v as u16),
                    duration_ms: r.get::<_, Option<i64>>(14)?.unwrap_or(0),
                    family: r.get(15)?,
                    content_type: r.get(16)?,
                    response_size: r.get::<_, Option<i64>>(17)?.unwrap_or(0),
                    request_headers: serde_json::from_str(&req_headers).unwrap_or_default(),
                    response_headers: res_headers
                        .and_then(|h| serde_json::from_str(&h).ok())
                        .unwrap_or_default(),
                })
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        Ok(rows)
    })
}

impl Dataset {
    pub fn endpoint_counts(&self) -> HashMap<String, i64> {
        let mut counts = HashMap::new();
        for row in &self.rows {
            let key = format!("{}{}", row.host, row.normalized_path);
            *counts.entry(key).or_insert(0) += 1;
        }
        counts
    }

    pub fn repeat_count(&self, row: &AnalysisRow) -> i64 {
        self.rows
            .iter()
            .filter(|r| r.host == row.host && r.normalized_path == row.normalized_path)
            .count() as i64
    }

    pub fn by_id(&self, id: &str) -> Option<&AnalysisRow> {
        self.rows.iter().find(|r| r.id == id)
    }
}
