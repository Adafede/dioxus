// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Calibration math applied per fragment using the precursor discrepancy.

use super::types::{CalibrationModel, Peak};

#[must_use]
pub fn recalibrate_fragment_mz(
    mz: f64,
    delta: f64,
    pepmass: f64,
    lambda: f64,
    calibration_model: CalibrationModel,
) -> f64 {
    match calibration_model {
        CalibrationModel::TOFDa { .. } => lambda.mul_add(-delta, mz),
        CalibrationModel::OrbitrapPPM { .. } => {
            let dppm = delta * 1e6 / pepmass;
            mz * (1.0 - lambda * dppm / 1e6)
        }
        CalibrationModel::None => mz,
    }
}

/// Applies Orbitrap-style (ppm) correction to fragments.
pub fn apply_orbitrap_correction(
    fragments: &mut [Peak],
    precursor_ms1: f64,
    precursor_ms2: f64,
    lambda: f64,
) {
    // Compute error in ppm
    let delta_ppm = 1e6 * (precursor_ms2 - precursor_ms1) / precursor_ms1;

    // Skip if error is negligible (< 1e-7 ppm, i.e., < 1e-13 in relative terms)
    if delta_ppm.abs() < 1e-7 {
        return;
    }

    // Clamp lambda to [0, 1] for safety
    let lambda_clamped = lambda.clamp(0.0, 1.0);

    // Compute the fractional correction factor: (1 - λ * delta_ppm / 1e6)
    let correction_factor = 1.0 - lambda_clamped * delta_ppm / 1e6;

    // Apply correction to each fragment
    for peak in fragments {
        if peak.mz.is_finite() && peak.mz > 0.0 {
            peak.mz *= correction_factor;
        }
    }
}

/// Applies TOF-style (absolute Da) correction to fragments.
pub fn apply_tof_correction(
    fragments: &mut [Peak],
    precursor_ms1: f64,
    precursor_ms2: f64,
    lambda: f64,
) {
    // Compute error in Da
    let delta_da = precursor_ms2 - precursor_ms1;

    // Skip if error is negligible (< 1e-14 Da)
    if delta_da.abs() < 1e-14 {
        return;
    }

    // Clamp lambda to [0, 1] for safety
    let lambda_clamped = lambda.clamp(0.0, 1.0);

    // Compute the correction: λ * delta_da
    let correction = lambda_clamped * delta_da;

    // Apply correction to each fragment
    for peak in fragments {
        if peak.mz.is_finite() {
            peak.mz -= correction;
        }
    }
}
