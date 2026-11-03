// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the mgf-precursor-erro-rs project

use dioxus::prelude::{Signal, WritableExt};

use crate::diagnostics::RecalibrationStats;
use crate::metrics::PrecursorStats;
use crate::recalibration::{CalibrationModel, recalibrate_fragment_mz};

pub(crate) fn update_recalibration_diagnostics(
    metrics: &PrecursorStats,
    model: CalibrationModel,
    diagnostics_signal: &mut Signal<Option<RecalibrationStats>>,
) {
    if matches!(model, CalibrationModel::None) {
        diagnostics_signal.set(None);
        return;
    }

    let mut diag = RecalibrationStats::new();
    let lambda = match model {
        CalibrationModel::TOFDa { lambda } | CalibrationModel::OrbitrapPPM { lambda } => lambda,
        CalibrationModel::None => 0.0,
    };

    for point in &metrics.plot_points {
        let Some(theoretical_mass) = point.expected_mass else {
            continue;
        };

        let precursor_ms2 = point.ms2_precursor_peak.unwrap_or(point.pepmass_header);
        let precursor_ms1 = point.pepmass_header;
        let error_da_ms1 = precursor_ms1 - theoretical_mass;
        let error_ppm_ms1 = if theoretical_mass > 0.0 {
            error_da_ms1 * 1e6 / theoretical_mass
        } else {
            0.0
        };

        let error_da_before = precursor_ms2 - theoretical_mass;
        let error_ppm_before = if theoretical_mass > 0.0 {
            error_da_before * 1e6 / theoretical_mass
        } else {
            0.0
        };

        let delta_ms2_ms1_da = precursor_ms2 - precursor_ms1;
        let delta_ppm_ms2_ms1 = if precursor_ms1 > 0.0 {
            delta_ms2_ms1_da * 1e6 / precursor_ms1
        } else {
            0.0
        };

        let precursor_ms2_after = recalibrate_fragment_mz(
            precursor_ms2,
            delta_ms2_ms1_da,
            precursor_ms1,
            lambda,
            model,
        );

        let error_da_after = precursor_ms2_after - theoretical_mass;
        let error_ppm_after = if theoretical_mass > 0.0 {
            error_da_after * 1e6 / theoretical_mass
        } else {
            0.0
        };

        diag.push_measurement(crate::diagnostics::RecalibrationMeasurement {
            error_ppm_ms1,
            delta_ppm_ms2_ms1,
            error_ppm_before,
            error_ppm_after,
            error_da_ms1,
            delta_da_ms2_ms1: delta_ms2_ms1_da,
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
    diagnostics_signal.set(Some(diag));
}
