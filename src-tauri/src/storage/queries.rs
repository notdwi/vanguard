use rusqlite::{OptionalExtension, Row};

use crate::error::{AppError, Result};
use crate::models::{
    BodyPayload, BodyRef, CapturedRequest, CapturedResponse, Header, Importance, TimelinePage,
    TimelineQuery, TimelineRow,
};

use super::{blobs, filters::build_where, Db};

pub fn timeline(db: &Db, q: &TimelineQuery) -> Result<TimelinePage> {
    let (clause, args) = build_where(q);
    let limit = if q.limit <= 0 { 500 } else { q.limit.min(5000) };
    let offset = q.offset.max(0);

    db.with(|c| {
        let count_sql = format!(
            "SELECT COUNT(*) FROM requests r LEFT JOIN responses s ON s.request_id = r.id WHERE {clause}"
        );
        let total: i64 = c.prepare(&count_sql)?.query_row(
            rusqlite::params_from_iter(args.iter()),
            |r| r.get(0),
        )?;

        let sql = format!(
            "SELECT r.id, r.sequence_id, r.timestamp, r.method, r.scheme, r.host, r.path, r.query,
                    s.status, s.duration_ms, COALESCE(s.body_size, 0), s.family,
                    r.importance, r.error
             FROM requests r LEFT JOIN responses s ON s.request_id = r.id
             WHERE {clause}
             ORDER BY r.sequence_id ASC
             LIMIT {limit} OFFSET {offset}"
        );
        let mut stmt = c.prepare(&sql)?;
        let rows = stmt
            .query_map(rusqlite::params_from_iter(args.iter()), row_to_timeline)?
            .collect::<rusqlite::Result<Vec<_>>>()?;

        Ok(TimelinePage { rows, total, offset })
    })
}

pub fn hosts(db: &Db, session_id: &str) -> Result<Vec<(String, i64)>> {
    db.with(|c| {
        let mut stmt = c.prepare(
            "SELECT host, COUNT(*) n FROM requests WHERE session_id = ?1
             GROUP BY host ORDER BY n DESC",
        )?;
        let rows = stmt
            .query_map([session_id], |r| Ok((r.get(0)?, r.get(1)?)))?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        Ok(rows)
    })
}

pub fn get_request(db: &Db, id: &str) -> Result<CapturedRequest> {
    db.with(|c| {
        let mut stmt = c.prepare(
            "SELECT r.id, r.session_id, r.sequence_id, r.timestamp, r.method, r.url, r.scheme,
                    r.host, r.port, r.path, r.query, r.normalized_path, r.protocol, r.client_addr,
                    r.remote_ip, r.request_headers, r.request_body_ref, r.request_size,
                    r.request_content_type, r.importance, r.importance_reasons, r.error,
                    s.status, s.status_text, s.protocol, s.headers, s.body_ref, s.content_type,
                    s.family, s.timestamp, s.duration_ms
             FROM requests r LEFT JOIN responses s ON s.request_id = r.id
             WHERE r.id = ?1",
        )?;
        stmt.query_row([id], row_to_captured)
            .optional()?
            .ok_or_else(|| AppError::NotFound(format!("request {id}")))
    })
}

pub fn request_ids_for_sequences(db: &Db, session_id: &str, seqs: &[i64]) -> Result<Vec<String>> {
    if seqs.is_empty() {
        return Ok(Vec::new());
    }
    let list = seqs.iter().map(|s| s.to_string()).collect::<Vec<_>>().join(",");
    db.with(|c| {
        let sql = format!(
            "SELECT id FROM requests WHERE session_id = ?1 AND sequence_id IN ({list})
             ORDER BY sequence_id"
        );
        let mut stmt = c.prepare(&sql)?;
        let rows = stmt
            .query_map([session_id], |r| r.get::<_, String>(0))?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        Ok(rows)
    })
}

pub fn load_body(db: &Db, request_id: &str, side: &str, max_bytes: i64) -> Result<BodyPayload> {
    let (ref_col, blob_col, ct_col, table, key) = match side {
        "request" => (
            "request_body_ref",
            "request_body_blob",
            "request_content_type",
            "requests",
            "id",
        ),
        _ => ("body_ref", "body_blob", "content_type", "responses", "request_id"),
    };

    let loaded = db.with(|c| {
        let sql = format!("SELECT {ref_col}, {blob_col}, {ct_col} FROM {table} WHERE {key} = ?1");
        let mut stmt = c.prepare(&sql)?;
        let row = stmt
            .query_row([request_id], |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, Option<Vec<u8>>>(1)?,
                    r.get::<_, Option<String>>(2)?,
                ))
            })
            .optional()?;
        row.ok_or_else(|| AppError::NotFound(format!("{side} body for {request_id}")))
    })?;

    let (body_ref, inline, content_type) = loaded;
    let reference: BodyRef = serde_json::from_str(&body_ref)?;
    blobs::load(&db.blob_root(), &reference, inline, content_type.as_deref(), max_bytes)
}

/// Decoded text bodies for a session, used by the analyzer and body search.
pub fn iter_text_bodies(
    db: &Db,
    session_id: &str,
    side: &str,
    limit: i64,
) -> Result<Vec<(String, i64, String)>> {
    let (table, key, ref_col, blob_col) = match side {
        "request" => ("requests", "id", "request_body_ref", "request_body_blob"),
        _ => ("responses", "request_id", "body_ref", "body_blob"),
    };
    let blob_root = db.blob_root();

    db.with(|c| {
        let sql = format!(
            "SELECT t.{key}, r.sequence_id, t.{ref_col}, t.{blob_col}
             FROM {table} t JOIN requests r ON r.id = t.{key}
             WHERE t.session_id = ?1 ORDER BY r.sequence_id ASC LIMIT ?2"
        );
        let mut stmt = c.prepare(&sql)?;
        let rows = stmt.query_map(rusqlite::params![session_id, limit], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, i64>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, Option<Vec<u8>>>(3)?,
            ))
        })?;

        let mut out = Vec::new();
        for row in rows {
            let (id, seq, refs, blob) = row?;
            let Ok(reference) = serde_json::from_str::<BodyRef>(&refs) else { continue };
            if !reference.is_text || !reference.is_loadable() {
                continue;
            }
            let loaded = blobs::load(&blob_root, &reference, blob, None, 2 * 1024 * 1024);
            if let Ok(payload) = loaded {
                if let Some(text) = payload.content {
                    out.push((id, seq, text));
                }
            }
        }
        Ok(out)
    })
}

fn row_to_timeline(r: &Row) -> rusqlite::Result<TimelineRow> {
    Ok(TimelineRow {
        id: r.get(0)?,
        sequence_id: r.get(1)?,
        timestamp: r.get(2)?,
        method: r.get(3)?,
        scheme: r.get(4)?,
        host: r.get(5)?,
        path: r.get(6)?,
        query: r.get(7)?,
        status: r.get::<_, Option<i64>>(8)?.map(|v| v as u16),
        duration_ms: r.get(9)?,
        response_size: r.get(10)?,
        family: r.get(11)?,
        importance: Importance::parse(&r.get::<_, String>(12)?),
        has_error: r.get::<_, Option<String>>(13)?.is_some(),
    })
}

fn row_to_captured(r: &Row) -> rusqlite::Result<CapturedRequest> {
    let req_headers: String = r.get(15)?;
    let req_body_ref: String = r.get(16)?;
    let reasons: String = r.get(20)?;

    let response = match r.get::<_, Option<i64>>(22)? {
        Some(status) => {
            let headers: String = r.get(25)?;
            let body_ref: String = r.get(26)?;
            Some(CapturedResponse {
                status: status as u16,
                status_text: r.get(23)?,
                protocol: r.get(24)?,
                headers: serde_json::from_str(&headers).unwrap_or_default(),
                body: serde_json::from_str(&body_ref).unwrap_or_else(|_| BodyRef::none()),
                content_type: r.get(27)?,
                family: r.get::<_, Option<String>>(28)?.unwrap_or_else(|| "other".into()),
                timestamp: r.get(29)?,
                duration_ms: r.get::<_, Option<i64>>(30)?.unwrap_or(0),
            })
        }
        None => None,
    };

    Ok(CapturedRequest {
        id: r.get(0)?,
        session_id: r.get(1)?,
        sequence_id: r.get(2)?,
        timestamp: r.get(3)?,
        method: r.get(4)?,
        url: r.get(5)?,
        scheme: r.get(6)?,
        host: r.get(7)?,
        port: r.get::<_, i64>(8)? as u16,
        path: r.get(9)?,
        query: r.get(10)?,
        normalized_path: r.get(11)?,
        protocol: r.get(12)?,
        client_addr: r.get(13)?,
        remote_ip: r.get(14)?,
        request_headers: serde_json::from_str::<Vec<Header>>(&req_headers).unwrap_or_default(),
        request_body: serde_json::from_str(&req_body_ref).unwrap_or_else(|_| BodyRef::none()),
        request_size: r.get(17)?,
        request_content_type: r.get(18)?,
        response,
        error: r.get(21)?,
        importance: Importance::parse(&r.get::<_, String>(19)?),
        importance_reasons: serde_json::from_str(&reasons).unwrap_or_default(),
    })
}
