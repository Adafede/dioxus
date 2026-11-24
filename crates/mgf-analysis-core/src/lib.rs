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
//!     CalibrationModel, compute_recalibration_diagnostics, generate_recalibrated_mgf,
//!     ingest_lines,
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
//! let stats = ingest_lines(mgf_text.lines(), |_| {});
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
mod error;
mod ingestor;
mod mass;
mod metrics;
mod numeric;
mod recalibration;

pub use adduct::{
    AMMONIUM_MASS, AdductFamily, AdductSpec, ELECTRON_MASS, HYDROGEN_MASS, NITROGEN_MASS,
    POTASSIUM_MASS, Polarity, SODIUM_MASS, expected_precursor_mz, is_supported_adduct,
    normalize_adduct_label, parse_adduct_polarity, parse_adduct_term_mass, parse_polarity_tokens,
    parse_unbracketed_polarity,
};
pub use diagnostics::{
    RecalibrationMeasurement, RecalibrationStats, compute_recalibration_diagnostics,
};
pub use error::MgfAnalysisError;
pub use ingestor::{
    LineIngestor, MAX_RAW_RETENTION_BYTES, append_retained_line, ingest_lines, record_block,
};
pub use mass::{exact_mass_from_formula_cached, exact_mass_from_smiles_cached};
pub use metrics::{
    ErrorBucket, ErrorMeasurement, ErrorUnit, HighErrorSmilesDetail, MAX_ECDF_POINTS,
    MAX_PLOT_POINTS, MedianTracker, PlotPoint, PrecursorStats, WarningDetail,
};
pub use numeric::{OrderedF64, ReservoirSampler, floor_to_usize, reservoir_index, usize_to_f64};
pub use recalibration::{
    CalibrationModel, Peak, ShrinkageLambda, closest_ms2_precursor_mz, extract_pepmass_from_line,
    find_ms2_precursor_peak, generate_recalibrated_mgf, is_fragment_line, recalibrate_fragment_mz,
    recalibrate_fragments, write_fragments_as_is, write_recalibrated_fragments,
};

#[cfg(test)]
#[expect(clippy::unwrap_used)]
#[expect(clippy::expect_used)]
#[allow(clippy::indexing_slicing)]
mod tests;
