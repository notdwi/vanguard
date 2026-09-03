use rusqlite::{params, Connection, OptionalExtension};

use crate::error::{AppError, Result};
use crate::models::{CaptureConfig, CaptureState, Session, SessionSummary};

use super::{blobs, Db};

pub fn create(db: &Db, name: &str, config: &CaptureConfig) -> Result<Session> {
    let now = crate::models::now_millis();
    let session = Session {
        id: crate::models::new_id("ses"),
        name: name.trim().to_string(),
        created_at: now,
        updated_at: now,
        status: CaptureState::Idle,
        config: config.clone(),
        request_count: 0,
        ignored_count: 0,
    };
    let cfg = serde_json::to_string(&session.config)?;
    db.with(|c| {
        c.execute(
            "INSERT INTO sessions(id, name, created_at, updated_at, status, config, request_count, ignored_count)
             VALUES(?1, ?2, ?3, ?4, ?5, ?6, 0, 0)",
            params![session.id, session.name, now, now, session.status.as_str(), cfg],
        )?;
        Ok(())
    })?;
    Ok(session)
}

pub fn get(db: &Db, id: &str) -> Result<Session> {
    db.with(|c| {
        let mut stmt = c.prepare(
            "SELECT id, name, created_at, updated_at, status, config, request_count, ignored_count
             FROM sessions WHERE id = ?1",
        )?;
        stmt.query_row([id], row_to_session)
            .optional()?
            .ok_or_else(|| AppError::NotFound(format!("session {id}")))
    })
}

pub fn list(db: &Db) -> Result<Vec<SessionSummary>> {
    db.with(|c| {
        let mut stmt = c.prepare(
            "SELECT id, name, created_at, updated_at, status, request_count, ignored_count
             FROM sessions ORDER BY created_at DESC",
        )?;
        let rows = stmt
            .query_map([], |r| {
                Ok(SessionSummary {
                    id: r.get(0)?,
                    name: r.get(1)?,
                    created_at: r.get(2)?,
                    updated_at: r.get(3)?,
                    status: CaptureState::parse(&r.get::<_, String>(4)?),
                    request_count: r.get(5)?,
                    ignored_count: r.get(6)?,
                    domains: Vec::new(),
                })
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;

        let mut out = Vec::with_capacity(rows.len());
        for mut s in rows {
            s.domains = top_domains(c, &s.id)?;
            out.push(s);
        }
        Ok(out)
    })
}

pub fn rename(db: &Db, id: &str, name: &str) -> Result<()> {
    db.with(|c| {
        c.execute(
            "UPDATE sessions SET name = ?2, updated_at = ?3 WHERE id = ?1",
            params![id, name.trim(), crate::models::now_millis()],
        )?;
        Ok(())
    })
}

pub fn set_status(db: &Db, id: &str, status: CaptureState) -> Result<()> {
    db.with(|c| {
        c.execute(
            "UPDATE sessions SET status = ?2, updated_at = ?3 WHERE id = ?1",
            params![id, status.as_str(), crate::models::now_millis()],
        )?;
        Ok(())
    })
}

pub fn set_config(db: &Db, id: &str, config: &CaptureConfig) -> Result<()> {
    let cfg = serde_json::to_string(config)?;
    db.with(|c| {
        c.execute(
            "UPDATE sessions SET config = ?2, updated_at = ?3 WHERE id = ?1",
            params![id, cfg, crate::models::now_millis()],
        )?;
        Ok(())
    })
}

pub fn bump_counters(db: &Db, id: &str, captured: i64, ignored: i64) -> Result<()> {
    db.with(|c| {
        c.execute(
            "UPDATE sessions SET request_count = request_count + ?2,
                                 ignored_count = ignored_count + ?3,
                                 updated_at = ?4
             WHERE id = ?1",
            params![id, captured, ignored, crate::models::now_millis()],
        )?;
        Ok(())
    })
}

pub fn delete(db: &Db, id: &str) -> Result<()> {
    db.with(|c| {
        c.execute("DELETE FROM responses WHERE session_id = ?1", [id])?;
        c.execute("DELETE FROM cookie_events WHERE session_id = ?1", [id])?;
        c.execute("DELETE FROM replays WHERE session_id = ?1", [id])?;
        c.execute("DELETE FROM drafts WHERE session_id = ?1", [id])?;
        c.execute("DELETE FROM requests WHERE session_id = ?1", [id])?;
        c.execute("DELETE FROM sessions WHERE id = ?1", [id])?;
        Ok(())
    })?;
    blobs::remove_session(&db.blob_root(), id);
    Ok(())
}

pub fn clear_requests(db: &Db, id: &str) -> Result<()> {
    db.with(|c| {
        c.execute("DELETE FROM responses WHERE session_id = ?1", [id])?;
        c.execute("DELETE FROM cookie_events WHERE session_id = ?1", [id])?;
        c.execute("DELETE FROM requests WHERE session_id = ?1", [id])?;
        c.execute(
            "UPDATE sessions SET request_count = 0, ignored_count = 0, updated_at = ?2 WHERE id = ?1",
            params![id, crate::models::now_millis()],
        )?;
        Ok(())
    })?;
    blobs::remove_session(&db.blob_root(), id);
    Ok(())
}

pub fn next_sequence(db: &Db, id: &str) -> Result<i64> {
    db.with(|c| {
        let max: Option<i64> = c
            .query_row("SELECT MAX(sequence_id) FROM requests WHERE session_id = ?1", [id], |r| r.get(0))
            .optional()?
            .flatten();
        Ok(max.unwrap_or(0))
    })
}

fn top_domains(c: &Connection, session_id: &str) -> Result<Vec<String>> {
    let mut stmt = c.prepare(
        "SELECT host, COUNT(*) n FROM requests WHERE session_id = ?1
         GROUP BY host ORDER BY n DESC LIMIT 4",
    )?;
    let rows = stmt
        .query_map([session_id], |r| r.get::<_, String>(0))?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(rows)
}

fn row_to_session(r: &rusqlite::Row) -> rusqlite::Result<Session> {
    let cfg: String = r.get(5)?;
    Ok(Session {
        id: r.get(0)?,
        name: r.get(1)?,
        created_at: r.get(2)?,
        updated_at: r.get(3)?,
        status: CaptureState::parse(&r.get::<_, String>(4)?),
        config: serde_json::from_str(&cfg).unwrap_or_default(),
        request_count: r.get(6)?,
        ignored_count: r.get(7)?,
    })
}
