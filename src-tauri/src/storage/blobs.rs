use std::path::{Path, PathBuf};

use crate::error::Result;
use crate::models::{BodyKind, BodyPayload, BodyRef, BodyStorage};

/// Bodies at or below this size are stored directly in SQLite.
pub const INLINE_LIMIT: usize = 256 * 1024;

pub struct StoredBody {
    pub reference: BodyRef,
    pub inline: Option<Vec<u8>>,
}

pub fn store(
    blob_root: &Path,
    session_id: &str,
    request_id: &str,
    slot: &str,
    bytes: &[u8],
    max_bytes: i64,
    content_type: Option<&str>,
) -> Result<StoredBody> {
    let size = bytes.len() as i64;
    if size == 0 {
        return Ok(StoredBody { reference: BodyRef::none(), inline: None });
    }
    if size > max_bytes {
        return Ok(StoredBody { reference: BodyRef::skipped(size), inline: None });
    }

    let is_text = looks_textual(bytes, content_type);

    if bytes.len() <= INLINE_LIMIT {
        return Ok(StoredBody {
            reference: BodyRef {
                storage: BodyStorage::Inline,
                size,
                is_text,
                truncated: false,
                path: None,
            },
            inline: Some(bytes.to_vec()),
        });
    }

    let dir = blob_root.join(session_id);
    std::fs::create_dir_all(&dir)?;
    let file = dir.join(format!("{request_id}.{slot}"));
    std::fs::write(&file, bytes)?;

    Ok(StoredBody {
        reference: BodyRef {
            storage: BodyStorage::File,
            size,
            is_text,
            truncated: false,
            path: Some(relative_path(blob_root, &file)),
        },
        inline: None,
    })
}

pub fn load(
    blob_root: &Path,
    reference: &BodyRef,
    inline: Option<Vec<u8>>,
    content_type: Option<&str>,
    max_bytes: i64,
) -> Result<BodyPayload> {
    let kind = BodyKind::from_content_type(content_type);
    let mut payload = BodyPayload {
        storage: reference.storage,
        size: reference.size,
        is_text: reference.is_text,
        truncated: false,
        content: None,
        kind: if reference.size == 0 { BodyKind::Empty } else { kind },
    };

    let bytes = match reference.storage {
        BodyStorage::Inline => inline,
        BodyStorage::File => match &reference.path {
            Some(p) => Some(std::fs::read(blob_root.join(p))?),
            None => None,
        },
        _ => None,
    };

    let Some(mut bytes) = bytes else { return Ok(payload) };

    if max_bytes > 0 && bytes.len() as i64 > max_bytes {
        bytes.truncate(max_bytes as usize);
        payload.truncated = true;
    }

    payload.content = Some(if reference.is_text {
        String::from_utf8_lossy(&bytes).into_owned()
    } else {
        use base64::Engine;
        base64::engine::general_purpose::STANDARD.encode(&bytes)
    });
    Ok(payload)
}

pub fn remove_session(blob_root: &Path, session_id: &str) {
    let _ = std::fs::remove_dir_all(blob_root.join(session_id));
}

fn relative_path(root: &Path, file: &Path) -> String {
    file.strip_prefix(root).unwrap_or(file).to_string_lossy().replace('\\', "/")
}

fn looks_textual(bytes: &[u8], content_type: Option<&str>) -> bool {
    if matches!(
        BodyKind::from_content_type(content_type),
        BodyKind::Json | BodyKind::Form | BodyKind::Text | BodyKind::Html | BodyKind::Xml
    ) {
        return true;
    }
    let sample = &bytes[..bytes.len().min(2048)];
    if sample.contains(&0) {
        return false;
    }
    std::str::from_utf8(sample).is_ok()
}

pub fn session_dir(blob_root: &Path, session_id: &str) -> PathBuf {
    blob_root.join(session_id)
}
