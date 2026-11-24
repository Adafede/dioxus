// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the smellfish-rs project

//! Transport-agnostic SPARQL over HTTP request encoding, retry, and error summarization.

use lotus_search::{FetchError, Http, HttpResponse, ResponseFormat};
use std::fmt::Write as _;

pub use lotus_search::{QLEVER_WIKIDATA, WDQS_WIKIDATA};

/// `PubChem`'s public `QLever` SPARQL endpoint.
pub const QLEVER_PUBCHEM: &str = "https://qlever.cs.uni-freiburg.de/api/pubchem";

pub(crate) const MAX_MESSAGE_CHARS: usize = 200;

/// POSTs `query` to `url`, retrying once on retryable failures and returning the first error.
///
/// # Errors
///
/// Returns the first [`FetchError`] encountered on transport/HTTP/decode failure.
pub async fn run_query(
    http: &impl Http,
    url: &str,
    query: &str,
    format: ResponseFormat,
) -> Result<String, FetchError> {
    match post(http, url, query, format).await {
        Ok(body) => Ok(body),
        Err(cause) if !cause.is_retryable() => Err(cause),
        Err(cause) => post(http, url, query, format).await.or(Err(cause)),
    }
}

async fn post(
    http: &impl Http,
    url: &str,
    query: &str,
    format: ResponseFormat,
) -> Result<String, FetchError> {
    let response = http
        .post(url, format.accept(), form_body(query, format))
        .await?;
    let status = response.status();
    let body = response.bytes().await?;

    if !(200..300).contains(&status) {
        return Err(FetchError::Http {
            status,
            message: summarise(&body),
        });
    }
    if body.is_empty() {
        return Err(FetchError::Empty);
    }

    String::from_utf8(body.to_vec())
        .map_err(|e| FetchError::Parse(format!("the response was not UTF-8: {e}")))
}

pub(crate) fn form_body(query: &str, format: ResponseFormat) -> String {
    let mut body = String::with_capacity(query.len() + 32);
    body.push_str("query=");
    for &byte in query.as_bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                body.push(char::from(byte));
            }
            other => {
                let _ = write!(body, "%{other:02X}");
            }
        }
    }
    if let Some(action) = format.qlever_action() {
        body.push_str("&action=");
        body.push_str(action);
    }
    body
}

pub(crate) fn summarise(body: &[u8]) -> String {
    let text = String::from_utf8_lossy(body);
    let line = lotus_search::html_title(&text)
        .or_else(|| text.lines().map(str::trim).find(|l| !l.is_empty()))
        .unwrap_or("the endpoint gave no reason");

    match line.char_indices().nth(MAX_MESSAGE_CHARS) {
        Some((byte_idx, _)) => format!("{}…", &line[..byte_idx]),
        None => line.to_string(),
    }
}
