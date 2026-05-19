// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Core domain types for the Explore feature.
//!
//! ## Error hierarchy
//!
//! Internal business logic uses [`DomainError`] — a structured, i18n-free type so
//! that formatting decisions remain at the UI boundary.  Components call
//! [`crate::components::layout::notices::format_domain_error`] to produce the
//! right locale string at render time.

use thiserror::Error;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum QueryPhase {
    Idle,
    PreparingQuery,
    ResolvingTaxon,
    FetchingResults,
    ProcessingResults,
    Rendering,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum ErrorKind {
    Validation,
    Network,
    Parse,
    #[cfg(target_arch = "wasm32")]
    Memory,
    #[default]
    Unknown,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum QueryStage {
    TaxonSearch,
    ResultsQuery,
}

impl QueryStage {
    pub const fn as_key(self) -> &'static str {
        match self {
            Self::TaxonSearch => "taxon_search",
            Self::ResultsQuery => "results_query",
        }
    }
}

impl std::fmt::Display for QueryStage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_key())
    }
}

// ── Taxon warning (structured, formatted at UI boundary) ─────────────────────

/// A structured warning about taxon resolution, formatted by the UI layer.
#[derive(Clone, Debug, PartialEq)]
pub enum TaxonWarning {
    /// The raw input was normalised before lookup.
    Standardized {
        original: String,
        standardized: String,
    },
    /// Multiple candidates found; `chosen_*` is the one we used.
    Ambiguous {
        chosen_name: String,
        chosen_qid: String,
        /// Top candidates as `"Name (QID)"` strings.
        candidates: Vec<String>,
    },
    /// Raw warning string received from the REST API response.
    ApiMessage(String),
}

// ── Domain error hierarchy (i18n-free) ───────────────────────────────────────

/// Fine-grained validation fault.
#[derive(Clone, Debug, PartialEq, Error)]
pub enum ValidationFault {
    #[error("empty input")]
    EmptyInput,
    #[error("taxon is too long")]
    TaxonTooLong,
    #[error("structure input is too long")]
    StructureTooLong,
    #[error("mass is out of range")]
    MassOutOfRange,
    #[error("mass range is invalid")]
    MassRangeInvalid,
    #[error("year is out of range")]
    YearOutOfRange,
    #[error("year range is invalid")]
    YearRangeInvalid,
    #[error("element count is too high")]
    ElementCountTooHigh,
    #[error("taxon not found: {input}")]
    TaxonNotFound { input: String },
    /// This variant is only used during non-wasm fallback flows.
    /// On wasm, taxon resolution is handled differently, but we keep this
    /// variant for API compatibility and to support test scenarios.
    #[allow(dead_code)]
    #[error("taxon resolution produced no candidates")]
    TaxonResolutionNoMatch,
    #[error("unsupported download format: {format}")]
    UnsupportedFormat { format: String },
}

/// Fine-grained CSV / data parse fault.
///
/// Parse variants are scoped to active Explore pipeline stages.
#[allow(dead_code)]
#[derive(Clone, Debug, PartialEq, Error)]
pub enum ParseFault {
    #[error("taxon csv parse failed: {details}")]
    TaxonCsv { details: String },
    #[error("taxon candidate selection failed: {details}")]
    TaxonPick { details: String },
    #[error("results csv parse failed: {details}")]
    ResultsCsv { details: String },
}

/// Top-level domain error used throughout the Explore feature.
///
/// Contains **no locale-dependent strings**; UI components call
/// [`crate::components::layout::notices::format_domain_error`] at render time.
#[derive(Clone, Debug, PartialEq, Error)]
pub enum DomainError {
    #[error("validation: {0}")]
    Validation(ValidationFault),

    #[error("transport at {stage}: {source}")]
    Transport {
        stage: QueryStage,
        #[source]
        source: crate::repositories::RepositoryError,
    },

    #[error("parse: {0}")]
    Parse(ParseFault),

    #[cfg(target_arch = "wasm32")]
    #[error("memory limit reached during {stage}")]
    MemoryLimit { stage: QueryStage },
}

impl DomainError {
    /// Construct a transport error for the given query stage and repository source.
    ///
    /// This function is primarily used in unit tests and benchmarks to construct
    /// error scenarios for testing error handling and propagation logic.
    #[allow(dead_code)] // Used in unit tests and test utilities
    pub fn transport(stage: QueryStage, source: crate::repositories::RepositoryError) -> Self {
        Self::Transport { stage, source }
    }

    /// Map a repository error to a transport domain error via `.map_err`.
    pub fn transport_at(
        stage: QueryStage,
    ) -> impl Fn(crate::repositories::RepositoryError) -> Self {
        move |source| Self::Transport { stage, source }
    }

    #[cfg(target_arch = "wasm32")]
    pub fn memory_limit(stage: QueryStage) -> Self {
        Self::MemoryLimit { stage }
    }

    pub fn kind(&self) -> ErrorKind {
        match self {
            Self::Validation(_) => ErrorKind::Validation,
            Self::Transport { .. } => ErrorKind::Network,
            Self::Parse(_) => ErrorKind::Parse,
            #[cfg(target_arch = "wasm32")]
            Self::MemoryLimit { .. } => ErrorKind::Memory,
        }
    }

    /// Extract the query stage at which this error occurred, if applicable.
    pub fn query_stage(&self) -> QueryStage {
        match self {
            Self::Transport { stage, .. } => *stage,
            #[cfg(target_arch = "wasm32")]
            Self::MemoryLimit { stage } => *stage,
            // For validation and parse errors, assume results stage as fallback
            _ => QueryStage::ResultsQuery,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;

    #[test]
    fn transport_domain_error_exposes_repository_source() {
        let err = DomainError::transport(
            QueryStage::ResultsQuery,
            crate::repositories::RepositoryError::network("timeout"),
        );

        let source = err.source().expect("transport errors should expose source");
        assert_eq!(source.to_string(), "network error: timeout");
    }

    #[test]
    fn transport_at_closure_maps_repository_error() {
        let mapper = DomainError::transport_at(QueryStage::ResultsQuery);
        let err = mapper(crate::repositories::RepositoryError::network("timed out"));
        assert!(matches!(
            err,
            DomainError::Transport {
                stage: QueryStage::ResultsQuery,
                ..
            }
        ));
    }
}
