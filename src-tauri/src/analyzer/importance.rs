use crate::models::Importance;

use super::endpoints;

pub struct Signals<'a> {
    pub method: &'a str,
    pub host: &'a str,
    pub path: &'a str,
    pub query: Option<&'a str>,
    pub request_content_type: Option<&'a str>,
    pub response_content_type: Option<&'a str>,
    pub status: Option<u16>,
    pub has_auth: bool,
    pub has_cookies: bool,
    pub has_request_body: bool,
    pub response_size: i64,
    pub has_path_id: bool,
    pub relationship_count: i64,
    pub repeat_count: i64,
}

pub struct Verdict {
    pub importance: Importance,
    pub reasons: Vec<String>,
    pub score: i32,
}

/// Deterministic scoring. No model, no training data: just the signals a
/// crawler engineer would look at, made explicit so the UI can show its work.
pub fn score(s: &Signals) -> Verdict {
    let mut score = 0i32;
    let mut reasons: Vec<String> = Vec::new();

    let is_api = endpoints::is_api_like(s.path, s.response_content_type);
    let is_asset = endpoints::is_static_asset(&s.path.to_ascii_lowercase());
    let json = s
        .response_content_type
        .map(|c| c.to_ascii_lowercase().contains("json"))
        .unwrap_or(false);

    if is_api {
        score += 30;
        reasons.push("Looks like an API endpoint".into());
    }
    if json {
        score += 25;
        reasons.push("Returns JSON".into());
    }
    if !s.method.eq_ignore_ascii_case("GET") {
        score += 20;
        reasons.push(format!("{} request", s.method.to_ascii_uppercase()));
    }
    if s.has_request_body {
        score += 10;
        reasons.push("Carries a request body".into());
    }
    if s.query.map(|q| !q.is_empty()).unwrap_or(false) {
        score += 8;
        reasons.push("Uses query parameters".into());
    }
    if s.has_auth {
        score += 18;
        reasons.push("Sends an authorization header".into());
    }
    if s.has_cookies {
        score += 8;
        reasons.push("Sends cookies".into());
    }
    if s.has_path_id {
        score += 12;
        reasons.push("Identifier in the path".into());
    }
    if s.relationship_count > 0 {
        score += 15;
        reasons.push(format!("Linked to {} other request(s)", s.relationship_count));
    }
    if s.repeat_count > 2 {
        score += 8;
        reasons.push(format!("Endpoint repeats {} times", s.repeat_count));
    }
    if matches!(s.status, Some(st) if (400..600).contains(&st)) {
        score += 10;
        reasons.push("Returned an error status".into());
    }

    if is_asset {
        score -= 45;
        reasons.push("Static asset".into());
    }
    if endpoints::is_noise_host(s.host) {
        score -= 40;
        reasons.push("Known analytics or ad host".into());
    }
    if matches!(
        s.response_content_type.map(family),
        Some("image") | Some("font") | Some("style") | Some("media")
    ) {
        score -= 30;
        reasons.push("Non-data content type".into());
    }
    if s.response_size == 0 && s.status.map(|st| st < 300).unwrap_or(false) {
        score -= 5;
    }

    let importance = if score >= 55 {
        Importance::High
    } else if score >= 25 {
        Importance::Medium
    } else {
        Importance::Low
    };

    Verdict { importance, reasons, score }
}

fn family(ct: &str) -> &'static str {
    let lower = ct.to_ascii_lowercase();
    if lower.starts_with("image/") {
        "image"
    } else if lower.contains("font") || lower.contains("woff") {
        "font"
    } else if lower.contains("css") {
        "style"
    } else if lower.starts_with("audio/") || lower.starts_with("video/") {
        "media"
    } else {
        "other"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base<'a>() -> Signals<'a> {
        Signals {
            method: "GET",
            host: "site.com",
            path: "/",
            query: None,
            request_content_type: None,
            response_content_type: None,
            status: Some(200),
            has_auth: false,
            has_cookies: false,
            has_request_body: false,
            response_size: 100,
            has_path_id: false,
            relationship_count: 0,
            repeat_count: 1,
        }
    }

    #[test]
    fn json_api_post_is_high() {
        let mut s = base();
        s.method = "POST";
        s.path = "/api/search";
        s.response_content_type = Some("application/json");
        s.has_request_body = true;
        assert_eq!(score(&s).importance, Importance::High);
    }

    #[test]
    fn static_asset_is_low() {
        let mut s = base();
        s.path = "/assets/logo.svg";
        s.response_content_type = Some("image/svg+xml");
        assert_eq!(score(&s).importance, Importance::Low);
    }
}
