// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the mgf-precursor-erro-rs project

//! Typed error definitions for MGF precursor analysis and recalibration.

/// Errors returned when validating domain inputs in `mgf-analysis-core`.
#[derive(Clone, Debug, PartialEq, thiserror::Error)]
#[non_exhaustive]
pub enum MgfAnalysisError {
    /// An adduct specification could not be parsed into valid ion terms.
    #[error("invalid or unsupported adduct specification: {spec}")]
    InvalidAdduct {
        /// Raw adduct string passed to parsing.
        spec: String,
    },
    /// A shrinkage factor `lambda` was non-finite or outside `[0.0, 1.0]`.
    #[error("shrinkage lambda must be finite and within [0.0, 1.0], got {0}")]
    InvalidShrinkageLambda(f64),
    /// An ion polarity / charge-mode string could not be recognized.
    #[error("unrecognized polarity: {0}")]
    InvalidPolarity(String),
}
