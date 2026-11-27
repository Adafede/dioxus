// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the mgf-precursor-erro-rs project

use super::*;
use mgf_analysis_core::{AdductFamily, ErrorUnit, PlotPoint};

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
    let result = render_ecdf_svg("test", &[], &[], ErrorUnit::Ppm);
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
    let result = render_ecdf_svg("test", &values, &[1.0, 5.0, 10.0], ErrorUnit::Ppm);
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
    let result = render_ecdf_svg("test", &[5.0], &[1.0, 5.0, 10.0], ErrorUnit::Ppm);
    assert!(result.is_ok(), "single ECDF value should not panic");
}

// ── render_absolute_mass_bias_svg ─────────────────────────────────

#[test]
fn render_absolute_mass_bias_svg_empty_points_returns_ok() -> Result<(), crate::errors::PlotError> {
    let svg = render_absolute_mass_bias_svg("test", &[], ErrorUnit::Ppm, &[])?;
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
    let svg = render_absolute_mass_bias_svg("test", &points, ErrorUnit::Ppm, &[1.0, 5.0])?;
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
    let result = render_absolute_mass_bias_svg("test", &points, ErrorUnit::Ppm, &[1.0, 2.0]);
    assert!(
        matches!(
            &result,
            Ok(svg) if svg.contains("<svg") && svg.contains("<circle")
        ),
        "expected SVG with scatter points, got {result:?}"
    );
}

#[test]
fn plotting_data_helpers_downsample_format_and_stay_monotonic() {
    use super::super::data::{
        build_ecdf_points, display_error_value_for_point, fallback_y_limit_for_unit,
        format_threshold_value, sample_scatter_points,
    };

    let pt = sample_point(0.002, 4.0);
    assert!(
        (display_error_value_for_point(&pt, ErrorUnit::Ppm) - 4.0).abs() < 1e-12,
        "ppm selector"
    );
    assert!(
        (display_error_value_for_point(&pt, ErrorUnit::Mda) - 2.0).abs() < 1e-12,
        "mDa selector"
    );
    assert!(
        (fallback_y_limit_for_unit(ErrorUnit::Ppm) - 10.0).abs() < 1e-12,
        "ppm fallback y_limit"
    );
    assert!(
        (fallback_y_limit_for_unit(ErrorUnit::Mda) - 5.0).abs() < 1e-12,
        "mDa fallback y_limit"
    );
    assert_eq!(format_threshold_value(0.5), "0.5", "trim trailing zeros");
    assert_eq!(format_threshold_value(10.0), "10", "integer formatting");

    let raw: Vec<(f64, f64)> = (0..100).map(|i| (f64::from(i), f64::from(i))).collect();
    let down = sample_scatter_points(raw, 10);
    assert_eq!(down.len(), 10, "downsampled len");
    assert_eq!(down.first().copied(), Some((0.0, 0.0)), "first kept");
    assert_eq!(down.last().copied(), Some((99.0, 99.0)), "last kept");

    let large_vals: Vec<f64> = (1..=60_000).map(|i| f64::from(i) * 0.001).collect();
    let ecdf = build_ecdf_points(&large_vals, 0.001, 50.0);
    assert!(ecdf.len() > 100, "ecdf emitted steps");
    assert!(
        ecdf.is_sorted_by(|a, b| a.0 <= b.0 && a.1 <= b.1),
        "ecdf must be non-decreasing in both axes"
    );
}
