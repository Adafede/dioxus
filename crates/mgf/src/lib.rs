// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Streaming MGF (`Mascot Generic Format`) block parser and spectrum model.
//!
//! Unifies `BEGIN IONS ... END IONS` line-by-line parsing across the
//! workspace with explicit options for UTF-8 BOM stripping, header-key case
//! sensitivity, inline `BEGIN IONS KEY=VALUE` headers, peak line validation,
//! and unterminated-EOF block handling.
//!
//! # Example
//!
//! ```rust
//! use mgf::{MgfLineParser, MgfOptions};
//!
//! let mut parser = MgfLineParser::new(MgfOptions::standard());
//! for line in ["BEGIN IONS", "PEPMASS=400.25 1000.0", "150.1 50.0", "END IONS"] {
//!     if let Some(block) = parser.push_line(line) {
//!         assert_eq!(block.header.effective_precursor_mz(), Some(400.25));
//!         assert_eq!(block.peaks.len(), 1);
//!     }
//! }
//! assert!(parser.finish().is_none());
//! ```

#![warn(missing_docs)]

mod error;
mod parser;
mod types;

pub use error::MgfError;
pub use parser::{
    MgfLineParser, extract_pepmass_from_line, is_ions_marker, parse_leading_float, parse_mgf_lines,
    parse_peak_line,
};
pub use types::{
    HeaderCase, MgfBlock, MgfHeader, MgfOptions, MgfPeak, PeakParseMode, UnterminatedBlockPolicy,
};

#[cfg(test)]
mod tests;
