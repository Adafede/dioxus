// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the smellfish-rs project

//! Typed domain errors for CSV parsing, `RDKit` JSON decoding, and SPARQL queries.

use lotus_search::FetchError;
use thiserror::Error;

/// Errors returned by CSV ingestion, `RDKit` payload decoding, and SPARQL enrichment.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum SmellfishError {
    /// Input text was empty or whitespace-only.
    #[error("Input is empty")]
    EmptyInput,
    /// CSV header was present without any data rows.
    #[error("CSV does not contain any data rows")]
    NoDataRows,
    /// A quoted CSV field was left unclosed at EOF.
    #[error("Unterminated quoted CSV field")]
    UnterminatedCsvQuote,
    /// `RDKit` bridge JSON could not be parsed.
    #[error("invalid RDKit response JSON: {0}")]
    InvalidRdkitJson(#[from] serde_json::Error),
    /// Upstream SPARQL HTTP request failed.
    #[error(transparent)]
    SparqlFetch(#[from] FetchError),
    /// SPARQL endpoint returned invalid JSON.
    #[error("invalid SPARQL JSON: {0}")]
    InvalidSparqlJson(#[source] serde_json::Error),
    /// SPARQL JSON lacked the expected `results.bindings` array.
    #[error("SPARQL response missing bindings")]
    MissingSparqlBindings,
}
