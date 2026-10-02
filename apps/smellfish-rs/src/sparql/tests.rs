// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the smellfish-rs project

// The `tests` tests, extracted from `sparql.rs`.
//
// They were inline, and together they were several times the size of the
// code they cover. `super` is still the `sparql` module, so
// `use super::*` below reaches exactly what it did before the move.
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
