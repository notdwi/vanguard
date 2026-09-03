use rusqlite::Connection;

use crate::error::Result;

const SCHEMA: &str = r#"
CREATE TABLE IF NOT EXISTS sessions (
    id              TEXT PRIMARY KEY,
    name            TEXT NOT NULL,
    created_at      INTEGER NOT NULL,
    updated_at      INTEGER NOT NULL,
    status          TEXT NOT NULL,
    config          TEXT NOT NULL,
    request_count   INTEGER NOT NULL DEFAULT 0,
    ignored_count   INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS requests (
    id                   TEXT PRIMARY KEY,
    session_id           TEXT NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
    sequence_id          INTEGER NOT NULL,
    timestamp            INTEGER NOT NULL,
    method               TEXT NOT NULL,
    url                  TEXT NOT NULL,
    scheme               TEXT NOT NULL,
    host                 TEXT NOT NULL,
    port                 INTEGER NOT NULL,
    path                 TEXT NOT NULL,
    query                TEXT,
    normalized_path      TEXT NOT NULL,
    protocol             TEXT NOT NULL,
    client_addr          TEXT,
    remote_ip            TEXT,
    request_headers      TEXT NOT NULL,
    request_body_ref     TEXT NOT NULL,
    request_body_blob    BLOB,
    request_size         INTEGER NOT NULL DEFAULT 0,
    request_content_type TEXT,
    has_request_body     INTEGER NOT NULL DEFAULT 0,
    has_cookies          INTEGER NOT NULL DEFAULT 0,
    has_auth             INTEGER NOT NULL DEFAULT 0,
    is_api               INTEGER NOT NULL DEFAULT 0,
    importance           TEXT NOT NULL DEFAULT 'low',
    importance_reasons   TEXT NOT NULL DEFAULT '[]',
    error                TEXT
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_requests_seq ON requests(session_id, sequence_id);
CREATE INDEX IF NOT EXISTS idx_requests_host ON requests(session_id, host);
CREATE INDEX IF NOT EXISTS idx_requests_norm ON requests(session_id, normalized_path);
CREATE INDEX IF NOT EXISTS idx_requests_importance ON requests(session_id, importance);

CREATE TABLE IF NOT EXISTS responses (
    request_id     TEXT PRIMARY KEY REFERENCES requests(id) ON DELETE CASCADE,
    session_id     TEXT NOT NULL,
    status         INTEGER NOT NULL,
    status_text    TEXT NOT NULL,
    protocol       TEXT NOT NULL,
    headers        TEXT NOT NULL,
    body_ref       TEXT NOT NULL,
    body_blob      BLOB,
    body_size      INTEGER NOT NULL DEFAULT 0,
    content_type   TEXT,
    family         TEXT NOT NULL DEFAULT 'other',
    timestamp      INTEGER NOT NULL,
    duration_ms    INTEGER NOT NULL DEFAULT 0
);

CREATE INDEX IF NOT EXISTS idx_responses_session ON responses(session_id, status);
CREATE INDEX IF NOT EXISTS idx_responses_family ON responses(session_id, family);

CREATE TABLE IF NOT EXISTS cookie_events (
    id           INTEGER PRIMARY KEY AUTOINCREMENT,
    session_id   TEXT NOT NULL,
    request_id   TEXT NOT NULL,
    sequence_id  INTEGER NOT NULL,
    direction    TEXT NOT NULL,
    name         TEXT NOT NULL,
    value        TEXT NOT NULL,
    domain       TEXT NOT NULL,
    path         TEXT NOT NULL DEFAULT '/',
    expires_at   INTEGER,
    secure       INTEGER NOT NULL DEFAULT 0,
    http_only    INTEGER NOT NULL DEFAULT 0,
    same_site    TEXT,
    timestamp    INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_cookies_session ON cookie_events(session_id, name);
CREATE INDEX IF NOT EXISTS idx_cookies_request ON cookie_events(request_id);

CREATE TABLE IF NOT EXISTS drafts (
    id                 TEXT PRIMARY KEY,
    session_id         TEXT NOT NULL,
    source_request_id  TEXT,
    source_sequence_id INTEGER,
    label              TEXT NOT NULL,
    payload            TEXT NOT NULL,
    created_at         INTEGER NOT NULL,
    updated_at         INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_drafts_session ON drafts(session_id, updated_at DESC);

CREATE TABLE IF NOT EXISTS replays (
    id           TEXT PRIMARY KEY,
    draft_id     TEXT NOT NULL,
    session_id   TEXT NOT NULL,
    idx          INTEGER NOT NULL,
    started_at   INTEGER NOT NULL,
    payload      TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_replays_draft ON replays(draft_id, started_at DESC);

CREATE TABLE IF NOT EXISTS settings (
    key   TEXT PRIMARY KEY,
    value TEXT NOT NULL
);
"#;

pub fn apply(conn: &Connection) -> Result<()> {
    conn.pragma_update(None, "journal_mode", "WAL")?;
    conn.pragma_update(None, "synchronous", "NORMAL")?;
    conn.pragma_update(None, "foreign_keys", "ON")?;
    conn.pragma_update(None, "temp_store", "MEMORY")?;
    conn.execute_batch(SCHEMA)?;
    Ok(())
}
