// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the mgf-precursor-erro-rs project

//! Precursor error diagnostics before and after MS2 fragment recalibration.

pub(crate) mod model;
pub(crate) mod stats;

use model::{RecalibrationMeasurement, RecalibrationStats};

use crate::metrics::stats::PrecursorStats;
use crate::numeric::usize_to_f64;
use crate::recalibration::calibration::recalibrate_fragment_mz;
use crate::recalibration::types::CalibrationModel;

pub(crate) fn compute_mean(values: &[f64]) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    values.iter().sum::<f64>() / usize_to_f64(values.len())
}

pub(crate) fn compute_rms(values: &[f64]) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    let sum_sq: f64 = values.iter().map(|v| v * v).sum();
    (sum_sq / usize_to_f64(values.len())).sqrt()
}

pub(crate) fn compute_max_abs(values: &[f64]) -> f64 {
    values.iter().map(|v| v.abs()).fold(0.0, f64::max)
}

/// Computes preview recalibration diagnostics from accumulated [`PrecursorStats`] and `model`.
///
/// Returns `None` when `model` is [`CalibrationModel::None`].
#[must_use]
pub fn compute_recalibration_diagnostics(
    metrics: &PrecursorStats,
    model: CalibrationModel,
) -> Option<RecalibrationStats> {
    if !model.is_active() {
        return None;
    }

    let ppm = |delta: f64, reference: f64| {
        if reference > 0.0 {
            delta * 1e6 / reference
        } else {
            0.0
        }
    };

    let mut diag = RecalibrationStats::new();
    for point in &metrics.plot_points {
        let Some(theoretical_mass) = point.expected_mass else {
            continue;
        };

        let precursor_ms2 = point.ms2_precursor_peak.unwrap_or(point.pepmass_header);
        let precursor_ms1 = point.pepmass_header;
        let error_da_ms1 = precursor_ms1 - theoretical_mass;
        let error_ppm_ms1 = ppm(error_da_ms1, theoretical_mass);

        let error_da_before = precursor_ms2 - theoretical_mass;
        let error_ppm_before = ppm(error_da_before, theoretical_mass);

        let delta_da_ms2_ms1 = precursor_ms2 - precursor_ms1;
        let delta_ppm_ms2_ms1 = ppm(delta_da_ms2_ms1, precursor_ms1);

        let precursor_ms2_after =
            recalibrate_fragment_mz(precursor_ms2, delta_da_ms2_ms1, precursor_ms1, model);

        let error_da_after = precursor_ms2_after - theoretical_mass;
        let error_ppm_after = ppm(error_da_after, theoretical_mass);

        diag.push_measurement(RecalibrationMeasurement {
            error_ppm_ms1,
            delta_ppm_ms2_ms1,
            error_ppm_before,
            error_ppm_after,
            error_da_ms1,
            delta_da_ms2_ms1,
            error_da_before,
            error_da_after,
            precursor_ms1,
            precursor_ms2_before: precursor_ms2,
            precursor_ms2_after,
            adduct_family: None,
            max_samples: 5000,
        });
    }

    diag.compute_statistics();
    Some(diag)
}
