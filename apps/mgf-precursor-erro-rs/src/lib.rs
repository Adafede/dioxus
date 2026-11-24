// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the mgf-precursor-erro-rs project

//! `mgf-precursor-erro-rs` — MGF precursor mass-error analysis.
//!
//! Thin Dioxus WebAssembly UI and SVG plotting adapter over [`mgf_analysis_core`],
//! streaming MGF spectra, recalibrating precursor and fragment *m/z* values, and
//! rendering interactive mass-error diagnostics.
//!
//! # Run locally
//!
//! ```bash
//! dx serve --package mgf-precursor-erro-rs
//! ```
//!
//! # Build for the website
//!
//! ```bash
//! dx build --profile wasm-release --platform web --package mgf-precursor-erro-rs
//! ```

pub(crate) mod app;
pub(crate) mod errors;
/// SVG plotting renderers consumed by [`app()`] and `examples/recalibration_demo.rs`.
pub mod plotting;

pub use app::app;
/// Error type surfaced in plotting renders.
pub use errors::PlotError;
