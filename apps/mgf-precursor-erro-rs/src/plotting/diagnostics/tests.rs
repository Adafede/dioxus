// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the mgf-precursor-erro-rs project

// The `tests` tests, extracted from `diagnostics.rs`.
//
// They were inline, and together they were several times the size of the
// code they cover. `super` is still the `diagnostics` module, so
// `use super::*` below reaches exactly what it did before the move.
use super::*;

// ── render_recalibration_diagnostic_ppm ─────────────────────────

#[test]
fn render_ppm_empty_arrays_returns_ok_empty() {
    let result = render_recalibration_diagnostic_ppm(&[], &[]);
    assert!(
        result.is_ok(),
        "empty ppm input should return Ok, not panic"
    );
    assert_eq!(result.unwrap(), "");
}

#[test]
fn render_ppm_all_nan_returns_ok_empty() {
    let result = render_recalibration_diagnostic_ppm(&[f64::NAN], &[f64::NAN]);
    assert!(
        result.is_ok(),
        "NaN ppm values should be filtered, returning empty buffer, not panic"
    );
    assert_eq!(result.unwrap(), "");
}

#[test]
fn render_ppm_valid_values_returns_ok_with_svg() {
    let result = render_recalibration_diagnostic_ppm(&[1.0, -2.0, 3.0], &[0.5, -0.3]);
    assert!(
        result.is_ok(),
        "valid ppm values should produce SVG, got {result:?}"
    );
}

#[test]
fn render_ppm_inf_values_filtered_returns_ok_with_svg() {
    let result = render_recalibration_diagnostic_ppm(
        &[1.0, f64::INFINITY, 3.0],
        &[0.5, f64::NEG_INFINITY, -0.3],
    );
    assert!(
        result.is_ok(),
        "inf values should be filtered, not panic, got {result:?}"
    );
}

// ── render_recalibration_diagnostic_histogram ───────────────────

#[test]
fn render_histogram_empty_arrays_returns_ok_empty() {
    let result = render_recalibration_diagnostic_histogram(&[], &[], 10);
    assert!(
        result.is_ok(),
        "empty histogram input should return Ok, not panic"
    );
    assert_eq!(result.unwrap(), "");
}

#[test]
fn render_histogram_zero_bin_count_returns_ok_empty() {
    // bin_count == 0 would produce NaN rectangle coordinates; the guard
    // returns an empty buffer instead of panicking.
    let result = render_recalibration_diagnostic_histogram(&[1.0, 2.0], &[3.0], 0);
    assert!(
        result.is_ok(),
        "zero bin_count should be guarded, returning empty buffer, not panic"
    );
    assert_eq!(result.unwrap(), "");
}

#[test]
fn render_histogram_all_nan_returns_ok_empty() {
    let result = render_recalibration_diagnostic_histogram(&[f64::NAN], &[f64::NAN], 5);
    assert!(result.is_ok(), "all-NaN histogram should return Ok empty");
    assert_eq!(result.unwrap(), "");
}

#[test]
fn render_histogram_single_bin_single_value_returns_ok() {
    let result = render_recalibration_diagnostic_histogram(&[1.0], &[], 1);
    assert!(
        result.is_ok(),
        "single-value histogram should produce SVG, got {result:?}"
    );
}

// ── render_recalibration_diagnostic_mz_comparison ────────────────

#[test]
fn render_mz_comparison_empty_arrays_returns_ok_empty() {
    let result = render_recalibration_diagnostic_mz_comparison(&[], &[], &[]);
    assert!(
        result.is_ok(),
        "empty mz comparison input should return Ok, not panic"
    );
    assert_eq!(result.unwrap(), "");
}

#[test]
fn render_mz_comparison_only_ms1_returns_ok_empty() {
    // ms2_before is empty → guard returns early
    let result = render_recalibration_diagnostic_mz_comparison(&[100.0], &[], &[]);
    assert!(result.is_ok(), "missing ms2_before should return Ok empty");
    assert_eq!(result.unwrap(), "");
}

#[test]
fn render_mz_comparison_valid_values_returns_ok_with_svg() {
    let result = render_recalibration_diagnostic_mz_comparison(
        &[100.0, 200.0, 300.0],
        &[102.0, 198.0, 299.0],
        &[100.5, 199.5, 299.5],
    );
    assert!(
        result.is_ok(),
        "valid mz comparison should produce SVG, got {result:?}"
    );
}

// ── render_recalibration_summary_text ────────────────────────────

#[test]
fn render_summary_text_with_nan_does_not_panic() {
    // NaN values propagate through arithmetic but format! never panics.
    let result = render_recalibration_summary_text(f64::NAN, 1.0, 2.0, 0.5, 10.0, 3.0);
    assert!(
        result.is_ok(),
        "summary text with NaN should return Ok, not panic, got {result:?}"
    );
    let html = result.unwrap();
    assert!(html.contains("Recalibration Summary"));
}

#[test]
fn render_summary_text_normal_values() {
    let result = render_recalibration_summary_text(5.0, 2.0, 8.0, 1.0, 12.0, 3.0);
    assert!(result.is_ok());
    let html = result.unwrap();
    assert!(html.contains("Recalibration Summary"));
}
