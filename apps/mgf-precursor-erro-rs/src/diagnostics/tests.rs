// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the mgf-precursor-erro-rs project

// The `tests` tests, extracted from `diagnostics.rs`.
//
// They were inline, and together they were several times the size of the
// code they cover. `super` is still the `diagnostics` module, so
// `use super::*` below reaches exactly what it did before the move.
use super::*;

#[test]
fn test_empty_diagnostics() {
    let diag = RecalibrationStats::new();
    assert_eq!(diag.sample_count, 0);
    assert_eq!(diag.total_count, 0);
    assert!(diag.mean_error_ppm_before.abs() < f64::EPSILON);
}

#[test]
fn test_push_error() {
    let mut diag = RecalibrationStats::new();
    diag.push_error(10.0, 5.0, 0.05, 0.025, Some("protonated"), 100);

    assert_eq!(diag.total_count, 1);
    assert_eq!(diag.sample_count, 1);
    assert!((diag.error_ppm_before[0] - 10.0).abs() < f64::EPSILON);
    assert!((diag.error_ppm_after[0] - 5.0).abs() < f64::EPSILON);
}

#[test]
fn test_compute_statistics() {
    let mut diag = RecalibrationStats::new();
    diag.push_error(10.0, 5.0, 0.05, 0.025, None, 100);
    diag.push_error(20.0, 10.0, 0.1, 0.05, None, 100);
    diag.compute_statistics();

    assert!((diag.mean_error_ppm_before - 15.0).abs() < 1e-6);
    assert!((diag.mean_error_ppm_after - 7.5).abs() < 1e-6);
}

#[test]
fn test_improvement_calculation() {
    let mut diag = RecalibrationStats::new();
    diag.push_error(10.0, 5.0, 0.1, 0.05, None, 100);
    diag.push_error(10.0, 5.0, 0.1, 0.05, None, 100);
    diag.compute_statistics();

    let improvement = diag.mean_error_improvement_ppm();
    assert!(improvement > 0.0);
}

#[test]
fn test_adduct_categorization() {
    let mut diag = RecalibrationStats::new();
    diag.push_error(10.0, 5.0, 0.05, 0.025, Some("protonated"), 100);
    diag.push_error(15.0, 8.0, 0.075, 0.04, Some("deprotonated"), 100);

    assert_eq!(diag.error_by_adduct_before.len(), 2);
    assert!((diag.error_by_adduct_before["protonated"][0] - 10.0).abs() < f64::EPSILON);
    assert!((diag.error_by_adduct_before["deprotonated"][0] - 15.0).abs() < f64::EPSILON);
}

#[test]
fn test_max_samples_limit() {
    let mut diag = RecalibrationStats::new();
    for i in 0..150 {
        let error = f64::from(i) * 0.1;
        diag.push_error(error, error / 2.0, error / 100.0, error / 200.0, None, 100);
    }

    assert_eq!(diag.sample_count, 100);
    assert_eq!(diag.total_count, 150);
}

#[test]
fn test_compute_mean() {
    let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
    assert!((compute_mean(&values) - 3.0).abs() < 1e-10);
}

#[test]
fn test_compute_rms() {
    let values = vec![1.0, 2.0, 3.0];
    let expected = ((1.0 + 4.0 + 9.0) / 3.0_f64).sqrt();
    assert!((compute_rms(&values) - expected).abs() < 1e-10);
}

#[test]
fn test_compute_max_abs() {
    let values = vec![1.0, -5.0, 3.0, -2.0];
    assert!((compute_max_abs(&values) - 5.0).abs() < 1e-10);
}

#[test]
fn push_measurement_keeps_all_parallel_series_synchronized_on_reservoir_replacement() {
    let mut diag = RecalibrationStats::new();
    for i in 1..=25u32 {
        let v = f64::from(i);
        diag.push_measurement(RecalibrationMeasurement {
            error_ppm_ms1: v,
            delta_ppm_ms2_ms1: v * 2.0,
            error_ppm_before: v * 3.0,
            error_ppm_after: v * 4.0,
            error_da_ms1: v * 5.0,
            delta_da_ms2_ms1: v * 6.0,
            error_da_before: v * 7.0,
            error_da_after: v * 8.0,
            precursor_ms1: v * 100.0,
            precursor_ms2_before: v * 101.0,
            precursor_ms2_after: v * 102.0,
            adduct_family: Some("protonated"),
            max_samples: 5,
        });
    }
    assert_eq!(diag.sample_count, 5);
    for idx in 0..diag.sample_count {
        let base = diag.error_ppm_ms1[idx];
        assert!((diag.error_da_before[idx] - base * 7.0).abs() < 1e-9);
        assert!((diag.error_da_after[idx] - base * 8.0).abs() < 1e-9);
        assert!((diag.precursor_ms1_values[idx] - base * 100.0).abs() < 1e-9);
        assert!((diag.precursor_ms2_before[idx] - base * 101.0).abs() < 1e-9);
        assert!((diag.precursor_ms2_after[idx] - base * 102.0).abs() < 1e-9);
    }
}
