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
//! use smellfish_core::csv::{format_csv_rows, parse_csv_rows};
//! use smellfish_core::model::{EnrichmentOutcome, RdkitInspectResponse};
//! use smellfish_core::pipeline::{RowInspection, finalize_pass2, prepare_pass1};
//!
//! let rows = parse_csv_rows("name,smiles\ncaffeine,CN1C=NC2=C1C(=O)N(C(=O)N2C)C\n").unwrap();
//! assert_eq!(rows[0].label, "caffeine");
//!
//! let batch = prepare_pass1(vec![RowInspection {
//!     raw: rows.into_iter().next().unwrap(),
//!     outcome: Ok(RdkitInspectResponse::default()),
//! }]);
//! let outcome = finalize_pass2(batch, EnrichmentOutcome::default());
//! assert!(format_csv_rows(&outcome.rows).contains("caffeine"));
//! ```

#![warn(missing_docs)]

pub mod csv;
mod error;
mod evidence;
pub mod model;
pub mod pipeline;
pub mod qlever;
mod query;
mod sparql;
mod svg;

#[cfg(test)]
mod tests;

pub use error::SmellfishError;
