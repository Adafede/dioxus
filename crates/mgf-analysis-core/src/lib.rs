// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the mgf-precursor-erro-rs project

//! Pure-Rust MGF precursor mass-error analysis, adduct grammar, MS2 recalibration,
//! and summary diagnostics.
//!
//! Zero Dioxus, browser, or plotter dependencies: feeds the thin WASM shell and
//! native/host CLI or example runners with deterministic math and streaming
//! block ingestion backed by [`mgf`].
//!
//! # Example
//!
//! ```rust
//! use mgf_analysis_core::{
//!     CalibrationModel, LineIngestor, compute_recalibration_diagnostics, generate_recalibrated_mgf,
//! };
//!
//! let mgf_text = "\
//! BEGIN IONS
//! TITLE=caffeine_test
//! PEPMASS=195.0880
//! FORMULA=C8H10N4O2
//! ADDUCT=[M+H]+
//! 195.0980 5000.0
//! 138.0660 2500.0
//! END IONS
//! ";
//! let mut ingestor = LineIngestor::new();
//! for line in mgf_text.lines() {
//!     ingestor.feed_line(line);
//! }
//! let stats = ingestor.finish();
//! assert_eq!(stats.spectra, 1);
//!
//! let model = CalibrationModel::tof(1.0);
//! let diag = compute_recalibration_diagnostics(&stats, model).expect("active model");
//! assert_eq!(diag.total_count, 1);
//! assert!(generate_recalibrated_mgf(mgf_text, model).contains("138.055"));
//! ```

#![warn(missing_docs)]

mod adduct;
mod diagnostics;
mod ingestor;
mod mass;
mod metrics;
mod numeric;
mod recalibration;

pub use adduct::AdductFamily;
pub use diagnostics::compute_recalibration_diagnostics;
pub use diagnostics::model::RecalibrationStats;
pub use ingestor::{LineIngestor, MAX_RAW_RETENTION_BYTES, append_retained_line};
pub use metrics::stats::PrecursorStats;
pub use metrics::{ErrorBucket, ErrorUnit, HighErrorSmilesDetail, PlotPoint, WarningDetail};
pub use numeric::{floor_to_usize, usize_to_f64};
pub use recalibration::generator::{generate_recalibrated_mgf, recalibrate_fragments};
pub use recalibration::types::{CalibrationModel, Peak, ShrinkageLambda};

#[cfg(test)]
#[expect(clippy::unwrap_used)]
#[expect(clippy::expect_used)]
#[expect(
    clippy::indexing_slicing,
    reason = "tests index fixtures of asserted lengths"
)]
mod tests;
