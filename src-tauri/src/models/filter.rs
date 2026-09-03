use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "kebab-case")]
pub enum ScopeMode {
    #[default]
    AllTraffic,
    /// Host must match a listed domain exactly.
    ExactHost,
    /// Host must equal a listed domain or be a subdomain of it.
    DomainAndSubdomains,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct CaptureConfig {
    pub mode: ScopeMode,
    /// Domains or wildcard patterns such as `*.site.com`.
    pub include_domains: Vec<String>,
    pub exclude_domains: Vec<String>,
    /// Glob-ish path patterns, e.g. `/api/*`.
    pub include_paths: Vec<String>,
    pub exclude_paths: Vec<String>,
    pub include_methods: Vec<String>,
    pub exclude_methods: Vec<String>,
    pub include_content_types: Vec<String>,
    pub exclude_content_types: Vec<String>,
    /// Bodies above this many bytes are not persisted.
    pub max_body_bytes: i64,
    pub capture_request_bodies: bool,
    pub capture_response_bodies: bool,
}

impl Default for CaptureConfig {
    fn default() -> Self {
        Self {
            mode: ScopeMode::AllTraffic,
            include_domains: Vec::new(),
            exclude_domains: Vec::new(),
            include_paths: Vec::new(),
            exclude_paths: Vec::new(),
            include_methods: Vec::new(),
            exclude_methods: Vec::new(),
            include_content_types: Vec::new(),
            exclude_content_types: Vec::new(),
            max_body_bytes: 16 * 1024 * 1024,
            capture_request_bodies: true,
            capture_response_bodies: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct TimelineQuery {
    pub session_id: String,
    pub search: Option<String>,
    pub methods: Vec<String>,
    /// Status classes: 2, 3, 4, 5. Empty means all.
    pub status_classes: Vec<u16>,
    pub families: Vec<String>,
    pub hosts: Vec<String>,
    pub importance: Vec<String>,
    pub only_api: bool,
    pub only_errors: bool,
    pub only_json: bool,
    pub only_with_cookies: bool,
    pub only_with_body: bool,
    pub only_authenticated: bool,
    pub search_bodies: bool,
    pub limit: i64,
    pub offset: i64,
}
