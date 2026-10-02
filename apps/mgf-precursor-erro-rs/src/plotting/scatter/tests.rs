// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the mgf-precursor-erro-rs project

// The `tests` tests, extracted from `scatter.rs`.
//
// They were inline, and together they were several times the size of the
// code they cover. `super` is still the `scatter` module, so
// `use super::*` below reaches exactly what it did before the move.
use super::*;
use crate::errors::MgfError;
use crate::metrics::{AdductFamily, PlotPoint};

fn sample_point(err_da: f64, err_ppm: f64) -> PlotPoint {
    PlotPoint {
        adduct_family: AdductFamily::Protonated,
        pepmass_header: 500.0,
        ms2_precursor_peak: Some(500.0),
        signed_error_da: err_da,
        signed_error_ppm: err_ppm,
        expected_mass: Some(500.0),
    }
}

// ── render_ecdf_svg ───────────────────────────────────────────────

#[test]
fn render_ecdf_svg_empty_values_returns_ok() {
    let result = render_ecdf_svg("test", &[], &[], "ppm");
    assert!(
        result.is_ok(),
        "empty ECDF input should be handled gracefully, got {result:?}"
    );
}

#[test]
fn render_ecdf_svg_nan_and_inf_values_returns_ok() {
    let values = [f64::NAN, f64::INFINITY, f64::NEG_INFINITY];
    let result = render_ecdf_svg("test", &values, &[1.0, 5.0, 10.0], "ppm");
    assert!(
        result.is_ok(),
        "NaN/inf values should be filtered, not panic, got {result:?}"
    );
}

#[test]
fn render_ecdf_svg_single_value_returns_ok() {
    let result = render_ecdf_svg("test", &[5.0], &[1.0, 5.0, 10.0], "ppm");
    assert!(result.is_ok(), "single ECDF value should not panic");
}

// ── render_mass_bias_svg ──────────────────────────────────────────

#[test]
fn render_mass_bias_svg_empty_points_returns_ok_or_err() {
    let result = render_mass_bias_svg("test", &[]);
    match &result {
        Ok(svg) => assert!(
            svg.contains("<svg"),
            "empty input should still produce SVG canvas"
        ),
        Err(MgfError::Drawing(_)) => {
            // A zero-width y-range (no finite data) can cause plotters to
            // return InvalidRange, which is correctly propagated as Err.
        }
    }
}

#[test]
fn render_mass_bias_svg_all_non_finite_y_does_not_panic() {
    // Every point has a non-finite signed_error_da, so prepare_scatter_plot_data
    // produces an empty series with y_limit = 0.0, creating a zero-width
    // y-range (0.0..0.0).  Plotters' SVG backend is lenient and accepts
    // the degenerate range, producing a (degenerate) SVG rather than
    // panicking or returning Err.  The pre-#4 code would have had a bare
    // `.unwrap()` here that could panic; the hardened `?` path and early
    // guards ensure graceful handling.
    let points = vec![
        sample_point(f64::NAN, f64::NAN),
        sample_point(f64::INFINITY, f64::INFINITY),
        sample_point(f64::NEG_INFINITY, f64::NEG_INFINITY),
    ];
    let result = render_mass_bias_svg("test", &points);
    match &result {
        Ok(svg) => assert!(
            svg.contains("<svg"),
            "degenerate range should still produce SVG canvas"
        ),
        Err(MgfError::Drawing(_)) => {} // also acceptable: typed error, no panic
    }
}

// ── render_absolute_mass_bias_svg ─────────────────────────────────

#[test]
fn render_absolute_mass_bias_svg_empty_points_returns_ok_or_err() {
    let result = render_absolute_mass_bias_svg("test", &[], "ppm", &[]);
    match &result {
        Ok(svg) => assert!(
            svg.contains("<svg"),
            "empty input should still produce SVG canvas"
        ),
        Err(MgfError::Drawing(_)) => {}
    }
}

#[test]
fn render_absolute_mass_bias_svg_all_non_finite_y_does_not_panic() {
    // All points have non-finite errors, producing a zero-width y-range.
    // As with render_mass_bias_svg, the hardened path must not panic.
    let points = vec![
        sample_point(f64::NAN, f64::NAN),
        sample_point(f64::INFINITY, f64::INFINITY),
    ];
    let result = render_absolute_mass_bias_svg("test", &points, "ppm", &[1.0, 5.0]);
    match &result {
        Ok(svg) => assert!(
            svg.contains("<svg"),
            "degenerate range should still produce SVG canvas"
        ),
        Err(MgfError::Drawing(_)) => {}
    }
}

#[test]
fn render_absolute_mass_bias_svg_valid_points_returns_ok() {
    let points = vec![
        sample_point(0.5, 1.0),
        sample_point(-0.3, -0.6),
        sample_point(1.2, 2.4),
    ];
    let result = render_absolute_mass_bias_svg("test", &points, "ppm", &[1.0, 2.0]);
    assert!(
        result.is_ok(),
        "valid points should produce SVG, got {result:?}"
    );
}
