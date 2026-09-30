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
mod tests {
    // The panic lints keep shipped code free of panics on an endpoint's
    // untrusted input. A poisoned lock here means a test already failed and
    // panicked while holding it, so the second panic says nothing new.
    #![allow(clippy::expect_used, clippy::panic)]

    use super::*;
    use futures::executor::block_on;
    use lotus_search::testing::Scripted;

    const ASK: &str = "SELECT ?s WHERE { ?s ?p ?o }";

    #[test]
    fn the_form_body_encodes_the_query_and_names_the_format() {
        let body = form_body(ASK, ResponseFormat::SparqlJson);
        assert_eq!(
            body,
            "query=SELECT%20%3Fs%20WHERE%20%7B%20%3Fs%20%3Fp%20%3Fo%20%7D\
             &action=sparql_json_export"
        );
    }

    #[test]
    fn a_format_qlever_cannot_name_sends_no_action() {
        // `NTriples` has no `action=`, so only `Accept` says what is wanted.
        assert_eq!(
            form_body("ASK", ResponseFormat::NTriples),
            "query=ASK",
            "an endpoint with nothing to name gets no action"
        );
    }

    #[test]
    fn an_ampersand_in_a_query_cannot_forge_a_parameter() {
        // Left unencoded this would be `query=...a&b...&action=csv_export`, and
        // the endpoint would read two parameters instead of one.
        let body = form_body("SELECT ?s { ?s ?p \"a&b\" }", ResponseFormat::Csv);
        assert_eq!(body.matches("&action=").count(), 1, "{body}");
        assert!(
            body.contains("%26b"),
            "the literal's `&` is not encoded: {body}"
        );
    }

    #[test]
    fn a_gateways_reason_is_its_title_and_not_its_markup() {
        let body = b"<html>\n<head><title>502 Bad Gateway</title></head>\n</html>";
        assert_eq!(summarise(body), "502 Bad Gateway");
    }

    #[test]
    fn a_json_error_becomes_one_line() {
        assert_eq!(
            summarise(br#"{"exception":"Variable ?s was not declared"}"#),
            r#"{"exception":"Variable ?s was not declared"}"#
        );
    }

    #[test]
    fn a_long_complaint_is_truncated_with_an_ellipsis() {
        let summarised = summarise("x".repeat(500).as_bytes());
        assert!(summarised.ends_with('…'), "{summarised}");
        assert_eq!(summarised.chars().count(), MAX_MESSAGE_CHARS + 1);
    }

    #[test]
    fn an_endpoint_that_says_nothing_still_produces_a_message() {
        assert_eq!(summarise(b""), "the endpoint gave no reason");
    }

    #[test]
    fn the_query_goes_to_the_url_it_was_given() -> Result<(), FetchError> {
        // The reason this module exists. `lotus-search`'s own `Endpoint` can only
        // name the three public LOTUS services, so a query that went through it
        // would silently land on Wikidata's endpoint instead of PubChem's.
        let http = Scripted::new(vec![(200, "[]")]);
        block_on(run_query(
            &http,
            QLEVER_PUBCHEM,
            ASK,
            ResponseFormat::SparqlJson,
        ))?;
        assert_eq!(http.endpoints(), [QLEVER_PUBCHEM]);
        Ok(())
    }

    #[test]
    fn the_answer_is_returned_as_text() -> Result<(), FetchError> {
        let http = Scripted::new(vec![(200, r#"{"head":{"vars":["s"]}}"#)]);
        let body = block_on(run_query(
            &http,
            QLEVER_PUBCHEM,
            ASK,
            ResponseFormat::SparqlJson,
        ))?;
        assert_eq!(body, r#"{"head":{"vars":["s"]}}"#);
        assert_eq!(http.call_count(), 1);
        Ok(())
    }

    #[test]
    fn a_rejected_query_is_not_asked_twice() {
        // A 4xx is the query's fault. Asking WDQS again would double the load on
        // a query that is already known to be wrong.
        let http = Scripted::new(vec![(400, "syntax error"), (200, "[]")]);
        let err = block_on(run_query(
            &http,
            WDQS_WIKIDATA,
            ASK,
            ResponseFormat::SparqlJson,
        ))
        .expect_err("a 400 is an error");
        assert_eq!(
            err,
            FetchError::Http {
                status: 400,
                message: "syntax error".into()
            }
        );
        assert_eq!(http.call_count(), 1, "the rejected query was asked again");
    }

    #[test]
    fn a_gateway_is_asked_a_second_time_and_can_answer() -> Result<(), FetchError> {
        let http = Scripted::new(vec![(503, ""), (200, "[]")]);
        block_on(run_query(
            &http,
            QLEVER_WIKIDATA,
            ASK,
            ResponseFormat::SparqlJson,
        ))?;
        assert_eq!(http.call_count(), 2, "the retry did not happen");
        Ok(())
    }

    #[test]
    fn a_gateway_that_stays_down_reports_its_reason() {
        let http = Scripted::new(vec![(502, "<title>502 Bad Gateway</title>")]);
        http.then_always_from(0, 502, "<title>502 Bad Gateway</title>");
        let err = block_on(run_query(
            &http,
            QLEVER_WIKIDATA,
            ASK,
            ResponseFormat::SparqlJson,
        ))
        .expect_err("a persistent 502 is an error");
        assert_eq!(
            err,
            FetchError::Http {
                status: 502,
                message: "502 Bad Gateway".into()
            }
        );
        assert_eq!(http.call_count(), 2, "no third attempt");
    }

    #[test]
    fn an_unreachable_endpoint_reports_the_reason_it_was_unreachable() {
        // A status of 0 is how the scripted transport says the request never
        // arrived, which is the case a retry is actually for. The retry finds an
        // empty `200`, because the script is spent — and that must not become the
        // reported reason.
        let http = Scripted::new(vec![(0, "")]);
        let err = block_on(run_query(
            &http,
            WDQS_WIKIDATA,
            ASK,
            ResponseFormat::SparqlJson,
        ))
        .expect_err("an unreachable endpoint is an error");
        assert!(
            matches!(err, FetchError::Network(_)),
            "the first failure is the one reported, not the retry's: {err}"
        );
        assert_eq!(http.call_count(), 2, "a network failure is worth one retry");
    }

    #[test]
    fn an_empty_answer_is_an_error_and_not_an_empty_result() {
        // Parsing an empty body would report "no compounds found", which reads as
        // an answer. It is a failed request.
        let http = Scripted::new(vec![(200, "")]);
        let err = block_on(run_query(
            &http,
            QLEVER_WIKIDATA,
            ASK,
            ResponseFormat::SparqlJson,
        ))
        .expect_err("an empty 200 is an error");
        assert_eq!(err, FetchError::Empty);
        assert_eq!(http.call_count(), 1, "an empty body is not retryable");
    }
}
