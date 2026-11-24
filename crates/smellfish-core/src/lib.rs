// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the smellfish-rs project

//! Pure natural-product originality assessment, CSV I/O, SPARQL queries, and
//! Pass-1/Pass-2 dataset pipeline for `smellfish-rs`.
//!
//! All domain logic in this crate compiles for both native hosts and
//! `wasm32-unknown-unknown` with zero Dioxus or browser dependencies, using
//! [`lotus_search::Http`] for transport-agnostic SPARQL queries.
//!
//! # Example
//!
//! ```
//! use smellfish_core::csv::parse_csv_rows;
//! use smellfish_core::evidence::{EvidenceCounts, EvidenceSignals, VerdictCategory, classify_np_evidence};
//!
//! let rows = parse_csv_rows("name,smiles\ncaffeine,CN1C=NC2=C1C(=O)N(C(=O)N2C)C\n").unwrap();
//! assert_eq!(rows[0].label, "caffeine");
//!
//! let verdict = classify_np_evidence(&EvidenceSignals {
//!     np_score: Some(2.6),
//!     has_lotus: true,
//!     has_pubchem: true,
//!     lotus_scaffolds: 0,
//!     counts: EvidenceCounts {
//!         np_core: 2,
//!         scaffold: 1,
//!         natural: 3,
//!         ..EvidenceCounts::default()
//!     },
//! });
//! assert_eq!(verdict.category, VerdictCategory::Likely);
//! ```

#![warn(missing_docs)]

pub mod csv;
mod error;
pub mod evidence;
pub mod model;
pub mod pipeline;
pub mod qlever;
pub mod query;
pub mod sparql;
mod svg;

#[cfg(test)]
mod tests;

pub use error::SmellfishError;
pub use svg::{is_safe_svg, sanitize_svg};
