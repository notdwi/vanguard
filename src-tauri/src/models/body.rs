use serde::{Deserialize, Serialize};

/// Where a captured body lives. Small bodies stay in SQLite, larger ones move
/// to a file on disk, and anything past the capture cap is never persisted.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum BodyStorage {
    None,
    Inline,
    File,
    Skipped,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BodyRef {
    pub storage: BodyStorage,
    pub size: i64,
    pub is_text: bool,
    pub truncated: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
}

impl BodyRef {
    pub fn none() -> Self {
        Self { storage: BodyStorage::None, size: 0, is_text: false, truncated: false, path: None }
    }

    pub fn skipped(size: i64) -> Self {
        Self { storage: BodyStorage::Skipped, size, is_text: false, truncated: false, path: None }
    }

    pub fn is_loadable(&self) -> bool {
        matches!(self.storage, BodyStorage::Inline | BodyStorage::File)
    }
}

/// A body payload resolved for the UI.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BodyPayload {
    pub storage: BodyStorage,
    pub size: i64,
    pub is_text: bool,
    pub truncated: bool,
    /// UTF-8 text when `is_text`, base64 otherwise.
    pub content: Option<String>,
    pub kind: BodyKind,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum BodyKind {
    Empty,
    Json,
    Form,
    Multipart,
    Text,
    Html,
    Xml,
    Binary,
}

impl BodyKind {
    pub fn from_content_type(ct: Option<&str>) -> Self {
        let Some(ct) = ct else { return BodyKind::Binary };
        let ct = ct.split(';').next().unwrap_or("").trim().to_ascii_lowercase();
        match ct.as_str() {
            "" => BodyKind::Empty,
            "application/json" | "text/json" => BodyKind::Json,
            "application/x-www-form-urlencoded" => BodyKind::Form,
            "text/html" | "application/xhtml+xml" => BodyKind::Html,
            "text/xml" | "application/xml" => BodyKind::Xml,
            _ if ct.starts_with("multipart/") => BodyKind::Multipart,
            _ if ct.ends_with("+json") => BodyKind::Json,
            _ if ct.ends_with("+xml") => BodyKind::Xml,
            _ if ct.starts_with("text/") => BodyKind::Text,
            _ if ct.contains("javascript") || ct.contains("ecmascript") => BodyKind::Text,
            _ => BodyKind::Binary,
        }
    }
}

/// Broad response family used by the type filter in the UI.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ContentFamily {
    Json,
    Html,
    Script,
    Style,
    Image,
    Font,
    Media,
    Document,
    Other,
}

impl ContentFamily {
    pub fn from_content_type(ct: Option<&str>) -> Self {
        let Some(ct) = ct else { return ContentFamily::Other };
        let ct = ct.split(';').next().unwrap_or("").trim().to_ascii_lowercase();
        if ct.contains("json") {
            ContentFamily::Json
        } else if ct.contains("html") {
            ContentFamily::Html
        } else if ct.contains("javascript") || ct.contains("ecmascript") {
            ContentFamily::Script
        } else if ct.contains("css") {
            ContentFamily::Style
        } else if ct.starts_with("image/") {
            ContentFamily::Image
        } else if ct.starts_with("font/") || ct.contains("woff") || ct.contains("ttf") {
            ContentFamily::Font
        } else if ct.starts_with("audio/") || ct.starts_with("video/") {
            ContentFamily::Media
        } else if ct.contains("pdf") || ct.starts_with("text/") || ct.contains("xml") {
            ContentFamily::Document
        } else {
            ContentFamily::Other
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            ContentFamily::Json => "json",
            ContentFamily::Html => "html",
            ContentFamily::Script => "script",
            ContentFamily::Style => "style",
            ContentFamily::Image => "image",
            ContentFamily::Font => "font",
            ContentFamily::Media => "media",
            ContentFamily::Document => "document",
            ContentFamily::Other => "other",
        }
    }
}
