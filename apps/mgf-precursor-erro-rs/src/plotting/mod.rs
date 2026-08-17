// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Plotting / SVG rendering helpers for mgf-precursor-erro-rs.
//!
//! Previously a single 1477-line god-file; split by responsibility into:
//! - `color` — colour palettes + adduct-family / tolerance-step colours.
//! - `data` — numeric + data-preparation primitives (ECDF, scatter, legends).
//! - `scatter` — scatter & ECDF chart renderers.
//! - `diagnostics` — before/after-recalibration diagnostic renderers + summary.
//! - `cumulative` — four-stage error analysis + cumulative CDF curve renderers.
//!
//! The flat `crate::plotting::<name>` public API is preserved via re-exports.
//!
//! All rendering uses plotters' SVG backend, whose `DrawingError` is
//! `Infallible`; consequently every `root.fill(...)` / `.draw(...)` /
//! `root.present()` result is statically guaranteed to be `Ok`, so the
//! `.unwrap()` calls on those results are intentional and cannot panic. (The
//! two `#[cfg(test)]` unwraps in `[`diagnostics`]`'s tests follow the same
//! rule.)

pub(crate) mod color;
pub(crate) mod cumulative;
pub(crate) mod data;
pub(crate) mod diagnostics;
pub(crate) mod scatter;

pub use color::{
    adduct_family_color, adduct_family_color_hex, adduct_family_shape_style, tolerance_step_color,
    tolerance_step_rgb,
};
pub use cumulative::{
    render_cumulative_error_curves, render_cumulative_error_three_curves, render_error_quartet,
};
pub use data::{
    build_ecdf_points, display_error_value, display_error_value_for_point, format_threshold_value,
    make_svg_responsive, prepare_scatter_plot_data, sample_scatter_points,
};
pub use diagnostics::{
    render_recalibration_diagnostic_histogram, render_recalibration_diagnostic_mz_comparison,
    render_recalibration_diagnostic_ppm, render_recalibration_summary_text,
};
pub use scatter::{render_absolute_mass_bias_svg, render_ecdf_svg, render_mass_bias_svg};
