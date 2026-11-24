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

pub use color::{
    adduct_family_color, adduct_family_color_hex, adduct_family_shape_style, tolerance_step_color,
    tolerance_step_rgb,
};
pub use cumulative::{
    render_cumulative_error_curves, render_cumulative_error_three_curves, render_error_quartet,
};
pub use data::{
    ScatterPlotData, build_ecdf_points, display_error_value, display_error_value_for_point,
    format_threshold_value, prepare_scatter_plot_data, sample_scatter_points,
};
pub use diagnostics::{
    render_recalibration_diagnostic_mz_comparison, render_recalibration_diagnostic_ppm,
};
pub use histogram::render_recalibration_diagnostic_histogram;
pub use scatter::{render_absolute_mass_bias_svg, render_ecdf_svg, render_mass_bias_svg};
pub use svg::make_svg_responsive;
