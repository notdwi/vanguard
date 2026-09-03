use std::path::{Path, PathBuf};
use std::sync::Arc;

use parking_lot::Mutex;
use rusqlite::Connection;

use crate::error::Result;

use super::schema;

#[derive(Clone)]
pub struct Db {
    conn: Arc<Mutex<Connection>>,
    root: PathBuf,
}

impl Db {
    pub fn open(root: &Path) -> Result<Self> {
        std::fs::create_dir_all(root)?;
        std::fs::create_dir_all(root.join("blobs"))?;
        let conn = Connection::open(root.join("vanguard.db"))?;
        schema::apply(&conn)?;
        Ok(Self { conn: Arc::new(Mutex::new(conn)), root: root.to_path_buf() })
    }

    pub fn with<T>(&self, f: impl FnOnce(&Connection) -> Result<T>) -> Result<T> {
        let guard = self.conn.lock();
        f(&guard)
    }

    pub fn with_tx<T>(&self, f: impl FnOnce(&rusqlite::Transaction) -> Result<T>) -> Result<T> {
        let mut guard = self.conn.lock();
        let tx = guard.transaction()?;
        let out = f(&tx)?;
        tx.commit()?;
        Ok(out)
    }

    pub fn blob_root(&self) -> PathBuf {
        self.root.join("blobs")
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn get_setting(&self, key: &str) -> Result<Option<String>> {
        self.with(|c| {
            let mut stmt = c.prepare("SELECT value FROM settings WHERE key = ?1")?;
            let mut rows = stmt.query([key])?;
            Ok(match rows.next()? {
                Some(row) => Some(row.get::<_, String>(0)?),
                None => None,
            })
        })
    }

    pub fn set_setting(&self, key: &str, value: &str) -> Result<()> {
        self.with(|c| {
            c.execute(
                "INSERT INTO settings(key, value) VALUES(?1, ?2)
                 ON CONFLICT(key) DO UPDATE SET value = excluded.value",
                (key, value),
            )?;
            Ok(())
        })
    }
}
