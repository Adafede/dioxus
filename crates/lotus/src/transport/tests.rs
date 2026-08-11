// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Unit tests for transport helpers.
//!
//! Only pure (non-async) helpers are tested here — `looks_like_gateway_error`,
//! `compact_http_error_text`, `extract_qid`, and `clean_doi`.

use super::error::{compact_http_error_text, looks_like_gateway_error};
use crate::transport::{clean_doi, extract_qid};

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
