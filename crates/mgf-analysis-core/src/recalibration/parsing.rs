// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the mgf-precursor-erro-rs project

//! MGF fragment and precursor-peak line helpers for recalibration.

/// Extracts a leading float from a `PEPMASS=` or `PRECURSOR_MZ=` header line.
#[must_use]
pub fn extract_pepmass_from_line(line: &str) -> Option<f64> {
    mgf::extract_pepmass_from_line(line)
}

pub(super) fn parse_fragment_prefix(line: &str) -> Option<(f64, f64)> {
    let mut fields = line.split_whitespace();
    let mz = fields.next()?.parse::<f64>().ok()?;
    let intensity = fields.next()?.parse::<f64>().ok()?;
    Some((mz, intensity))
}

/// Returns `true` when `line` begins with two parseable floating-point columns (`mz intensity`).
#[must_use]
pub fn is_fragment_line(line: &str) -> bool {
    parse_fragment_prefix(line).is_some()
}

/// Finds the fragment peak closest to `pepmass` within `0.02 Da` and `100 ppm`.
pub fn closest_ms2_precursor_mz(mzs: impl IntoIterator<Item = f64>, pepmass: f64) -> Option<f64> {
    if !pepmass.is_finite() || pepmass <= 0.0 {
        return None;
    }
    let mut best_mz: Option<f64> = None;
    let mut best_delta = f64::INFINITY;
    for mz in mzs {
        let da = (mz - pepmass).abs();
        let ppm = da * 1e6 / pepmass;
        if da <= 0.02 && ppm <= 100.0 && da < best_delta {
            best_delta = da;
            best_mz = Some(mz);
        }
    }
    best_mz
}

/// Locates the MS2 precursor peak among formatted fragment lines.
#[must_use]
pub fn find_ms2_precursor_peak<S: AsRef<str>>(spectrum_frags: &[S], pepmass: f64) -> Option<f64> {
    closest_ms2_precursor_mz(
        spectrum_frags
            .iter()
            .filter_map(|frag| frag.as_ref().split_whitespace().next()?.parse::<f64>().ok()),
        pepmass,
    )
}
