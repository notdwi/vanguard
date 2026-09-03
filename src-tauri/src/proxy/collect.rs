use http_body_util::BodyExt;
use hudsucker::Body;
use hudsucker::hyper::Response;

/// Reads a body fully so it can be recorded, then hands back an equivalent
/// body for the wire. Anything past `max_bytes` is forwarded but not kept.
pub async fn buffer(body: Body, max_bytes: i64) -> (Vec<u8>, Body) {
    let collected = match body.collect().await {
        Ok(c) => c.to_bytes(),
        Err(e) => {
            tracing::debug!("could not read body: {e}");
            return (Vec::new(), Body::empty());
        }
    };

    let replay = Body::from(collected.clone());
    if max_bytes > 0 && collected.len() as i64 > max_bytes {
        return (Vec::new(), replay);
    }
    (collected.to_vec(), replay)
}

const SUPPORTED: &[&str] = &["gzip", "x-gzip", "deflate", "br", "zstd", "identity"];

/// Decodes compressed responses so the inspector shows readable text. Bodies
/// with an encoding hudsucker cannot handle are passed through untouched
/// rather than dropped.
pub fn decode_if_supported(res: Response<Body>) -> Response<Body> {
    let encodings: Vec<String> = res
        .headers()
        .get_all(http::header::CONTENT_ENCODING)
        .iter()
        .filter_map(|v| v.to_str().ok())
        .flat_map(|v| v.split(',').map(|p| p.trim().to_ascii_lowercase()))
        .filter(|p| !p.is_empty())
        .collect();

    if encodings.is_empty() {
        return res;
    }
    if !encodings.iter().all(|e| SUPPORTED.contains(&e.as_str())) {
        tracing::debug!("leaving body encoded: unsupported content-encoding {encodings:?}");
        return res;
    }

    match hudsucker::decode_response(res) {
        Ok(decoded) => decoded,
        Err(e) => {
            tracing::warn!("response decode failed after support check: {e}");
            Response::builder()
                .status(hudsucker::hyper::StatusCode::BAD_GATEWAY)
                .body(Body::from("vanguard: could not decode the upstream response"))
                .expect("static response is valid")
        }
    }
}
