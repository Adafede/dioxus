// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the smarts-evoliposuction project

//! Typed top-level CLI errors for `smarts-evoliposuction`.

use crate::SdfConvertError;
use thiserror::Error;

/// Unified error type returned by `smarts-evoliposuction` CLI subcommands.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum CliError {
    /// Filesystem I/O failure.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    /// CSV reader/writer failure.
    #[error("CSV error: {0}")]
    Csv(#[from] csv::Error),
    /// HTTP request failure during dataset download.
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    /// LMSD archive/SDF conversion failure.
    #[error(transparent)]
    SdfConvert(#[from] SdfConvertError),
    /// Input TSV/CSV file contained no header or data.
    #[error("Empty input file")]
    EmptyInput,
    /// Required column missing from TSV/CSV header.
    #[error("Missing {0} column")]
    MissingColumn(&'static str),
}
