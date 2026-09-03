use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HarFile {
    pub log: HarLog,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HarLog {
    pub version: String,
    pub creator: HarCreator,
    #[serde(default)]
    pub entries: Vec<HarEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HarCreator {
    pub name: String,
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HarEntry {
    pub started_date_time: String,
    pub time: f64,
    pub request: HarRequest,
    pub response: HarResponse,
    #[serde(default)]
    pub cache: serde_json::Value,
    pub timings: HarTimings,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub server_ip_address: Option<String>,
    #[serde(rename = "_sequenceId", default, skip_serializing_if = "Option::is_none")]
    pub sequence_id: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HarRequest {
    pub method: String,
    pub url: String,
    pub http_version: String,
    #[serde(default)]
    pub headers: Vec<HarNameValue>,
    #[serde(default)]
    pub query_string: Vec<HarNameValue>,
    #[serde(default)]
    pub cookies: Vec<HarNameValue>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub post_data: Option<HarPostData>,
    #[serde(default)]
    pub headers_size: i64,
    #[serde(default)]
    pub body_size: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HarResponse {
    pub status: u16,
    pub status_text: String,
    pub http_version: String,
    #[serde(default)]
    pub headers: Vec<HarNameValue>,
    #[serde(default)]
    pub cookies: Vec<HarNameValue>,
    pub content: HarContent,
    #[serde(default)]
    pub redirect_url: String,
    #[serde(default)]
    pub headers_size: i64,
    #[serde(default)]
    pub body_size: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HarNameValue {
    pub name: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HarPostData {
    pub mime_type: String,
    #[serde(default)]
    pub text: String,
    #[serde(default)]
    pub params: Vec<HarNameValue>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HarContent {
    pub size: i64,
    pub mime_type: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub encoding: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HarTimings {
    pub send: f64,
    pub wait: f64,
    pub receive: f64,
}

impl HarTimings {
    pub fn from_total(total_ms: i64) -> Self {
        Self { send: 0.0, wait: total_ms as f64, receive: 0.0 }
    }
}

pub fn creator() -> HarCreator {
    HarCreator { name: "Vanguard".into(), version: env!("CARGO_PKG_VERSION").into() }
}

pub fn iso_time(millis: i64) -> String {
    chrono::DateTime::from_timestamp_millis(millis)
        .unwrap_or_else(chrono::Utc::now)
        .to_rfc3339_opts(chrono::SecondsFormat::Millis, true)
}

pub fn parse_iso(value: &str) -> i64 {
    chrono::DateTime::parse_from_rfc3339(value)
        .map(|d| d.timestamp_millis())
        .unwrap_or_else(|_| crate::models::now_millis())
}

pub fn to_pairs(items: &[crate::models::Header]) -> Vec<HarNameValue> {
    items
        .iter()
        .map(|h| HarNameValue { name: h.name.clone(), value: h.value.clone() })
        .collect()
}

pub fn from_pairs(items: &[HarNameValue]) -> Vec<crate::models::Header> {
    items
        .iter()
        .map(|h| crate::models::Header { name: h.name.clone(), value: h.value.clone() })
        .collect()
}
