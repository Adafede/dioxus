// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the lotus-explore-rs project

//! Export URL builders for QLever and WDQS endpoints.

use lotus::export::ExportFormat;
use lotus::transport::{WDQS_SCHOLARLY, WDQS_WIKIDATA};

/// Builds a `QLever` export URL for the given query and format.
///
/// This wraps `lotus::export::qlever_export_url` for use in the lotus-explore-rs app.
#[must_use]
pub fn qlever_export_url(query: &str, format: ExportFormat) -> String {
    lotus::export::qlever_export_url(query, format)
}

/// Builds a `WDQS` export URL for the given query and format.
///
/// WDQS uses direct SPARQL POST requests without the QLever `action=` parameter.
#[must_use]
pub fn wdqs_export_url(query: &str, format: ExportFormat) -> String {
    let prepared_query = format.prepared_query(query);
    let encoded_query = urlencoding::encode(&prepared_query);

    match format {
        ExportFormat::Csv => format!("{}?query={}&Accept=text/csv", WDQS_WIKIDATA, encoded_query),
        ExportFormat::Json => format!(
            "{}?query={}&Accept=application/sparql-results+json",
            WDQS_WIKIDATA, encoded_query
        ),
        ExportFormat::Rdf => format!(
            "{}?query={}&Accept=text/turtle",
            WDQS_WIKIDATA, encoded_query
        ),
    }
}

/// Builds a scholarly-subgraph WDQS export URL for the given query and format.
///
/// For downloads when falling back to WDQS with scholarly subgraph queries,
/// this URL points to the scholarly subgraph endpoint.
#[must_use]
pub fn wdqs_scholarly_export_url(query: &str, format: ExportFormat) -> String {
    let prepared_query = format.prepared_query(query);
    let encoded_query = urlencoding::encode(&prepared_query);

    match format {
        ExportFormat::Csv => format!("{}?query={}&Accept=text/csv", WDQS_SCHOLARLY, encoded_query),
        ExportFormat::Json => format!(
            "{}?query={}&Accept=application/sparql-results+json",
            WDQS_SCHOLARLY, encoded_query
        ),
        ExportFormat::Rdf => format!(
            "{}?query={}&Accept=text/turtle",
            WDQS_SCHOLARLY, encoded_query
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn qlever_url_generates_correct_format() {
        let url = qlever_export_url("SELECT ?s WHERE { ?s ?p ?o }", ExportFormat::Csv);
        assert!(url.starts_with(lotus::transport::QLEVER_WIKIDATA));
        assert!(url.contains("action=csv_export"));
    }

    #[test]
    fn wdqs_url_uses_correct_endpoint() {
        let url = wdqs_export_url("SELECT ?s WHERE { ?s ?p ?o }", ExportFormat::Csv);
        assert!(url.starts_with(WDQS_WIKIDATA));
        assert!(url.contains("&Accept=text/csv"));
    }

    #[test]
    fn wdqs_scholarly_url_uses_correct_endpoint() {
        let url = wdqs_scholarly_export_url("SELECT ?s WHERE { ?s ?p ?o }", ExportFormat::Csv);
        assert!(url.starts_with(WDQS_SCHOLARLY));
        assert!(url.contains("&Accept=text/csv"));
    }
}
