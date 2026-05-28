// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Generic SPARQL/QLever HTTP utilities shared by all apps.
//!
//! `QLever` CSV export URL format:
//!   `https://qlever.dev/api/wikidata?query=<encoded>&action=csv_export`

#[cfg(not(target_arch = "wasm32"))]
use std::io::{Seek, Write};
use std::sync::OnceLock;
#[cfg(not(target_arch = "wasm32"))]
use std::time::Duration;

pub type ResponseBody = bytes::Bytes;

/// Default `QLever` endpoint for Wikidata (used by lotus-explorer).
pub const QLEVER_WIKIDATA: &str = "https://qlever.dev/api/wikidata";
const MAX_HTTP_ATTEMPTS: u32 = 2;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResponseFormat {
    Csv,
    SparqlJson,
    Turtle,
    NTriples,
}

impl ResponseFormat {
    const fn accept(self) -> &'static str {
        match self {
            Self::Csv => "text/csv",
            Self::SparqlJson => "application/sparql-results+json",
            Self::Turtle => "text/turtle",
            Self::NTriples => "application/n-triples",
        }
    }

    const fn action(self) -> Option<&'static str> {
        match self {
            Self::Csv => Some("csv_export"),
            Self::SparqlJson => Some("sparql_json_export"),
            Self::Turtle => Some("turtle_export"),
            Self::NTriples => None,
        }
    }
}

// ── Error type ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub enum FetchError {
    Network(String),
    Http(u16, String),
    Parse(String),
    Empty,
}

impl std::fmt::Display for FetchError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Network(e) => write!(f, "Network error: {e}"),
            Self::Http(s, msg) => write!(f, "HTTP {s}: {msg}"),
            Self::Parse(e) => write!(f, "Parse error: {e}"),
            Self::Empty => write!(f, "Query returned no results"),
        }
    }
}

// ── HTTP execution ────────────────────────────────────────────────────────────

/// Execute a SPARQL query against `endpoint` and return the raw CSV body.
///
/// Up to two attempts,
/// with `Accept: text/csv` so the endpoint can honor content negotiation
/// even when the `action=csv_export` form parameter is ignored. Retries
/// transient network / 5xx errors; 4xx errors fail fast.
///
/// # Errors
/// Returns [`FetchError`] when the request fails, the upstream responds with an
/// HTTP error, or the body is empty / invalid UTF-8.
pub async fn execute_query(sparql: &str, endpoint: &str) -> Result<String, FetchError> {
    execute_sparql_with_format(sparql, endpoint, ResponseFormat::Csv).await
}

/// Execute a SPARQL query and return raw response bytes.
///
/// Useful for memory-sensitive paths where callers parse CSV directly from bytes
/// without first materializing an intermediate UTF-8 `String`.
///
/// # Errors
/// Returns [`FetchError`] for network/HTTP failures or empty upstream payloads.
pub async fn execute_sparql_bytes(sparql: &str, endpoint: &str) -> Result<Vec<u8>, FetchError> {
    let body = execute_sparql_body(sparql, endpoint).await?;
    Ok(body.to_vec())
}

/// Execute a SPARQL query and return the raw response body.
///
/// This avoids an extra `Bytes -> Vec<u8>` copy for callers that can parse from
/// borrowed byte slices or readers.
///
/// # Errors
/// Returns [`FetchError`] for network/HTTP failures or empty upstream payloads.
pub async fn execute_sparql_body(sparql: &str, endpoint: &str) -> Result<ResponseBody, FetchError> {
    execute_sparql_with_format_body(sparql, endpoint, ResponseFormat::Csv).await
}

#[cfg(not(target_arch = "wasm32"))]
/// Execute a SPARQL query and stream the response into a temporary file.
///
/// # Errors
/// Returns [`FetchError`] when request/streaming/tempfile I/O fails, or when
/// the upstream response is empty / an HTTP error.
pub async fn execute_sparql_tempfile(
    sparql: &str,
    endpoint: &str,
) -> Result<tempfile::NamedTempFile, FetchError> {
    execute_sparql_with_format_tempfile(sparql, endpoint, ResponseFormat::Csv).await
}

/// Execute a SPARQL query and decode response bytes as UTF-8 text.
///
/// # Errors
/// Returns [`FetchError`] for transport/HTTP failures, empty responses, or
/// invalid UTF-8 payloads.
pub async fn execute_sparql_with_format(
    sparql: &str,
    endpoint: &str,
    format: ResponseFormat,
) -> Result<String, FetchError> {
    let bytes = execute_sparql_with_format_bytes(sparql, endpoint, format).await?;
    String::from_utf8(bytes).map_err(|e| FetchError::Parse(e.to_string()))
}

/// Execute a SPARQL query and return response bytes in a chosen representation.
///
/// # Errors
/// Returns [`FetchError`] for transport/HTTP failures or empty responses.
pub async fn execute_sparql_with_format_bytes(
    sparql: &str,
    endpoint: &str,
    format: ResponseFormat,
) -> Result<Vec<u8>, FetchError> {
    let body = execute_sparql_with_format_body(sparql, endpoint, format).await?;
    Ok(body.to_vec())
}

/// Execute a SPARQL query and return the raw response body for a representation.
///
/// # Errors
/// Returns [`FetchError`] for transport/HTTP failures or empty responses.
pub async fn execute_sparql_with_format_body(
    sparql: &str,
    endpoint: &str,
    format: ResponseFormat,
) -> Result<ResponseBody, FetchError> {
    log::debug!("SPARQL POST endpoint: {endpoint}");

    let client = http_client()?;
    let mut last_err: Option<FetchError> = None;

    for attempt in 0..MAX_HTTP_ATTEMPTS {
        // `Accept` and `Content-Type: application/x-www-form-urlencoded` are
        // both CORS-safelisted, so the request stays simple (no preflight).
        // Do not add `User-Agent` or other custom headers — browsers refuse to
        // let WASM set them, which causes QLever to reject the preflight.
        let result = client
            .post(endpoint)
            .header("Accept", format.accept())
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(build_sparql_form_body(sparql, format))
            .send()
            .await;

        match result {
            Ok(resp) => {
                let status = resp.status();
                let code = status.as_u16();
                if status.is_success() {
                    return match resp.bytes().await {
                        Ok(bytes) if bytes.is_empty() => Err(FetchError::Empty),
                        Ok(bytes) => {
                            // HTML gateway pages are text, so inspect a lossy preview.
                            let preview = String::from_utf8_lossy(&bytes);
                            if looks_like_gateway_error(&preview) {
                                let err = FetchError::Http(
                                    502,
                                    "upstream gateway error (HTML payload)".into(),
                                );
                                if attempt + 1 < MAX_HTTP_ATTEMPTS {
                                    last_err = Some(err);
                                    continue;
                                }
                                return Err(err);
                            }
                            Ok(bytes)
                        }
                        Err(e) => {
                            let err = FetchError::Network(e.to_string());
                            if attempt + 1 < MAX_HTTP_ATTEMPTS {
                                last_err = Some(err);
                                continue;
                            }
                            Err(err)
                        }
                    };
                }

                let body = resp.text().await.unwrap_or_default();
                let detail = compact_http_error_text(&body);
                log::error!("event=sparql_http_error status={code} detail={detail}");
                // Fail fast on client errors (4xx); retry on server errors (5xx).
                if (400..500).contains(&code) {
                    return Err(FetchError::Http(code, detail));
                }
                last_err = Some(FetchError::Http(code, detail));
            }
            Err(e) => {
                last_err = Some(FetchError::Network(e.to_string()));
            }
        }
    }

    Err(last_err.unwrap_or_else(|| FetchError::Network("unknown error".into())))
}

/// Execute a SPARQL query and stream the selected representation into a tempfile.
///
/// # Errors
/// Returns [`FetchError`] when request/streaming/tempfile I/O fails, or when
/// the upstream response is empty / an HTTP error.
#[cfg(not(target_arch = "wasm32"))]
pub async fn execute_sparql_with_format_tempfile(
    sparql: &str,
    endpoint: &str,
    format: ResponseFormat,
) -> Result<tempfile::NamedTempFile, FetchError> {
    log::debug!("SPARQL POST endpoint: {endpoint}");

    let client = http_client()?;
    let mut last_err: Option<FetchError> = None;

    'attempts: for attempt in 0..MAX_HTTP_ATTEMPTS {
        let result = client
            .post(endpoint)
            .header("Accept", format.accept())
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(build_sparql_form_body(sparql, format))
            .send()
            .await;

        match result {
            Ok(mut resp) => {
                let status = resp.status();
                let code = status.as_u16();
                if status.is_success() {
                    let mut file = tempfile::NamedTempFile::new()
                        .map_err(|e| FetchError::Parse(format!("tempfile create failed: {e}")))?;
                    let mut preview = Vec::with_capacity(2048);
                    let mut wrote_any = false;

                    loop {
                        match resp.chunk().await {
                            Ok(Some(chunk)) => {
                                wrote_any = true;
                                if preview.len() < 2048 {
                                    let take = (2048 - preview.len()).min(chunk.len());
                                    preview.extend_from_slice(&chunk[..take]);
                                }
                                file.write_all(&chunk).map_err(|e| {
                                    FetchError::Parse(format!("tempfile write failed: {e}"))
                                })?;
                            }
                            Ok(None) => break,
                            Err(e) => {
                                let err = FetchError::Network(e.to_string());
                                if attempt + 1 < MAX_HTTP_ATTEMPTS {
                                    last_err = Some(err);
                                    continue 'attempts;
                                }
                                return Err(err);
                            }
                        }
                    }

                    if !wrote_any {
                        return Err(FetchError::Empty);
                    }

                    let preview_text = String::from_utf8_lossy(&preview);
                    if looks_like_gateway_error(&preview_text) {
                        let err =
                            FetchError::Http(502, "upstream gateway error (HTML payload)".into());
                        if attempt + 1 < MAX_HTTP_ATTEMPTS {
                            last_err = Some(err);
                            continue;
                        }
                        return Err(err);
                    }

                    file.as_file_mut()
                        .rewind()
                        .map_err(|e| FetchError::Parse(format!("tempfile rewind failed: {e}")))?;
                    return Ok(file);
                }

                let body = resp.text().await.unwrap_or_default();
                let detail = compact_http_error_text(&body);
                log::error!("event=sparql_http_error status={code} detail={detail}");
                if (400..500).contains(&code) {
                    return Err(FetchError::Http(code, detail));
                }
                last_err = Some(FetchError::Http(code, detail));
            }
            Err(e) => {
                last_err = Some(FetchError::Network(e.to_string()));
            }
        }
    }

    Err(last_err.unwrap_or_else(|| FetchError::Network("unknown error".into())))
}

/// Fetch a fully-formed export URL (for example with `action=csv_export`) and
/// return raw response bytes.
///
/// This is useful for clients that want direct `QLever` export representations
/// while still using HTTP content negotiation (`Accept` / `Accept-Encoding`).
///
/// # Errors
/// Returns [`FetchError`] for transport/HTTP failures or empty responses.
pub async fn fetch_export_url_bytes(
    url: &str,
    format: ResponseFormat,
) -> Result<Vec<u8>, FetchError> {
    fetch_url_bytes_with_accept(url, format.accept()).await
}

/// Fetch an arbitrary URL and return raw response bytes.
///
/// Unlike [`fetch_export_url_bytes`], this does not constrain the `Accept`
/// header to a specific SPARQL representation. It is used for API-managed
/// download artifacts such as `application/gzip` attachments.
///
/// # Errors
/// Returns [`FetchError`] for transport/HTTP failures or empty responses.
pub async fn fetch_url_bytes(url: &str) -> Result<Vec<u8>, FetchError> {
    fetch_url_bytes_with_accept(url, "*/*").await
}

async fn fetch_url_bytes_with_accept(url: &str, accept: &str) -> Result<Vec<u8>, FetchError> {
    let client = http_client()?;
    let mut last_err: Option<FetchError> = None;

    for attempt in 0..MAX_HTTP_ATTEMPTS {
        let result = client.get(url).header("Accept", accept).send().await;

        match result {
            Ok(resp) => {
                let status = resp.status();
                let code = status.as_u16();
                if status.is_success() {
                    return match resp.bytes().await {
                        Ok(bytes) if bytes.is_empty() => Err(FetchError::Empty),
                        Ok(bytes) => {
                            let preview = String::from_utf8_lossy(&bytes);
                            if looks_like_gateway_error(&preview) {
                                let err = FetchError::Http(
                                    502,
                                    "upstream gateway error (HTML payload)".into(),
                                );
                                if attempt + 1 < MAX_HTTP_ATTEMPTS {
                                    last_err = Some(err);
                                    continue;
                                }
                                return Err(err);
                            }
                            Ok(bytes.to_vec())
                        }
                        Err(e) => {
                            let err = FetchError::Network(e.to_string());
                            if attempt + 1 < MAX_HTTP_ATTEMPTS {
                                last_err = Some(err);
                                continue;
                            }
                            Err(err)
                        }
                    };
                }

                let body = resp.text().await.unwrap_or_default();
                let detail = compact_http_error_text(&body);
                if (400..500).contains(&code) {
                    return Err(FetchError::Http(code, detail));
                }
                last_err = Some(FetchError::Http(code, detail));
            }
            Err(e) => {
                last_err = Some(FetchError::Network(e.to_string()));
            }
        }
    }

    Err(last_err.unwrap_or_else(|| FetchError::Network("unknown error".into())))
}

fn build_sparql_form_body(sparql: &str, format: ResponseFormat) -> String {
    let encoded = urlencoding::encode(sparql);
    // "query=" + encoded + optional "&action=<name>"
    let action = format.action();
    let capacity = 6 + encoded.len() + action.map_or(0, |a| 8 + a.len());
    let mut body = String::with_capacity(capacity);
    body.push_str("query=");
    body.push_str(&encoded);
    if let Some(action) = action {
        body.push_str("&action=");
        body.push_str(action);
    }
    body
}

fn http_client() -> Result<&'static reqwest::Client, FetchError> {
    static CLIENT: OnceLock<Result<reqwest::Client, String>> = OnceLock::new();
    match CLIENT.get_or_init(build_http_client) {
        Ok(client) => Ok(client),
        Err(msg) => Err(FetchError::Network(format!(
            "failed to initialize SPARQL HTTP client: {msg}"
        ))),
    }
}

fn build_http_client() -> Result<reqwest::Client, String> {
    #[cfg(target_arch = "wasm32")]
    {
        // In the browser, fetch automatically sends `Accept-Encoding: gzip,
        // deflate, br` and decompresses transparently — no extra configuration
        // is required.
        reqwest::Client::builder()
            .build()
            .map_err(|e| e.to_string())
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        // Enable automatic gzip decompression so QLever can return compressed
        // CSV/JSON/Turtle payloads. This adds `Accept-Encoding: gzip` to every
        // request and decodes the response body with flate2 before handing bytes
        // to the caller — substantially reducing transfer size for large result
        // sets without any changes to callers.
        reqwest::Client::builder()
            .connect_timeout(Duration::from_secs(8))
            .timeout(Duration::from_mins(2))
            .pool_idle_timeout(Duration::from_secs(90))
            .pool_max_idle_per_host(32)
            .tcp_keepalive(Duration::from_secs(30))
            .gzip(true)
            .build()
            .map_err(|e| e.to_string())
    }
}

fn looks_like_gateway_error(body: &str) -> bool {
    let cap = body.len().min(2048);
    let safe_end = (0..=cap)
        .rev()
        .find(|&i| body.is_char_boundary(i))
        .unwrap_or(0);
    let sample = &body[..safe_end];
    let html = contains_ci(sample, "<html")
        || contains_ci(sample, "<!doctype")
        || contains_ci(sample, "<head")
        || contains_ci(sample, "<title");
    let gateway = contains_ci(sample, "bad gateway")
        || contains_ci(sample, "gateway timeout")
        || contains_ci(sample, "service unavailable")
        || contains_ci(sample, "upstream")
        || contains_ci(sample, "nginx")
        || contains_ci(sample, "cloudflare");
    html && gateway
}

fn contains_ci(h: &str, needle: &str) -> bool {
    if needle.len() > h.len() {
        return false;
    }
    let nb = needle.as_bytes();
    let hb = h.as_bytes();
    for i in 0..=hb.len() - nb.len() {
        if hb[i..i + nb.len()]
            .iter()
            .zip(nb)
            .all(|(a, b)| a.eq_ignore_ascii_case(b))
        {
            return true;
        }
    }
    false
}

fn compact_http_error_text(body: &str) -> String {
    const MAX_CHARS: usize = 240;
    let trimmed = body.trim();
    if trimmed.is_empty() {
        return "empty response body".into();
    }

    if let Some(exception) = parse_json_exception_field(trimmed) {
        return truncate_chars(exception.trim(), MAX_CHARS);
    }

    let first_meaningful_line = trimmed
        .lines()
        .map(str::trim)
        .find(|line| !line.is_empty() && *line != "{" && *line != "}")
        .unwrap_or(trimmed);

    truncate_chars(first_meaningful_line.trim_matches(','), MAX_CHARS)
}

fn parse_json_exception_field(input: &str) -> Option<String> {
    let key = "\"exception\"";
    let key_pos = input.find(key)?;
    let mut rest = input[key_pos + key.len()..].trim_start();
    rest = rest.strip_prefix(':')?.trim_start();
    let quoted = rest.strip_prefix('"')?;

    let mut out = String::new();
    let mut escaped = false;
    for ch in quoted.chars() {
        if escaped {
            let decoded = match ch {
                'n' => '\n',
                'r' => '\r',
                't' => '\t',
                '"' => '"',
                '\\' => '\\',
                other => other,
            };
            out.push(decoded);
            escaped = false;
            continue;
        }
        match ch {
            '\\' => escaped = true,
            '"' => return Some(out),
            other => out.push(other),
        }
    }
    None
}

fn truncate_chars(text: &str, max_chars: usize) -> String {
    // Single forward scan: find the byte position after `max_chars` codepoints.
    let mut chars = text.char_indices();
    match chars.nth(max_chars) {
        // Fewer than max_chars codepoints — no truncation needed.
        None => text.to_string(),
        Some((byte_pos, _)) => {
            let mut out = String::with_capacity(byte_pos + 4); // 4 bytes for '…'
            out.push_str(&text[..byte_pos]);
            out.push('…');
            out
        }
    }
}

// ── CSV helpers ───────────────────────────────────────────────────────────────

/// Index of a named header column (None if absent).
#[must_use]
pub fn col_idx(headers: &csv::StringRecord, name: &str) -> Option<usize> {
    headers.iter().position(|h| h == name)
}

/// Get a trimmed field value by optional column index.
#[must_use]
pub fn field(record: &csv::StringRecord, idx: Option<usize>) -> &str {
    idx.and_then(|i| record.get(i)).unwrap_or("").trim()
}

/// Strip the Wikidata entity URI prefix to get a bare QID (e.g. `Q12345`).
///
/// Accepts:
/// * Full canonical URIs:  `http://www.wikidata.org/entity/Q12345`  ([`WIKIDATA_ENTITY_BASE`])
/// * HTTPS variant:        `https://www.wikidata.org/entity/Q12345`
/// * Bare QIDs:            `Q12345`
///
/// Returns an empty string for any unrecognized format.
///
/// [`WIKIDATA_ENTITY_BASE`]: crate::lotus::models::WIKIDATA_ENTITY_BASE
pub fn extract_qid(s: &str) -> String {
    use crate::lotus::models::WIKIDATA_ENTITY_BASE;
    const WIKIDATA_ENTITY_BASE_HTTPS: &str = "https://www.wikidata.org/entity/";

    let candidate = s
        .strip_prefix(WIKIDATA_ENTITY_BASE)
        .or_else(|| s.strip_prefix(WIKIDATA_ENTITY_BASE_HTTPS))
        .unwrap_or(s)
        .trim();

    // All QID characters are ASCII — check bytes instead of chars to avoid
    // the full Unicode iterator overhead.
    let bytes = candidate.as_bytes();
    if bytes.first() == Some(&b'Q') && bytes[1..].iter().all(u8::is_ascii_digit) && bytes.len() > 1
    {
        candidate.to_string()
    } else {
        String::new()
    }
}

/// Return `Some(s)` only if `s` is non-empty after trimming.
#[must_use]
pub fn non_empty(s: &str) -> Option<&str> {
    let t = s.trim();
    if t.is_empty() { None } else { Some(t) }
}

/// Prefer `a`, fall back to `b`, return None if both empty.
#[must_use]
pub fn coalesce<'a>(a: &'a str, b: &'a str) -> Option<&'a str> {
    non_empty(a).or_else(|| non_empty(b))
}

/// Parse `2021-04-23T00:00:00Z` or `2021` → year as i32.
#[must_use]
pub fn parse_year(s: &str) -> Option<i32> {
    s.trim().split(['-', 'T']).next()?.trim().parse().ok()
}

/// Normalise a DOI: strip `https://doi.org/` prefix if present.
#[must_use]
pub fn clean_doi(s: &str) -> Option<String> {
    let t = s.trim();
    if t.is_empty() {
        return None;
    }
    if let Some(doi) = t.split("doi.org/").last() {
        let doi = doi.trim();
        if !doi.is_empty() {
            return Some(doi.to_string());
        }
    }
    Some(t.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_html_gateway_payloads() {
        let html = "<html><head><title>502 Bad Gateway</title></head><body>nginx</body></html>";
        assert!(looks_like_gateway_error(html));
    }

    #[test]
    fn does_not_flag_regular_csv_as_gateway_error() {
        let csv = "compound,taxon\nQ1,Q2\n";
        assert!(!looks_like_gateway_error(csv));
    }

    #[test]
    fn extract_qid_handles_uri_and_plain_qid() {
        assert_eq!(
            extract_qid("http://www.wikidata.org/entity/Q12345"),
            "Q12345"
        );
        assert_eq!(extract_qid("Q999"), "Q999");
        assert_eq!(extract_qid("not-a-qid"), "");
    }

    #[test]
    fn clean_doi_normalizes_prefixed_urls() {
        assert_eq!(
            clean_doi("https://doi.org/10.1000/xyz"),
            Some("10.1000/xyz".to_string())
        );
        assert_eq!(clean_doi("  "), None);
    }

    #[test]
    fn compact_http_error_text_prefers_json_exception_field() {
        let body = r#"{
  "exception": "Trying to insert a cache key which was already present",
  "query": "SELECT ..."
}"#;
        assert_eq!(
            compact_http_error_text(body),
            "Trying to insert a cache key which was already present"
        );
    }

    #[test]
    fn compact_http_error_text_truncates_long_fallback_line() {
        let body = format!("{{\n  \"detail\": \"{}\"\n}}", "x".repeat(400));
        let compact = compact_http_error_text(&body);
        assert!(compact.chars().count() <= 241);
        assert!(compact.ends_with('…'));
    }
}
