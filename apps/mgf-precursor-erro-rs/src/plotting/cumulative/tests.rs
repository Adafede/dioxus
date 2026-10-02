// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the mgf-precursor-erro-rs project

// The `tests` tests, extracted from `cumulative.rs`.
//
// They were inline, and together they were several times the size of the
// code they cover. `super` is still the `cumulative` module, so
// `use super::*` below reaches exactly what it did before the move.
use super::*;

// ── render_error_quartet ──────────────────────────────────────────

#[test]
fn render_error_quartet_empty_arrays_returns_ok_empty() {
    let result = render_error_quartet(&[], &[], &[], &[]);
    assert!(
        result.is_ok(),
        "empty quartet input should return Ok, not panic"
    );
    assert_eq!(result.unwrap(), "");
}

#[test]
fn render_error_quartet_all_nan_returns_ok_empty() {
    let result = render_error_quartet(&[f64::NAN], &[f64::NAN], &[f64::NAN], &[f64::NAN]);
    assert!(
        result.is_ok(),
        "all-NaN quartet should return Ok empty, not panic"
    );
    assert_eq!(result.unwrap(), "");
}

#[test]
fn render_error_quartet_single_value_returns_ok_with_svg() {
    let result = render_error_quartet(&[1.0], &[2.0], &[0.5], &[1.0]);
    assert!(
        result.is_ok(),
        "single-value quartet should produce SVG, got {result:?}"
    );
}

// ── render_cumulative_error_curves ────────────────────────────────

#[test]
fn render_cumulative_error_curves_empty_arrays_returns_ok_empty() {
    let result = render_cumulative_error_curves(&[], &[]);
    assert!(
        result.is_ok(),
        "empty curves input should return Ok, not panic"
    );
    assert_eq!(result.unwrap(), "");
}

#[test]
fn render_cumulative_error_curves_one_empty_returns_ok_empty() {
    let result = render_cumulative_error_curves(&[1.0, 2.0], &[]);
    assert!(
        result.is_ok(),
        "one-empty curves input should return Ok empty, not panic"
    );
    assert_eq!(result.unwrap(), "");
}

#[test]
fn render_cumulative_error_curves_all_nan_returns_ok_empty() {
    let result = render_cumulative_error_curves(&[f64::NAN], &[f64::NAN]);
    assert!(
        result.is_ok(),
        "all-NaN curves should return Ok empty, not panic"
    );
    assert_eq!(result.unwrap(), "");
}

#[test]
fn render_cumulative_error_curves_single_point_returns_ok_with_svg() {
    let result = render_cumulative_error_curves(&[1.0], &[0.5]);
    assert!(
        result.is_ok(),
        "single-point curves should produce SVG, got {result:?}"
    );
}

// ── render_cumulative_error_three_curves ─────────────────────────

#[test]
fn render_cumulative_error_three_curves_empty_arrays_returns_ok_empty() {
    let result = render_cumulative_error_three_curves(&[], &[], &[], "ppm", vec![]);
    assert!(
        result.is_ok(),
        "empty three-curves input should return Ok, not panic"
    );
    assert_eq!(result.unwrap(), "");
}

#[test]
fn render_cumulative_error_three_curves_all_nan_returns_ok_empty() {
    let result =
        render_cumulative_error_three_curves(&[f64::NAN], &[f64::NAN], &[f64::NAN], "ppm", vec![]);
    assert!(
        result.is_ok(),
        "all-NaN three-curves should return Ok empty, not panic"
    );
    assert_eq!(result.unwrap(), "");
}

#[test]
fn render_cumulative_error_three_curves_single_point_returns_ok_with_svg() {
    let result = render_cumulative_error_three_curves(&[1.0], &[2.0], &[0.5], "ppm", vec![5.0]);
    assert!(
        result.is_ok(),
        "single-point three-curves should produce SVG, got {result:?}"
    );
}

// ── internal helpers ─────────────────────────────────────────────

#[test]
fn cumulative_points_empty_returns_empty_vec() {
    let result = cumulative_points(&[]);
    assert_eq!(result, Vec::new());
}

#[test]
fn cumulative_points_filters_non_finite() {
    let result = cumulative_points(&[1.0, f64::NAN, 2.0, f64::INFINITY]);
    // After filtering NaN/Inf: [1.0, 2.0] → cumulative percentages:
    // (1.0, 50.0), (2.0, 100.0)
    assert_eq!(result.len(), 2);
    assert_eq!(result[0], (1.0, 50.0));
    assert_eq!(result[1], (2.0, 100.0));
}

#[test]
fn append_cumulative_legend_no_svg_tag_is_noop() {
    let input = "no svg here";
    let result = append_cumulative_legend(input);
    assert_eq!(result, input);
}
