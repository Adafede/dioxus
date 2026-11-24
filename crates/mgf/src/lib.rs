// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Streaming MGF (`Mascot Generic Format`) block parser and spectrum model.
//!
//! Provides `BEGIN IONS ... END IONS` line-by-line parsing with UTF-8 BOM
//! stripping, case-insensitive header keys, positive-m/z peak validation, and
//! dropping of unclosed EOF blocks.
//!
//! # Example
//!
//! ```rust
//! use mgf::MgfLineParser;
//!
//! let mut parser = MgfLineParser::new();
//! let mut blocks = Vec::new();
//! for line in ["BEGIN IONS", "PEPMASS=400.25 1000.0", "150.1 50.0", "END IONS"] {
//!     if let Some(block) = parser.push_line(line) {
//!         blocks.push(block);
//!     }
//! }
//! assert_eq!(blocks.len(), 1);
//! assert_eq!(blocks[0].header.effective_precursor_mz(), Some(400.25));
//! assert_eq!(blocks[0].peaks.len(), 1);
//! ```

#![warn(missing_docs)]

mod parser;
mod types;

pub use parser::{MgfLineParser, extract_pepmass_from_line, is_ions_marker, parse_leading_float};
pub use types::{MgfBlock, MgfHeader, MgfPeak};

#[cfg(test)]
mod tests;
