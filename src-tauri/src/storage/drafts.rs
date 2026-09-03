use rusqlite::{params, OptionalExtension};

use crate::error::{AppError, Result};
use crate::models::{now_millis, RepeaterDraft, ReplayResult};

use super::Db;

pub fn upsert(db: &Db, draft: &RepeaterDraft) -> Result<()> {
    let payload = serde_json::to_string(draft)?;
    db.with(|c| {
        c.execute(
            "INSERT INTO drafts(id, session_id, source_request_id, source_sequence_id, label,
                                payload, created_at, updated_at)
             VALUES(?1,?2,?3,?4,?5,?6,?7,?8)
             ON CONFLICT(id) DO UPDATE SET
                label = excluded.label,
                payload = excluded.payload,
                updated_at = excluded.updated_at",
            params![
                draft.id,
                draft.session_id,
                draft.source_request_id,
                draft.source_sequence_id,
                draft.label,
                payload,
                draft.created_at,
                draft.updated_at,
            ],
        )?;
        Ok(())
    })
}

pub fn get(db: &Db, id: &str) -> Result<RepeaterDraft> {
    db.with(|c| {
        let payload: Option<String> = c
            .query_row("SELECT payload FROM drafts WHERE id = ?1", [id], |r| r.get(0))
            .optional()?;
        let payload = payload.ok_or_else(|| AppError::NotFound(format!("draft {id}")))?;
        Ok(serde_json::from_str(&payload)?)
    })
}

pub fn list(db: &Db, session_id: &str) -> Result<Vec<RepeaterDraft>> {
    db.with(|c| {
        let mut stmt = c.prepare(
            "SELECT payload FROM drafts WHERE session_id = ?1 ORDER BY updated_at DESC",
        )?;
        let rows = stmt
            .query_map([session_id], |r| r.get::<_, String>(0))?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        Ok(rows.iter().filter_map(|p| serde_json::from_str(p).ok()).collect())
    })
}

pub fn delete(db: &Db, id: &str) -> Result<()> {
    db.with(|c| {
        c.execute("DELETE FROM replays WHERE draft_id = ?1", [id])?;
        c.execute("DELETE FROM drafts WHERE id = ?1", [id])?;
        Ok(())
    })
}

pub fn insert_replay(db: &Db, result: &ReplayResult) -> Result<()> {
    let payload = serde_json::to_string(result)?;
    db.with(|c| {
        c.execute(
            "INSERT INTO replays(id, draft_id, session_id, idx, started_at, payload)
             VALUES(?1,?2,?3,?4,?5,?6)",
            params![
                result.id,
                result.draft_id,
                result.session_id,
                result.index,
                result.started_at,
                payload
            ],
        )?;
        c.execute(
            "UPDATE drafts SET updated_at = ?2 WHERE id = ?1",
            params![result.draft_id, now_millis()],
        )?;
        Ok(())
    })
}

pub fn list_replays(db: &Db, draft_id: &str, limit: i64) -> Result<Vec<ReplayResult>> {
    db.with(|c| {
        let mut stmt = c.prepare(
            "SELECT payload FROM replays WHERE draft_id = ?1 ORDER BY started_at DESC, idx DESC
             LIMIT ?2",
        )?;
        let rows = stmt
            .query_map(params![draft_id, limit], |r| r.get::<_, String>(0))?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        Ok(rows.iter().filter_map(|p| serde_json::from_str(p).ok()).collect())
    })
}

pub fn get_replay(db: &Db, id: &str) -> Result<ReplayResult> {
    db.with(|c| {
        let payload: Option<String> = c
            .query_row("SELECT payload FROM replays WHERE id = ?1", [id], |r| r.get(0))
            .optional()?;
        let payload = payload.ok_or_else(|| AppError::NotFound(format!("replay {id}")))?;
        Ok(serde_json::from_str(&payload)?)
    })
}

pub fn clear_replays(db: &Db, draft_id: &str) -> Result<()> {
    db.with(|c| {
        c.execute("DELETE FROM replays WHERE draft_id = ?1", [draft_id])?;
        Ok(())
    })
}
