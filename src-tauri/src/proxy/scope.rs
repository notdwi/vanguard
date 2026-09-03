use crate::models::{CaptureConfig, ScopeMode};

/// Decides whether an exchange belongs in the timeline. Everything outside the
/// scope is counted as ignored, never silently dropped.
pub struct Scope {
    config: CaptureConfig,
}

impl Scope {
    pub fn new(config: CaptureConfig) -> Self {
        Self { config }
    }

    pub fn config(&self) -> &CaptureConfig {
        &self.config
    }

    pub fn allows(&self, host: &str, path: &str, method: &str) -> bool {
        self.host_allowed(host) && self.path_allowed(path) && self.method_allowed(method)
    }

    pub fn content_type_allowed(&self, content_type: Option<&str>) -> bool {
        let c = &self.config;
        let ct = content_type.unwrap_or("").to_ascii_lowercase();
        if c.exclude_content_types.iter().any(|p| ct.contains(&p.to_ascii_lowercase())) {
            return false;
        }
        if c.include_content_types.is_empty() {
            return true;
        }
        c.include_content_types.iter().any(|p| ct.contains(&p.to_ascii_lowercase()))
    }

    fn host_allowed(&self, host: &str) -> bool {
        let host = host.trim_end_matches('.').to_ascii_lowercase();
        let c = &self.config;

        if c.exclude_domains.iter().any(|p| matches_domain(p, &host, ScopeMode::DomainAndSubdomains))
        {
            return false;
        }

        match c.mode {
            ScopeMode::AllTraffic => true,
            mode => {
                if c.include_domains.is_empty() {
                    return true;
                }
                c.include_domains.iter().any(|p| matches_domain(p, &host, mode))
            }
        }
    }

    fn path_allowed(&self, path: &str) -> bool {
        let c = &self.config;
        if c.exclude_paths.iter().any(|p| matches_glob(p, path)) {
            return false;
        }
        if c.include_paths.is_empty() {
            return true;
        }
        c.include_paths.iter().any(|p| matches_glob(p, path))
    }

    fn method_allowed(&self, method: &str) -> bool {
        let c = &self.config;
        let m = method.to_ascii_uppercase();
        if c.exclude_methods.iter().any(|x| x.eq_ignore_ascii_case(&m)) {
            return false;
        }
        if c.include_methods.is_empty() {
            return true;
        }
        c.include_methods.iter().any(|x| x.eq_ignore_ascii_case(&m))
    }
}

/// A pattern may be a bare domain or use a leading `*.` wildcard. In
/// DomainAndSubdomains mode a bare domain also covers its subdomains.
pub fn matches_domain(pattern: &str, host: &str, mode: ScopeMode) -> bool {
    let pattern = pattern.trim().trim_end_matches('.').to_ascii_lowercase();
    if pattern.is_empty() {
        return false;
    }

    if let Some(base) = pattern.strip_prefix("*.") {
        return host == base || host.ends_with(&format!(".{base}"));
    }
    if pattern == "*" {
        return true;
    }
    if host == pattern {
        return true;
    }
    matches!(mode, ScopeMode::DomainAndSubdomains) && host.ends_with(&format!(".{pattern}"))
}

/// Supports `*` (any run of characters) anywhere in the pattern.
pub fn matches_glob(pattern: &str, value: &str) -> bool {
    let pattern = pattern.trim();
    if pattern.is_empty() {
        return false;
    }
    if !pattern.contains('*') {
        return value == pattern || value.starts_with(pattern);
    }

    let segments: Vec<&str> = pattern.split('*').collect();
    let mut cursor = 0usize;

    for (i, seg) in segments.iter().enumerate() {
        if seg.is_empty() {
            continue;
        }
        if i == 0 {
            if !value[cursor..].starts_with(seg) {
                return false;
            }
            cursor += seg.len();
            continue;
        }
        match value[cursor..].find(seg) {
            Some(pos) => cursor += pos + seg.len(),
            None => return false,
        }
    }

    match segments.last() {
        Some(last) if !last.is_empty() => value.ends_with(last),
        _ => true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact_host_rejects_subdomains() {
        assert!(matches_domain("api.site.com", "api.site.com", ScopeMode::ExactHost));
        assert!(!matches_domain("site.com", "api.site.com", ScopeMode::ExactHost));
    }

    #[test]
    fn subdomain_mode_and_wildcards() {
        assert!(matches_domain("site.com", "api.site.com", ScopeMode::DomainAndSubdomains));
        assert!(matches_domain("*.site.com", "cdn.site.com", ScopeMode::ExactHost));
        assert!(!matches_domain("*.site.com", "othersite.com", ScopeMode::ExactHost));
        assert!(!matches_domain("site.com", "notsite.com", ScopeMode::DomainAndSubdomains));
    }

    #[test]
    fn glob_paths() {
        assert!(matches_glob("/api/*", "/api/search"));
        assert!(matches_glob("*/tracking/*", "/x/tracking/y"));
        assert!(!matches_glob("/api/*", "/static/app.js"));
    }
}
