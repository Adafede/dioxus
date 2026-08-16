// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Recalibration story generation: MGF round-tripping + bulk fragment correction.

use super::calibration::{
    apply_orbitrap_correction, apply_tof_correction, recalibrate_fragment_mz,
};
use super::parsing::{extract_pepmass_from_line, find_ms2_precursor_peak, is_fragment_line};
use super::types::{CalibrationModel, Peak};
use crate::diagnostics::RecalibrationStats;

pub fn write_recalibrated_fragments(
    spectrum_frags: &[String],
    result: &mut String,
    delta: f64,
    pepmass: f64,
    lambda: f64,
    calibration_model: CalibrationModel,
) {
    use std::fmt::Write;

    for frag in spectrum_frags {
        let parts: Vec<&str> = frag.split_whitespace().collect();
        if parts.len() >= 2
            && let (Ok(mz), Ok(intensity)) = (parts[0].parse::<f64>(), parts[1].parse::<f64>())
        {
            let corrected_mz =
                recalibrate_fragment_mz(mz, delta, pepmass, lambda, calibration_model);
            let _ = write!(result, "{corrected_mz} {intensity}");
            for p in &parts[2..] {
                result.push(' ');
                result.push_str(p);
            }
            result.push('\n');
            continue;
        }
        result.push_str(frag);
        result.push('\n');
    }
}

pub fn write_fragments_as_is(spectrum_frags: &[String], result: &mut String) {
    for frag in spectrum_frags {
        result.push_str(frag);
        result.push('\n');
    }
}

#[must_use]
pub fn generate_recalibrated_mgf(
    original_content: &str,
    calibration_model: CalibrationModel,
    _diagnostics: Option<&RecalibrationStats>,
) -> String {
    #[cfg(target_arch = "wasm32")]
    use web_sys::console;

    if matches!(calibration_model, CalibrationModel::None) {
        #[cfg(target_arch = "wasm32")]
        console::log_1(&"Recalibration: model is None, returning original".into());
        return original_content.to_string();
    }

    let lambda = match calibration_model {
        CalibrationModel::TOFDa { lambda } | CalibrationModel::OrbitrapPPM { lambda } => lambda,
        CalibrationModel::None => 0.0,
    };

    let mut result = String::new();
    let mut pepmass: Option<f64>;
    let mut spectrum_frags: Vec<String> = Vec::new();

    let lines: Vec<&str> = original_content.lines().collect();
    let mut idx = 0;

    while idx < lines.len() {
        let line = lines[idx];
        let trimmed = line.trim();

        if trimmed.eq_ignore_ascii_case("BEGIN IONS") {
            pepmass = None;
            spectrum_frags.clear();
            result.push_str(line);
            result.push('\n');
            idx += 1;

            while idx < lines.len() {
                let spec_line = lines[idx];
                if spec_line.trim().eq_ignore_ascii_case("END IONS") {
                    break;
                }

                if let Some(pm) = extract_pepmass_from_line(spec_line) {
                    pepmass = Some(pm);
                    result.push_str(spec_line);
                    result.push('\n');
                    idx += 1;
                    continue;
                }

                if is_fragment_line(spec_line) {
                    spectrum_frags.push(spec_line.to_string());
                } else {
                    result.push_str(spec_line);
                    result.push('\n');
                }
                idx += 1;
            }

            if let Some(pm) = pepmass {
                if let Some(ms2_peak) = find_ms2_precursor_peak(&spectrum_frags, pm) {
                    let delta = ms2_peak - pm;
                    write_recalibrated_fragments(
                        &spectrum_frags,
                        &mut result,
                        delta,
                        pm,
                        lambda,
                        calibration_model,
                    );
                } else {
                    write_fragments_as_is(&spectrum_frags, &mut result);
                }
            } else {
                write_fragments_as_is(&spectrum_frags, &mut result);
            }

            result.push_str("END IONS");
            result.push('\n');
            idx += 1;
        } else {
            result.push_str(line);
        }
        result.push('\n');
        idx += 1;
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
