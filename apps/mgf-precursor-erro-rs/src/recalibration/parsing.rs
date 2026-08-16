// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! MGF line parsing: pepmass directives and fragment-line detection.

#[must_use]
pub fn extract_pepmass_from_line(line: &str) -> Option<f64> {
    let trimmed = line.trim().to_uppercase();
    if let Some(stripped) = trimmed.strip_prefix("PRECURSOR_MZ=") {
        stripped.split_whitespace().next()?.parse().ok()
    } else if let Some(stripped) = trimmed.strip_prefix("PEPMASS=") {
        stripped.split_whitespace().next()?.parse().ok()
    } else {
        None
    }
}

#[must_use]
pub fn is_fragment_line(line: &str) -> bool {
    let parts: Vec<&str> = line.split_whitespace().collect();
    parts.len() >= 2 && parts[0].parse::<f64>().is_ok() && parts[1].parse::<f64>().is_ok()
}

#[must_use]
pub fn find_ms2_precursor_peak(spectrum_frags: &[String], pepmass: f64) -> Option<f64> {
    let mut best_mz: Option<f64> = None;
    let mut best_delta = f64::INFINITY;

    for frag in spectrum_frags {
        let parts: Vec<&str> = frag.split_whitespace().collect();
        if let Ok(mz) = parts[0].parse::<f64>() {
            let da = (mz - pepmass).abs();
            let ppm = da * 1e6 / pepmass;
            if da <= 0.02 && ppm <= 100.0 && da < best_delta {
                best_delta = da;
                best_mz = Some(mz);
            }
        }
    }
    best_mz
}
