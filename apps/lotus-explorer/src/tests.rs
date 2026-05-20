// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

use crate::download::DownloadFormat;

fn is_supported_download_format(fmt: &str) -> bool {
    DownloadFormat::from_str(fmt).is_some()
}

#[test]
fn supported_download_formats_include_documented_values() {
    assert!(is_supported_download_format("csv"));
    assert!(is_supported_download_format("json"));
    assert!(is_supported_download_format("ndjson"));
    assert!(is_supported_download_format("rdf"));
    assert!(!is_supported_download_format("ttl"));
}

#[test]
fn supported_download_formats_allow_case_and_whitespace_variants() {
    assert!(is_supported_download_format(" CSV "));
    assert!(is_supported_download_format("Json"));
    assert!(is_supported_download_format("RDF"));
}
