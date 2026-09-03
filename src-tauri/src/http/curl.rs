use crate::models::Header;

use super::headers;

pub struct CurlOptions {
    pub mask_secrets: bool,
    pub multiline: bool,
}

impl Default for CurlOptions {
    fn default() -> Self {
        Self { mask_secrets: false, multiline: true }
    }
}

pub fn build(
    method: &str,
    url: &str,
    hs: &[Header],
    body: Option<&str>,
    opts: &CurlOptions,
) -> String {
    let mut parts: Vec<String> = vec![format!("curl {}", quote(url))];

    if !method.eq_ignore_ascii_case("GET") {
        parts.push(format!("-X {method}"));
    }

    for h in hs {
        if headers::is_hop_by_hop(&h.name) && !h.name.eq_ignore_ascii_case("accept-encoding") {
            continue;
        }
        let value = if opts.mask_secrets && headers::is_sensitive(&h.name) {
            headers::mask(&h.value)
        } else {
            h.value.clone()
        };
        parts.push(format!("-H {}", quote(&format!("{}: {}", h.name, value))));
    }

    if let Some(body) = body.filter(|b| !b.is_empty()) {
        parts.push(format!("--data-raw {}", quote(body)));
    }

    if opts.multiline {
        parts.join(" \\\n  ")
    } else {
        parts.join(" ")
    }
}

/// Single-quotes for POSIX shells, escaping embedded quotes the portable way.
fn quote(value: &str) -> String {
    if value.is_empty() {
        return "''".to_string();
    }
    format!("'{}'", value.replace('\'', "'\\''"))
}
