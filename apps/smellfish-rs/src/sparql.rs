// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the smellfish-rs project

//! SPARQL over HTTP, for endpoints `lotus-search` does not name.
//!
//! `lotus-search` owns the parts of this policy that are not app-specific: which
//! representation to ask for (`ResponseFormat`), what a second attempt can fix
//! (`FetchError::is_retryable`), and the two public Wikidata endpoints, which
//! are re-exported below rather than restated. What it deliberately does not
//! offer is a way to name an endpoint that is not one of its three public LOTUS
//! services, and this app also queries `PubChem`'s own `QLever` instance, which
//! is not one of them.
//!
//! So the transport trait is used directly. It takes the endpoint as a `&str`,
//! which is the only reason the URL can live here at all. The parts left to this
//! module are the ones `lotus-search` could not have chosen: how many times to
//! ask, and how to reduce an endpoint's complaint to something a browser can
//! show.
//!
//! Nothing here is browser-specific. The one endpoint the browser build queries
//! that `lotus-search` has no name for is named below, and the request itself is
//! generic over [`Http`], so the whole module is tested natively against a
//! scripted transport and never needs a network.

use lotus_search::{FetchError, Http, HttpResponse, ResponseFormat};

// The two public LOTUS endpoints come from the dependency rather than being
// restated here: they are the URLs `lotus-search` itself would have posted to,
// and a second copy is a second place for them to drift.
pub(crate) use lotus_search::{QLEVER_WIKIDATA, WDQS_WIKIDATA};

/// `PubChem`'s `QLever` instance, queried over the same protocol with a different
/// vocabulary. There is no upstream constant for it because no LOTUS service
/// lives there.
pub(crate) const QLEVER_PUBCHEM: &str = "https://qlever.cs.uni-freiburg.de/api/pubchem";

/// How much of an endpoint's complaint is worth keeping. A SPARQL error can be a
/// stack trace, and this string ends up in a warning in a browser.
const MAX_MESSAGE_CHARS: usize = 200;

/// POST `query` to `url` and return the answer as text.
///
/// `url` is whatever the caller wants to ask, which is the reason this exists at
/// all: the endpoints are a property of the query, not of this app, and one of
/// them is not a service `lotus-search` has a name for.
///
/// A query is sent at most twice. The second attempt covers a dropped connection
/// or a gateway that was briefly unwell, and no more: an endpoint that is
/// already struggling should not have a client's patience added to its load.
///
/// A failure a retry cannot fix is final. A 4xx means the query itself is
/// rejected, and WDQS would reject the same query the second time.
///
/// The error reported is the *first* one. A retry that fails differently is the
/// same endpoint in the same state, and the later message tends to be the less
/// informative of the two: after a refused connection, a retry answered with an
/// empty `200` would otherwise be reported as "the query returned no results",
/// which describes the retry and not the problem.
///
/// # Errors
///
/// Returns the [`FetchError`] from the first attempt: a transport failure, a
/// non-2xx status carrying the endpoint's own one-line complaint, an empty
/// answer, or a body that is not UTF-8.
pub(crate) async fn run_query(
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

/// One attempt: POST, check the status, decode.
///
/// # Errors
///
/// See [`run_query`]. This is where each of its errors is produced.
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
    // The body is read even for a failure, because that is where the endpoint
    // says what it did not like.
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

/// The `application/x-www-form-urlencoded` body a SPARQL endpoint expects.
///
/// `action=` is how `QLever` is told which of its three export formats to
/// produce; it is sent for every endpoint, because an endpoint that does not
/// know the parameter ignores it and one that does needs it. `Accept` is sent
/// alongside it rather than instead of it: both are CORS-safelisted, so the
/// browser still treats the request as simple and sends no preflight, and
/// `WDQS` honours `Accept` where `QLever` honours `action`.
fn form_body(query: &str, format: ResponseFormat) -> String {
    let mut body = format!("query={}", urlencoding::encode(query));
    if let Some(action) = format.qlever_action() {
        body.push_str("&action=");
        body.push_str(action);
    }
    body
}

/// Reduce an endpoint's complaint to one readable line.
///
/// A gateway puts the reason in an HTML `<title>` and its markup around it says
/// nothing; an endpoint puts it in the body, sometimes as one line of JSON and
/// sometimes as a stack trace. The `<title>` is preferred when there is one, and
/// the result is truncated, because this string is shown in a browser and an
/// untruncated SPARQL error is unreadable there.
fn summarise(body: &[u8]) -> String {
    let text = String::from_utf8_lossy(body);

    let line = html_title(&text)
        .or_else(|| text.lines().map(str::trim).find(|l| !l.is_empty()))
        .unwrap_or("the endpoint gave no reason");

    if line.chars().count() <= MAX_MESSAGE_CHARS {
        return line.to_string();
    }
    let truncated: String = line.chars().take(MAX_MESSAGE_CHARS).collect();
    format!("{truncated}…")
}

/// The `<title>` of an error page, which is where a gateway states the failure.
fn html_title(text: &str) -> Option<&str> {
    if !text.contains('<') {
        return None;
    }
    let rest = text.split_once("<title>")?.1;
    let end = rest.find("</title>")?;
    let title = rest[..end].trim();
    (!title.is_empty()).then_some(title)
}

#[cfg(test)]
mod tests;
