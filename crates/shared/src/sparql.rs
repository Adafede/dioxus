//! Generic SPARQL/QLever HTTP utilities shared by all apps.
//!
//! QLever CSV export URL format:
//!   `https://qlever.dev/api/wikidata?query=<encoded>&action=csv_export`

/// Default QLever endpoint for Wikidata (used by lotus-explorer).
pub const QLEVER_WIKIDATA: &str = "https://qlever.dev/api/wikidata";

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
/// Mirrors the Python `execute_with_retry` behaviour: up to two attempts,
/// with `Accept: text/csv` so the endpoint can honour content negotiation
/// even when the `action=csv_export` form parameter is ignored. Retries
/// transient network / 5xx errors; 4xx errors fail fast.
pub async fn execute_sparql(sparql: &str, endpoint: &str) -> Result<String, FetchError> {
    log::debug!("SPARQL POST endpoint: {endpoint}");

    const MAX_ATTEMPTS: u32 = 2;
    let client = reqwest::Client::new();
    let mut last_err: Option<FetchError> = None;

    for attempt in 0..MAX_ATTEMPTS {
        let result = client
            .post(endpoint)
            // `Accept` and `Content-Type: application/x-www-form-urlencoded`
            // (added by `.form(...)`) are both CORS-safelisted, so this stays
            // a *simple* request and no preflight is triggered. Do **not**
            // add `User-Agent` or any other custom header here — browsers
            // refuse to let WASM set them and the resulting preflight is
            // rejected by QLever with an opaque "CORS request did not
            // succeed" error.
            .header("Accept", "text/csv")
            .form(&[("query", sparql), ("action", "csv_export")])
            .send()
            .await;

        match result {
            Ok(resp) => {
                let status = resp.status();
                let code = status.as_u16();
                if status.is_success() {
                    match resp.text().await {
                        Ok(text) if text.trim().is_empty() => return Err(FetchError::Empty),
                        Ok(text) => {
                            // QLever sometimes returns an HTML gateway-error page with
                            // 200 OK when the upstream SPARQL server is flaky. Detect.
                            if looks_like_gateway_error(&text) {
                                last_err = Some(FetchError::Http(
                                    502,
                                    "upstream gateway error (HTML payload)".into(),
                                ));
                                if attempt + 1 < MAX_ATTEMPTS {
                                    continue;
                                }
                                return Err(last_err.unwrap());
                            }
                            return Ok(text);
                        }
                        Err(e) => {
                            last_err = Some(FetchError::Network(e.to_string()));
                            if attempt + 1 < MAX_ATTEMPTS {
                                continue;
                            }
                            return Err(last_err.unwrap());
                        }
                    }
                }

                let body = resp.text().await.unwrap_or_default();
                log::error!("HTTP {code}: {body}");
                // Fail fast on client errors (4xx); retry on server errors (5xx).
                if (400..500).contains(&code) {
                    return Err(FetchError::Http(code, body));
                }
                last_err = Some(FetchError::Http(code, body));
            }
            Err(e) => {
                last_err = Some(FetchError::Network(e.to_string()));
            }
        }
    }

    Err(last_err.unwrap_or_else(|| FetchError::Network("unknown error".into())))
}

fn looks_like_gateway_error(body: &str) -> bool {
    let sample: String = body.chars().take(2048).collect();
    let low = sample.to_ascii_lowercase();
    let html = low.contains("<html")
        || low.contains("<!doctype")
        || low.contains("<head")
        || low.contains("<title");
    let gateway = low.contains("bad gateway")
        || low.contains("gateway timeout")
        || low.contains("service unavailable")
        || low.contains("upstream")
        || low.contains("nginx")
        || low.contains("cloudflare");
    html && gateway
}

// ── CSV helpers ───────────────────────────────────────────────────────────────

/// Index of a named header column (None if absent).
pub fn col_idx(headers: &csv::StringRecord, name: &str) -> Option<usize> {
    headers.iter().position(|h| h == name)
}

/// Get a trimmed field value by optional column index.
pub fn field<'a>(record: &'a csv::StringRecord, idx: Option<usize>) -> &'a str {
    idx.and_then(|i| record.get(i)).unwrap_or("").trim()
}

/// Strip the Wikidata entity URL prefix to get a bare QID (e.g. `Q12345`).
/// Also accepts bare QIDs like `Q12345` directly.
pub fn extract_qid(s: &str) -> String {
    // Full URI: http://www.wikidata.org/entity/Q12345
    if let Some(rest) = s.split("wikidata.org/entity/").last() {
        let r = rest.trim();
        if r.starts_with('Q') && r[1..].chars().all(|c| c.is_ascii_digit()) {
            return r.to_string();
        }
    }
    // Already a bare QID
    if s.starts_with('Q') && s[1..].chars().all(|c| c.is_ascii_digit()) {
        return s.to_string();
    }
    String::new()
}

/// Return `Some(s)` only if `s` is non-empty after trimming.
pub fn non_empty(s: &str) -> Option<String> {
    let t = s.trim();
    if t.is_empty() {
        None
    } else {
        Some(t.to_string())
    }
}

/// Prefer `a`, fall back to `b`, return None if both empty.
pub fn coalesce<'a>(a: &'a str, b: &'a str) -> Option<String> {
    non_empty(a).or_else(|| non_empty(b))
}

/// Parse `2021-04-23T00:00:00Z` or `2021` → year as i32.
pub fn parse_year(s: &str) -> Option<i32> {
    s.trim().split(['-', 'T']).next()?.trim().parse().ok()
}

/// Normalise a DOI: strip `https://doi.org/` prefix if present.
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
