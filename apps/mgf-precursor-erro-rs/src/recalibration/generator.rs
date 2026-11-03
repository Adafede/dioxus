// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the mgf-precursor-erro-rs project

//! Recalibration story generation: MGF round-tripping + bulk fragment correction.

use super::calibration::{
    apply_orbitrap_correction, apply_tof_correction, recalibrate_fragment_mz,
};
use super::parsing::{closest_ms2_precursor_mz, extract_pepmass_from_line, parse_fragment_prefix};
use super::types::{CalibrationModel, Peak};
use crate::diagnostics::RecalibrationStats;

fn write_recalibrated_fragment_line(
    frag: &str,
    mz: f64,
    intensity: f64,
    result: &mut String,
    delta: f64,
    pepmass: f64,
    lambda: f64,
    calibration_model: CalibrationModel,
) {
    use std::fmt::Write;

    let corrected_mz = recalibrate_fragment_mz(mz, delta, pepmass, lambda, calibration_model);
    let _ = write!(result, "{corrected_mz} {intensity}");
    for field in frag.split_whitespace().skip(2) {
        result.push(' ');
        result.push_str(field);
    }
    result.push('\n');
}

pub fn write_recalibrated_fragments<S: AsRef<str>>(
    spectrum_frags: &[S],
    result: &mut String,
    delta: f64,
    pepmass: f64,
    lambda: f64,
    calibration_model: CalibrationModel,
) {
    for frag in spectrum_frags {
        let frag = frag.as_ref();
        if let Some((mz, intensity)) = parse_fragment_prefix(frag) {
            write_recalibrated_fragment_line(
                frag,
                mz,
                intensity,
                result,
                delta,
                pepmass,
                lambda,
                calibration_model,
            );
        } else {
            result.push_str(frag);
            result.push('\n');
        }
    }
}

pub fn write_fragments_as_is<S: AsRef<str>>(spectrum_frags: &[S], result: &mut String) {
    for frag in spectrum_frags {
        result.push_str(frag.as_ref());
        result.push('\n');
    }
}

#[must_use]
pub fn generate_recalibrated_mgf(
    original_content: &str,
    calibration_model: CalibrationModel,
    _diagnostics: Option<&RecalibrationStats>,
) -> String {
    if matches!(calibration_model, CalibrationModel::None) {
        return original_content.to_string();
    }

    let lambda = match calibration_model {
        CalibrationModel::TOFDa { lambda } | CalibrationModel::OrbitrapPPM { lambda } => lambda,
        CalibrationModel::None => 0.0,
    };

    let mut result = String::with_capacity(original_content.len());
    let mut spectrum_frags: Vec<(&str, f64, f64)> = Vec::new();
    let mut lines = original_content.lines();

    while let Some(line) = lines.next() {
        let trimmed = line.trim();

        if !trimmed.eq_ignore_ascii_case("BEGIN IONS") {
            result.push_str(line);
            result.push('\n');
            continue;
        }

        let mut pepmass: Option<f64> = None;
        spectrum_frags.clear();
        result.push_str(line);
        result.push('\n');

        for spec_line in lines.by_ref() {
            if spec_line.trim().eq_ignore_ascii_case("END IONS") {
                break;
            }

            if let Some(pm) = extract_pepmass_from_line(spec_line) {
                pepmass = Some(pm);
                result.push_str(spec_line);
                result.push('\n');
                continue;
            }

            if let Some((mz, intensity)) = parse_fragment_prefix(spec_line) {
                spectrum_frags.push((spec_line, mz, intensity));
            } else {
                result.push_str(spec_line);
                result.push('\n');
            }
        }

        if let Some(pm) = pepmass
            && let Some(ms2_peak) =
                closest_ms2_precursor_mz(spectrum_frags.iter().map(|&(_, mz, _)| mz), pm)
        {
            let delta = ms2_peak - pm;
            for &(frag, mz, intensity) in &spectrum_frags {
                write_recalibrated_fragment_line(
                    frag,
                    mz,
                    intensity,
                    &mut result,
                    delta,
                    pm,
                    lambda,
                    calibration_model,
                );
            }
        } else {
            for &(frag, _, _) in &spectrum_frags {
                result.push_str(frag);
                result.push('\n');
            }
        }

        result.push_str("END IONS\n");
    }

    result
}

/// Recalibrates fragment m/z values based on the precursor discrepancy.
///
/// This function applies the calibration model to all fragments, modifying their m/z values
/// in-place while preserving intensities and peak ordering.
///
/// # Arguments
///
/// * `fragments` - A slice of peaks to recalibrate. m/z values are modified in place.
/// * `precursor_ms1` - The precursor m/z measured in MS1 (reference).
/// * `precursor_ms2` - The precursor m/z reported in the MS2 scan.
/// * `model` - The calibration model to apply.
///
/// # Behavior
///
/// - If `model` is `None`, the function returns immediately without modification.
/// - If either precursor is missing (NaN or infinite), no correction is applied.
/// - Corrections smaller than floating-point precision (< 1e-14) are skipped.
/// - Fragment intensities are never modified.
/// - Fragment ordering is preserved.
///
/// # Numerical stability
///
/// For Orbitrap (ppm) model:
/// - A ppm error < 1e-7 (parts per million) is considered negligible and skipped.
///
/// For TOF (Da) model:
/// - A Da error < 1e-14 is considered negligible and skipped.
pub fn recalibrate_fragments(
    fragments: &mut [Peak],
    precursor_ms1: f64,
    precursor_ms2: f64,
    model: CalibrationModel,
) {
    // Skip if no model is active
    if !model.is_active() {
        return;
    }

    // Skip if either precursor is missing or invalid
    if !precursor_ms1.is_finite() || !precursor_ms2.is_finite() {
        return;
    }

    // Skip if precursor values are zero (would cause division by zero or nonsensical correction)
    if precursor_ms1.abs() < f64::EPSILON {
        return;
    }

    match model {
        CalibrationModel::None => {
            // Already handled above
        }
        CalibrationModel::OrbitrapPPM { lambda } => {
            apply_orbitrap_correction(fragments, precursor_ms1, precursor_ms2, lambda);
        }
        CalibrationModel::TOFDa { lambda } => {
            apply_tof_correction(fragments, precursor_ms1, precursor_ms2, lambda);
        }
    }
}
// BOUNDARY NOTE (kept out of the hot path): the `BEGIN IONS` branch advances
// `idx` itself as it walks the block, so it must not fall through to the
// loop-level advance. The line after an `END IONS` is the following
// `BEGIN IONS`, and consuming it merged two spectra into one.
//
// Do not re-add a shared tail advance to this loop.
