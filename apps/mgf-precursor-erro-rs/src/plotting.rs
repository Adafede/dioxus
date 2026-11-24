// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the mgf-precursor-erro-rs project

//! SVG plotting helpers for `mgf-precursor-erro-rs`.
//!
//! Consumes domain statistics from [`mgf_analysis_core`] and renders
//! responsive SVGs through `plotters` (`PlotError` on backend failure).

pub(crate) mod color;
pub(crate) mod cumulative;
pub(crate) mod data;
pub(crate) mod diagnostics;
pub(crate) mod histogram;
pub(crate) mod scatter;
pub(crate) mod svg;

pub use diagnostics::render_recalibration_diagnostic_ppm;
pub use histogram::render_recalibration_diagnostic_histogram;
