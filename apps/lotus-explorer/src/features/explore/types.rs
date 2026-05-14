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
    ResolvingTaxon,
    Counting,
    FetchingPreview,
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
    #[error("taxon not found: {input}")]
    TaxonNotFound { input: String },
    #[allow(dead_code)]
    #[error("taxon resolution produced no candidates")]
    TaxonResolutionNoMatch,
    #[error("unsupported download format: {format}")]
    UnsupportedFormat { format: String },
}

/// Fine-grained CSV / data parse fault.
// Some variants are only reachable on non-wasm targets (e.g. FallbackCsv).
#[allow(dead_code)]
#[derive(Clone, Debug, PartialEq, Error)]
pub enum ParseFault {
    #[error("taxon csv parse failed: {details}")]
    TaxonCsv { details: String },
    #[error("taxon candidate selection failed: {details}")]
    TaxonPick { details: String },
    #[error("count csv parse failed: {details}")]
    CountCsv { details: String },
    #[error("display csv parse failed: {details}")]
    DisplayCsv { details: String },
    #[error("fallback csv parse failed: {details}")]
    FallbackCsv { details: String },
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
        stage: &'static str,
        #[source]
        source: crate::repositories::RepositoryError,
    },

    #[error("parse: {0}")]
    Parse(ParseFault),

    #[cfg(target_arch = "wasm32")]
    #[error("memory limit reached during {stage}")]
    MemoryLimit { stage: &'static str },
}

impl DomainError {
    pub fn kind(&self) -> ErrorKind {
        match self {
            Self::Validation(_) => ErrorKind::Validation,
            Self::Transport { .. } => ErrorKind::Network,
            Self::Parse(_) => ErrorKind::Parse,
            #[cfg(target_arch = "wasm32")]
            Self::MemoryLimit { .. } => ErrorKind::Memory,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;

    #[test]
    fn transport_domain_error_exposes_repository_source() {
        let err = DomainError::Transport {
            stage: "display query",
            source: crate::repositories::RepositoryError::network("timeout"),
        };

        let source = err.source().expect("transport errors should expose source");
        assert_eq!(source.to_string(), "network error: timeout");
    }
}
