// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the mgf-precursor-erro-rs project

//! MGF line parsing: pepmass directives and fragment-line detection.

#[must_use]
pub fn extract_pepmass_from_line(line: &str) -> Option<f64> {
    let trimmed = line.trim();
    if let Some((prefix, rest)) = trimmed.split_at_checked(13)
        && prefix.eq_ignore_ascii_case("PRECURSOR_MZ=")
    {
        rest.split_whitespace().next()?.parse().ok()
    } else if let Some((prefix, rest)) = trimmed.split_at_checked(8)
        && prefix.eq_ignore_ascii_case("PEPMASS=")
    {
        rest.split_whitespace().next()?.parse().ok()
    } else {
        None
    }
}

pub(super) fn parse_fragment_prefix(line: &str) -> Option<(f64, f64)> {
    let mut fields = line.split_whitespace();
    let mz = fields.next()?.parse::<f64>().ok()?;
    let intensity = fields.next()?.parse::<f64>().ok()?;
    Some((mz, intensity))
}

#[must_use]
pub fn is_fragment_line(line: &str) -> bool {
    parse_fragment_prefix(line).is_some()
}

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
        let ppm = da * 1e6 / pepmass;
        if da <= 0.02 && ppm <= 100.0 && da < best_delta {
            best_delta = da;
            best_mz = Some(mz);
        }
    }
    best_mz
}

#[must_use]
pub fn find_ms2_precursor_peak<S: AsRef<str>>(spectrum_frags: &[S], pepmass: f64) -> Option<f64> {
    closest_ms2_precursor_mz(
        spectrum_frags
            .iter()
            .filter_map(|frag| frag.as_ref().split_whitespace().next()?.parse::<f64>().ok()),
        pepmass,
    )
}
