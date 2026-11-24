// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Structured errors for MGF parsing.

use thiserror::Error;

/// Error produced when parsing or validating MGF records.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[non_exhaustive]
pub enum MgfError {
    /// A malformed header key/value line was encountered.
    #[error("invalid MGF header line: {0}")]
    InvalidHeaderLine(String),

    /// A malformed MS2 peak line was encountered.
    #[error("invalid MGF peak line: {0}")]
    InvalidPeakLine(String),
}
