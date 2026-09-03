use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionAnalysis {
    pub requests: i64,
    pub domains: i64,
    pub api_endpoints: i64,
    pub unique_endpoints: i64,
    pub json_responses: i64,
    pub post_requests: i64,
    pub with_cookies: i64,
    pub possible_tokens: i64,
    pub high_importance: i64,
    pub errors: i64,
    pub total_bytes: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndpointGroup {
    pub normalized: String,
    pub host: String,
    pub methods: Vec<String>,
    pub count: i64,
    pub is_api: bool,
    pub sample_request_id: String,
    pub sequence_ids: Vec<i64>,
    pub status_codes: Vec<u16>,
    pub avg_duration_ms: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CookieUsage {
    pub name: String,
    pub domain: String,
    pub value_preview: String,
    pub distinct_values: i64,
    pub created_by: Vec<CookieEvent>,
    pub used_by: Vec<CookieEvent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CookieEvent {
    pub request_id: String,
    pub sequence_id: i64,
    pub method: String,
    pub path: String,
    pub value_preview: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum TokenKind {
    Bearer,
    Jwt,
    ApiKey,
    Csrf,
    SessionId,
    RequestId,
    Basic,
    Unknown,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum TokenSource {
    Header,
    Cookie,
    Query,
    Body,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectedToken {
    pub kind: TokenKind,
    pub source: TokenSource,
    pub name: String,
    pub value_preview: String,
    pub value_hash: String,
    pub used_by: Vec<i64>,
    pub first_seen_request_id: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum LinkKind {
    Cookie,
    HeaderValue,
    QueryValue,
    BodyValue,
    JsonValue,
    PathValue,
}

/// A heuristic link: a value produced by one exchange reappearing in a later
/// request. Suggestive, never a guarantee.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Relationship {
    pub from_request_id: String,
    pub from_sequence_id: i64,
    pub from_path: String,
    pub to_request_id: String,
    pub to_sequence_id: i64,
    pub to_path: String,
    pub kind: LinkKind,
    pub value_preview: String,
    pub source_json_path: Option<String>,
    pub target_location: String,
    pub confidence: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowNode {
    pub id: String,
    pub label: String,
    pub host: String,
    pub method: String,
    pub count: i64,
    pub importance: String,
    pub depth: i32,
    pub sample_request_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowEdge {
    pub from: String,
    pub to: String,
    pub kind: LinkKind,
    pub label: String,
    pub weight: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowGraph {
    pub nodes: Vec<FlowNode>,
    pub edges: Vec<FlowEdge>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestAnalysis {
    pub importance: String,
    pub reasons: Vec<String>,
    pub normalized_endpoint: String,
    pub is_api: bool,
    pub detected_ids: Vec<DetectedId>,
    pub tokens: Vec<DetectedToken>,
    pub inbound: Vec<Relationship>,
    pub outbound: Vec<Relationship>,
    pub repeat_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectedId {
    pub value: String,
    pub location: String,
    pub kind: String,
}
