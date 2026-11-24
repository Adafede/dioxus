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
    let svg = render_recalibration_diagnostic_ppm(&[1.0, -2.0, 3.0], &[0.5, -0.3]).unwrap();
    assert!(svg.contains("<svg"), "expected SVG root element");
    assert!(
        svg.contains("<polyline") || svg.contains("<path"),
        "expected drawn plot lines"
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
    let svg = render_recalibration_diagnostic_histogram(&[1.0], &[], 1).unwrap();
    assert!(svg.contains("<svg"), "expected SVG root element");
    assert!(svg.contains("<rect"), "expected histogram bar <rect>");
}

#[test]
fn fill_histogram_populates_first_middle_and_final_bins() {
    let mut bins = vec![0usize; 4];
    fill_histogram(&mut bins, &[0.0, 1.0, 2.5, 3.0, 4.0], 0.0, 4.0, 4);
    assert_eq!(
        bins,
        vec![1, 1, 1, 2],
        "all four bins including max endpoint must be populated"
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
    let svg = render_recalibration_diagnostic_mz_comparison(
        &[100.0, 200.0, 300.0],
        &[102.0, 198.0, 299.0],
        &[100.5, 199.5, 299.5],
    )
    .unwrap();
    assert!(svg.contains("<svg"), "expected SVG root element");
    assert!(svg.contains("<circle"), "expected scatter point <circle>");
}

#[test]
fn render_diagnostic_mz_comparison_downsamples_large_inputs() {
    let n = 4000usize;
    let ms1: Vec<f64> = (0..n)
        .map(|i| 200.0 + f64::from(u32::try_from(i).unwrap()))
        .collect();
    let ms2_before: Vec<f64> = ms1.iter().map(|v| v + 0.02).collect();
    let ms2_after: Vec<f64> = ms1.iter().map(|v| v + 0.001).collect();
    let svg = render_recalibration_diagnostic_mz_comparison(&ms1, &ms2_before, &ms2_after).unwrap();
    let circles = svg.matches("<circle").count();
    assert!(
        circles <= 3_100,
        "expected downsampled scatter circles (<= ~3000), got {circles}"
    );
}

#[test]
fn render_histogram_uniform_dataset_produces_non_degenerate_bars() {
    let svg = render_recalibration_diagnostic_histogram(&[5.0, 5.0, 5.0], &[5.0], 10).unwrap();
    assert!(svg.contains("<svg"), "expected SVG root element");
    assert!(
        svg.contains("<rect"),
        "uniform error values must render histogram bars without collapsing to 0 span"
    );
    assert!(
        !svg.contains(r#"width="0""#),
        "uniform histogram bars must have non-zero width: {svg}"
    );
    assert!(!svg.contains("NaN"), "histogram SVG must not contain NaN");
}
