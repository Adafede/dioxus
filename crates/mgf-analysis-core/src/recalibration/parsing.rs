// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the mgf-precursor-erro-rs project

//! MGF fragment and precursor-peak line helpers for recalibration.

use crate::numeric::da_to_ppm;

pub(super) fn parse_fragment_prefix(line: &str) -> Option<(f64, f64)> {
    let mut fields = line.split_whitespace();
    let mz = fields.next()?.parse::<f64>().ok()?;
    let intensity = fields.next()?.parse::<f64>().ok()?;
    Some((mz, intensity))
}

/// Finds the fragment peak closest to `pepmass` within `0.02 Da` and `100 ppm`.
pub(crate) fn closest_ms2_precursor_mz(
    mzs: impl IntoIterator<Item = f64>,
    pepmass: f64,
) -> Option<f64> {
    if !pepmass.is_finite() || pepmass <= 0.0 {
        return None;
    }
    let mut best_mz: Option<f64> = None;
    let mut best_delta = f64::INFINITY;
    for mz in mzs {
        let da = (mz - pepmass).abs();
        let ppm = da_to_ppm(da, pepmass);
        if da <= 0.02 && ppm <= 100.0 && da < best_delta {
            best_delta = da;
            best_mz = Some(mz);
        }
    }
    best_mz
}
