// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the mgf-precursor-erro-rs project

use super::*;
use mgf_analysis_core::{AdductFamily, PlotPoint};

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
    let points = super::super::data::build_ecdf_points(&values, 1e-6, 10.0);
    assert_eq!(
        points,
        Vec::<(f64, f64)>::new(),
        "empty or all-non-finite values must not emit a fake diagonal"
    );
    let result = render_ecdf_svg("test", &values, &[1.0, 5.0, 10.0], "ppm");
    assert!(
        result.is_ok(),
        "NaN/inf values should be filtered, not panic, got {result:?}"
    );

    let small_a = 1e-20;
    let small_b = 1e-17;
    let small_pts = super::super::data::build_ecdf_points(&[small_a, small_b], 1e-22, 1.0);
    assert!(
        small_pts.len() >= 6,
        "distinct small log-scale values must not be merged by f64::EPSILON: {small_pts:?}"
    );
}

#[test]
fn render_ecdf_svg_single_value_returns_ok() {
    let result = render_ecdf_svg("test", &[5.0], &[1.0, 5.0, 10.0], "ppm");
    assert!(result.is_ok(), "single ECDF value should not panic");
}

// ── render_absolute_mass_bias_svg ─────────────────────────────────

#[test]
fn render_absolute_mass_bias_svg_empty_points_returns_ok() -> Result<(), crate::errors::PlotError> {
    let svg = render_absolute_mass_bias_svg("test", &[], "ppm", &[])?;
    assert!(
        svg.contains("<svg"),
        "empty input should produce SVG canvas"
    );
    Ok(())
}

#[test]
fn render_absolute_mass_bias_svg_all_non_finite_y_returns_ok()
-> Result<(), crate::errors::PlotError> {
    let points = vec![
        sample_point(f64::NAN, f64::NAN),
        sample_point(f64::INFINITY, f64::INFINITY),
    ];
    let svg = render_absolute_mass_bias_svg("test", &points, "ppm", &[1.0, 5.0])?;
    assert!(
        svg.contains("<svg"),
        "all-non-finite input should produce SVG canvas"
    );
    Ok(())
}

#[test]
fn render_absolute_mass_bias_svg_valid_points_returns_ok() {
    let points = vec![
        sample_point(0.5, 1.0),
        sample_point(-0.3, -0.6),
        sample_point(1.2, 2.4),
    ];
    let plot_data = super::super::data::prepare_scatter_plot_data(
        &points,
        |p| Some(p.pepmass_header),
        |p| Some(p.signed_error_ppm),
        10.0,
    );
    assert!(
        plot_data.x_min > 400.0,
        "scatter plot x_min should stay anchored near observed m/z (~500), got {}",
        plot_data.x_min
    );
    let result = render_absolute_mass_bias_svg("test", &points, "ppm", &[1.0, 2.0]);
    assert!(
        matches!(
            &result,
            Ok(svg) if svg.contains("<svg") && svg.contains("<circle")
        ),
        "expected SVG with scatter points, got {result:?}"
    );
}
