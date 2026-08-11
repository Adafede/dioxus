// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Export-format and download-URL helpers shared by `lotus-api` and
//! `lotus-explore-rs`.
//!
//! Both apps independently defined an enum for CSV/JSON/RDF export and a
//! mapping from that enum to `action=` strings ("`csv_export`", "`qlever_json_export`",
//! "`turtle_export`").  This module is the single source of truth — apps import
//! [`ExportFormat`] and call [`qlever_export_url`] / [`build_upstream_export_url`]
//! instead of re-implementing the mapping.

#![allow(clippy::module_name_repetitions)]

/// The three archive formats supported by the LOTUS/QLever export pipeline.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ExportFormat {
    Csv,
    Json,
    Rdf,
}

impl ExportFormat {
    /// Parse from a CLI / URL fragment ("csv", "json", "ndjson", "rdf").
    ///
    /// Returns `None` for unrecognized formats.
    #[must_use]
    pub fn parse(s: &str) -> Option<Self> {
        let normalized = s.trim();
        if normalized.eq_ignore_ascii_case("csv") {
            Some(Self::Csv)
        } else if normalized.eq_ignore_ascii_case("json")
            || normalized.eq_ignore_ascii_case("ndjson")
        {
            Some(Self::Json)
        } else if normalized.eq_ignore_ascii_case("rdf") {
            Some(Self::Rdf)
        } else {
            None
        }
    }

    /// File extension without leading dot (e.g. `"csv"`).
    #[must_use]
    pub const fn extension(self) -> &'static str {
        match self {
            Self::Csv => "csv",
            Self::Json => "json",
            Self::Rdf => "rdf",
        }
    }

    /// `QLever` `action=` parameter value for this format.
    #[must_use]
    pub const fn qlever_action(self) -> &'static str {
        match self {
            Self::Csv => "csv_export",
            Self::Json => "qlever_json_export",
            Self::Rdf => "turtle_export",
        }
    }

    /// Prepares the SPARQL query for export, wrapping it in a `CONSTRUCT`
    /// template when the format is RDF.
    #[must_use]
    pub fn prepared_query(self, query: &str) -> String {
        match self {
            Self::Rdf => crate::queries::query_construct_from_select(query),
            _ => query.to_string(),
        }
    }

    /// MIME type associated with this export format.
    #[must_use]
    pub const fn content_type(self) -> &'static str {
        match self {
            Self::Csv => "text/csv;charset=utf-8",
            Self::Json => "application/sparql-results+json;charset=utf-8",
            Self::Rdf => "text/turtle;charset=utf-8",
        }
    }

    /// `perf::start_timer` label fragment for this format.
    #[must_use]
    pub const fn log_name(self) -> &'static str {
        match self {
            Self::Csv => "csv",
            Self::Json => "json",
            Self::Rdf => "rdf",
        }
    }

    /// `perf::start_timer` full label (e.g. `"LOTUS:download_csv"`).
    #[must_use]
    pub const fn timer_label(self) -> &'static str {
        match self {
            Self::Csv => "LOTUS:download_csv",
            Self::Json => "LOTUS:download_json",
            Self::Rdf => "LOTUS:download_rdf",
        }
    }

    /// Trigger timer label (full timer label + `"_trigger"`).
    #[must_use]
    pub fn trigger_timer_label(self) -> String {
        format!("{}_trigger", self.timer_label())
    }
}

/// Builds a `QLever` export URL for the given query and format.
///
/// RDF exports use a `CONSTRUCT` query so `QLever` emits triples rather than
/// `SELECT` rows; CSV and JSON go straight to `QLever`'s native export actions.
#[must_use]
pub fn qlever_export_url(query: &str, format: ExportFormat) -> String {
    let prepared_query = if format == ExportFormat::Rdf {
        crate::queries::query_construct_from_select(query)
    } else {
        query.to_string()
    };
    format!(
        "{}?query={}&action={}",
        crate::transport::QLEVER_WIKIDATA,
        urlencoding::encode(&prepared_query),
        format.qlever_action()
    )
}

/// Builds a `QLever` export URL for the given query and format action string.
///
/// This is kept for callers that need a raw action string (e.g. the lotus-api
/// `/v1/search` endpoint which lists direct `QLever` URLs alongside its own
/// gzip-cached export URLs).
#[must_use]
pub fn qlever_export_url_with_action(query: &str, action: &str) -> String {
    format!(
        "{}?query={}&action={action}",
        crate::transport::QLEVER_WIKIDATA,
        urlencoding::encode(query)
    )
}

/// Builds a lotus-api `/v1/export-file/{cache_key}/{format}` URL.
#[must_use]
pub fn api_export_file_url(cache_key: &str, format: ExportFormat) -> String {
    format!("/v1/export-file/{cache_key}/{}", format.extension())
}

/// Builds the upstream `QLever` export URL for a given query and format,
/// choosing the appropriate action string.
///
/// For RDF, the query is first wrapped in a `CONSTRUCT` template via
/// [`crate::queries::query_construct_from_select`].
#[must_use]
pub fn build_upstream_export_url(query: &str, format: ExportFormat) -> String {
    qlever_export_url(query, format)
}

/// Sanitizes a filename for safe browser download.
///
/// Removes control characters and replaces path separators and quotes with
/// underscores.  Delegates to [`upload::sanitize_filename`].
///
/// This function exists so callers that already depend on `lotus` don't also
/// need to pull in the `upload` crate just for one helper.
#[inline]
#[must_use]
pub fn sanitize_download_filename(input: &str) -> String {
    upload::sanitize_filename(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_known_formats() {
        assert_eq!(ExportFormat::parse("csv"), Some(ExportFormat::Csv));
        assert_eq!(ExportFormat::parse("json"), Some(ExportFormat::Json));
        assert_eq!(ExportFormat::parse("ndjson"), Some(ExportFormat::Json));
        assert_eq!(ExportFormat::parse("rdf"), Some(ExportFormat::Rdf));
        assert_eq!(ExportFormat::parse(" JSON "), Some(ExportFormat::Json));
        assert_eq!(ExportFormat::parse("RDF"), Some(ExportFormat::Rdf));
        assert_eq!(ExportFormat::parse("ttl"), None);
    }

    #[test]
    fn extensions_match_variants() {
        assert_eq!(ExportFormat::Csv.extension(), "csv");
        assert_eq!(ExportFormat::Json.extension(), "json");
        assert_eq!(ExportFormat::Rdf.extension(), "rdf");
    }

    #[test]
    fn qlever_actions_are_stable() {
        assert_eq!(ExportFormat::Csv.qlever_action(), "csv_export");
        assert_eq!(ExportFormat::Json.qlever_action(), "qlever_json_export");
        assert_eq!(ExportFormat::Rdf.qlever_action(), "turtle_export");
    }

    #[test]
    fn sanitize_download_filename_strips_path_separators() {
        assert!(!sanitize_download_filename("../../etc/passwd").contains('/'));
        assert!(!sanitize_download_filename("../../etc/passwd").contains('\\'));
    }
}
