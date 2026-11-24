// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the smellfish-rs project

use crate::sparql::{MAX_MESSAGE_CHARS, QLEVER_PUBCHEM, form_body, run_query, summarise};
use futures_executor::block_on;
use lotus_search::testing::Scripted;
use lotus_search::{FetchError, QLEVER_WIKIDATA, ResponseFormat, WDQS_WIKIDATA};

const ASK: &str = "SELECT ?s WHERE { ?s ?p ?o }";

#[test]
fn form_body_encodes_query_and_format_action() {
    assert_eq!(
        form_body(ASK, ResponseFormat::SparqlJson),
        "query=SELECT%20%3Fs%20WHERE%20%7B%20%3Fs%20%3Fp%20%3Fo%20%7D&action=sparql_json_export"
    );
    assert_eq!(form_body("ASK", ResponseFormat::NTriples), "query=ASK");
    let body = form_body("SELECT ?s { ?s ?p \"a&b\" }", ResponseFormat::Csv);
    assert_eq!(body.matches("&action=").count(), 1, "{body}");
    assert!(body.contains("%26b"), "{body}");
}

#[test]
fn summarise_prefers_html_title_and_truncates() {
    let body = b"<html>\n<head><title>502 Bad Gateway</title></head>\n</html>";
    assert_eq!(summarise(body), "502 Bad Gateway");
    assert_eq!(
        summarise(br#"{"exception":"Variable ?s was not declared"}"#),
        r#"{"exception":"Variable ?s was not declared"}"#
    );
    let long = summarise("x".repeat(500).as_bytes());
    assert!(long.ends_with('…'), "{long}");
    assert_eq!(long.chars().count(), MAX_MESSAGE_CHARS + 1);
    assert_eq!(summarise(b""), "the endpoint gave no reason");
}

#[test]
fn run_query_routes_url_and_handles_retries_deterministically() -> Result<(), FetchError> {
    let ok_http = Scripted::new(vec![(200, r#"{"head":{"vars":["s"]}}"#)]);
    let body = block_on(run_query(
        &ok_http,
        QLEVER_PUBCHEM,
        ASK,
        ResponseFormat::SparqlJson,
    ))?;
    assert_eq!(body, r#"{"head":{"vars":["s"]}}"#);
    assert_eq!(ok_http.endpoints(), [QLEVER_PUBCHEM]);
    assert_eq!(ok_http.call_count(), 1);

    let rejected = Scripted::new(vec![(400, "syntax error"), (200, "[]")]);
    let err = block_on(run_query(
        &rejected,
        WDQS_WIKIDATA,
        ASK,
        ResponseFormat::SparqlJson,
    ))
    .err();
    assert_eq!(
        err,
        Some(FetchError::Http {
            status: 400,
            message: "syntax error".into(),
        })
    );
    assert_eq!(rejected.call_count(), 1);

    let recovered = Scripted::new(vec![(503, ""), (200, "[]")]);
    block_on(run_query(
        &recovered,
        QLEVER_WIKIDATA,
        ASK,
        ResponseFormat::SparqlJson,
    ))?;
    assert_eq!(recovered.call_count(), 2);

    let gateway_down = Scripted::new(vec![(502, "<title>502 Bad Gateway</title>")]);
    gateway_down.then_always_from(0, 502, "<title>502 Bad Gateway</title>");
    let err_down = block_on(run_query(
        &gateway_down,
        QLEVER_WIKIDATA,
        ASK,
        ResponseFormat::SparqlJson,
    ))
    .err();
    assert_eq!(
        err_down,
        Some(FetchError::Http {
            status: 502,
            message: "502 Bad Gateway".into(),
        })
    );
    assert_eq!(gateway_down.call_count(), 2);

    let network_down = Scripted::new(vec![(0, "")]);
    let err_net = block_on(run_query(
        &network_down,
        WDQS_WIKIDATA,
        ASK,
        ResponseFormat::SparqlJson,
    ))
    .err();
    assert!(
        matches!(err_net, Some(FetchError::Network(_))),
        "{err_net:?}"
    );
    assert_eq!(network_down.call_count(), 2);

    let empty = Scripted::new(vec![(200, "")]);
    let err_empty = block_on(run_query(
        &empty,
        QLEVER_WIKIDATA,
        ASK,
        ResponseFormat::SparqlJson,
    ))
    .err();
    assert_eq!(err_empty, Some(FetchError::Empty));
    assert_eq!(empty.call_count(), 1);
    Ok(())
}
