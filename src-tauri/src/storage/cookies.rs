use rusqlite::Row;

use crate::error::Result;
use crate::models::{CookieEvent, CookieUsage};

use super::Db;

pub struct RawCookieEvent {
    pub request_id: String,
    pub sequence_id: i64,
    pub direction: String,
    pub name: String,
    pub value: String,
    pub domain: String,
    pub method: String,
    pub path: String,
}

pub fn all_events(db: &Db, session_id: &str) -> Result<Vec<RawCookieEvent>> {
    db.with(|c| {
        let mut stmt = c.prepare(
            "SELECT e.request_id, e.sequence_id, e.direction, e.name, e.value, e.domain,
                    r.method, r.path
             FROM cookie_events e JOIN requests r ON r.id = e.request_id
             WHERE e.session_id = ?1
             ORDER BY e.sequence_id ASC, e.id ASC",
        )?;
        let rows = stmt
            .query_map([session_id], row_to_event)?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        Ok(rows)
    })
}

pub fn for_request(db: &Db, request_id: &str) -> Result<Vec<RawCookieEvent>> {
    db.with(|c| {
        let mut stmt = c.prepare(
            "SELECT e.request_id, e.sequence_id, e.direction, e.name, e.value, e.domain,
                    r.method, r.path
             FROM cookie_events e JOIN requests r ON r.id = e.request_id
             WHERE e.request_id = ?1 ORDER BY e.id ASC",
        )?;
        let rows = stmt
            .query_map([request_id], row_to_event)?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        Ok(rows)
    })
}

/// Groups every cookie name in a session into who set it and who sent it back.
pub fn usage(db: &Db, session_id: &str) -> Result<Vec<CookieUsage>> {
    let events = all_events(db, session_id)?;
    let mut grouped: Vec<CookieUsage> = Vec::new();

    for e in events {
        let key = (e.name.clone(), e.domain.clone());
        let idx = grouped
            .iter()
            .position(|g| g.name == key.0 && g.domain == key.1)
            .unwrap_or_else(|| {
                grouped.push(CookieUsage {
                    name: key.0.clone(),
                    domain: key.1.clone(),
                    value_preview: preview(&e.value),
                    distinct_values: 0,
                    created_by: Vec::new(),
                    used_by: Vec::new(),
                });
                grouped.len() - 1
            });

        let entry = CookieEvent {
            request_id: e.request_id,
            sequence_id: e.sequence_id,
            method: e.method,
            path: e.path,
            value_preview: preview(&e.value),
        };
        if e.direction == "set" {
            grouped[idx].created_by.push(entry);
        } else {
            grouped[idx].used_by.push(entry);
        }
    }

    for g in grouped.iter_mut() {
        let mut values: Vec<&str> = g
            .created_by
            .iter()
            .chain(g.used_by.iter())
            .map(|e| e.value_preview.as_str())
            .collect();
        values.sort_unstable();
        values.dedup();
        g.distinct_values = values.len() as i64;
    }

    grouped.sort_by(|a, b| {
        let ac = a.created_by.len() + a.used_by.len();
        let bc = b.created_by.len() + b.used_by.len();
        bc.cmp(&ac).then_with(|| a.name.cmp(&b.name))
    });
    Ok(grouped)
}

pub fn preview(value: &str) -> String {
    let trimmed = value.trim();
    if trimmed.chars().count() <= 42 {
        return trimmed.to_string();
    }
    let head: String = trimmed.chars().take(38).collect();
    format!("{head}...")
}

fn row_to_event(r: &Row) -> rusqlite::Result<RawCookieEvent> {
    Ok(RawCookieEvent {
        request_id: r.get(0)?,
        sequence_id: r.get(1)?,
        direction: r.get(2)?,
        name: r.get(3)?,
        value: r.get(4)?,
        domain: r.get(5)?,
        method: r.get(6)?,
        path: r.get(7)?,
    })
}
