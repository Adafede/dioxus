// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the mgf-precursor-erro-rs project

use super::assert_close;
use crate::diagnostics::model::RecalibrationMeasurement;
use crate::diagnostics::{compute_max_abs, compute_mean, compute_rms};
use crate::metrics::ErrorMeasurement;
use crate::{
    AdductFamily, CalibrationModel, PrecursorStats, RecalibrationStats,
    compute_recalibration_diagnostics,
};

#[test]
fn diagnostics_accumulator_mean_rms_max_and_improvements() {
    let mut diag = RecalibrationStats::new();
    assert_eq!(diag.sample_count, 0);
    assert_eq!(diag.total_count, 0);

    diag.push_error(10.0, 5.0, 0.05, 0.025, Some("protonated"), 100);
    diag.push_error(20.0, 10.0, 0.1, 0.05, Some("deprotonated"), 100);
    diag.compute_statistics();

    assert_close(diag.mean_error_ppm_before, 15.0, 1e-6, "mean before");
    assert_close(diag.mean_error_ppm_after, 7.5, 1e-6, "mean after");
    assert!(diag.mean_error_improvement_ppm() > 0.0);
    assert_eq!(diag.error_by_adduct_before.len(), 2);

    assert_close(compute_mean(&[1.0, 2.0, 3.0, 4.0, 5.0]), 3.0, 1e-10, "mean");
    assert_close(
        compute_rms(&[1.0, 2.0, 3.0]),
        ((14.0_f64) / 3.0).sqrt(),
        1e-10,
        "rms",
    );
    assert_close(
        compute_max_abs(&[1.0, -5.0, 3.0, -2.0]),
        5.0,
        1e-10,
        "max_abs",
    );
}

#[test]
fn push_measurement_keeps_parallel_series_synchronized_and_tracks_stream_max() {
    let mut diag = RecalibrationStats::new();
    for i in 1..=25u32 {
        let v = f64::from(i);
        diag.push_measurement(RecalibrationMeasurement {
            error_ppm_ms1: v,
            error_ppm_before: if i == 1 { 250.0 } else { v * 3.0 },
            error_ppm_after: v * 4.0,
            error_da_ms1: v * 5.0,
            error_da_before: v * 7.0,
            error_da_after: v * 8.0,
            adduct_family: Some("protonated"),
            max_samples: 5,
        });
    }
    diag.compute_statistics();
    assert_eq!(diag.sample_count, 5);
    assert_eq!(diag.total_count, 25);
    assert_close(diag.max_abs_error_ppm_before, 250.0, 1e-9, "stream max");
    for idx in 0..diag.sample_count {
        let base = diag.error_ppm_ms1[idx];
        assert_close(diag.error_da_ms1[idx], base * 5.0, 1e-9, "da_ms1");
        assert_close(diag.error_da_before[idx], base * 7.0, 1e-9, "da_before");
        assert_close(diag.error_da_after[idx], base * 8.0, 1e-9, "da_after");
    }
}

fn sample_measurement(
    pepmass_header: f64,
    ms2_precursor_peak: Option<f64>,
    calculated_mass: Option<f64>,
    expected_mass: Option<f64>,
) -> ErrorMeasurement<'static> {
    ErrorMeasurement {
        abs_error_da: 0.01,
        abs_ppm: 20.0,
        adduct_family: AdductFamily::Protonated,
        ppm_error: 20.0,
        signed_error_da: 0.01,
        pepmass_header,
        ms2_precursor_peak,
        smiles: None,
        calculated_mass,
        expected_mass,
        formula: None,
    }
}

#[test]
fn compute_recalibration_diagnostics_pins_none_and_tof_improvement() {
    let mut metrics = PrecursorStats::default();
    metrics.record_error(sample_measurement(
        500.0,
        Some(500.01),
        Some(500.0),
        Some(500.0),
    ));
    metrics.record_error(sample_measurement(600.0, Some(600.01), None, None));

    assert!(compute_recalibration_diagnostics(&metrics, CalibrationModel::None).is_none());

    let diag = compute_recalibration_diagnostics(&metrics, CalibrationModel::tof(1.0)).unwrap();
    assert_eq!(diag.total_count, 1);
    assert_eq!(
        diag.error_by_adduct_before.get("Protonated").map(Vec::len),
        Some(1)
    );
    assert_close(diag.mae_error_ppm_before, 20.0, 1e-6, "mae before");
    assert_close(diag.mae_error_ppm_after, 0.0, 1e-9, "mae after");
    assert_close(diag.mean_error_improvement_ppm(), 20.0, 1e-6, "improvement");

    let mut no_ms2 = PrecursorStats::default();
    no_ms2.record_error(sample_measurement(500.0, None, Some(500.0), Some(500.0)));
    let diag_no_ms2 =
        compute_recalibration_diagnostics(&no_ms2, CalibrationModel::tof(1.0)).unwrap();
    assert_eq!(
        diag_no_ms2.total_count, 0,
        "spectra without MS2 precursor peak must not fabricate zero delta"
    );
}
