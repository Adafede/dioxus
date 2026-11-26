// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the mgf-precursor-erro-rs project

//! Per-fragment MS2 recalibration formulas.

use super::types::{CalibrationModel, Peak, ShrinkageLambda};

/// Applies `model` to a single fragment `mz` given precursor discrepancy `delta = ms2 - ms1`.
#[must_use]
pub(crate) fn recalibrate_fragment_mz(
    mz: f64,
    delta: f64,
    pepmass: f64,
    model: CalibrationModel,
) -> f64 {
    if !mz.is_finite() || !delta.is_finite() {
        return mz;
    }
    match model {
        CalibrationModel::TOFDa { lambda } => lambda.get().mul_add(-delta, mz),
        CalibrationModel::OrbitrapPPM { lambda } => {
            if !pepmass.is_finite() || pepmass <= 0.0 || !mz.is_finite() || mz <= 0.0 {
                return mz;
            }
            let denom = 1.0 + lambda.get() * delta / pepmass;
            if !denom.is_finite() || denom <= 0.0 {
                return mz;
            }
            mz / denom
        }
        CalibrationModel::None => mz,
    }
}

pub(super) fn apply_orbitrap_correction(
    fragments: &mut [Peak],
    precursor_ms1: f64,
    precursor_ms2: f64,
    lambda: ShrinkageLambda,
) {
    let delta = precursor_ms2 - precursor_ms1;
    let delta_ppm = 1e6 * delta / precursor_ms1;
    if delta_ppm.abs() < 1e-7 {
        return;
    }
    let model = CalibrationModel::OrbitrapPPM { lambda };
    for peak in fragments {
        peak.mz = recalibrate_fragment_mz(peak.mz, delta, precursor_ms1, model);
    }
}

pub(super) fn apply_tof_correction(
    fragments: &mut [Peak],
    precursor_ms1: f64,
    precursor_ms2: f64,
    lambda: ShrinkageLambda,
) {
    let delta_da = precursor_ms2 - precursor_ms1;
    if delta_da.abs() < 1e-14 {
        return;
    }
    let model = CalibrationModel::TOFDa { lambda };
    for peak in fragments {
        peak.mz = recalibrate_fragment_mz(peak.mz, delta_da, precursor_ms1, model);
    }
}
