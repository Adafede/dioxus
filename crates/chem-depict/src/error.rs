// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Structured errors for panic-guarded SMILES parsing.

use thiserror::Error;

/// Error returned when parsing a SMILES string via [`crate::parse_smiles_safe`].
#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[non_exhaustive]
pub enum ChemParseError {
    /// The input was empty or whitespace-only.
    #[error("SMILES input is empty")]
    Empty,

    /// The parser rejected the SMILES string.
    #[error("invalid SMILES `{smiles}`: {message}")]
    InvalidSmiles {
        /// The trimmed SMILES input.
        smiles: String,
        /// Underlying parser diagnostic message.
        message: String,
    },

    /// The underlying parser panicked while reading the SMILES string.
    #[error("SMILES parser panicked: {0}")]
    ParserPanicked(String),
}
