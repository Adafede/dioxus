// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the mgf-precursor-erro-rs project

//! Bulk fragment slice correction and full-MGF recalibrated text generation.

use std::fmt::Write as _;

use mgf::{extract_pepmass_from_line, is_ions_marker};

use super::calibration::{
    apply_orbitrap_correction, apply_tof_correction, recalibrate_fragment_mz,
};
use super::parsing::{closest_ms2_precursor_mz, parse_fragment_prefix};
use super::types::{CalibrationModel, Peak};

#[derive(Clone, Copy)]
struct RecalibrationParams {
    delta: f64,
    pepmass: f64,
    model: CalibrationModel,
}

fn write_recalibrated_fragment_line(
    frag: &str,
    mz: f64,
    intensity: f64,
    result: &mut String,
    params: RecalibrationParams,
) {
    let corrected_mz = recalibrate_fragment_mz(mz, params.delta, params.pepmass, params.model);
    let _ = write!(result, "{corrected_mz} {intensity}");
    for field in frag.split_whitespace().skip(2) {
        result.push(' ');
        result.push_str(field);
    }
    result.push('\n');
}

/// Generates recalibrated MGF text for every `BEGIN IONS ... END IONS` spectrum block.
#[must_use]
pub fn generate_recalibrated_mgf(original_content: &str, model: CalibrationModel) -> String {
    if !model.is_active() {
        return original_content.to_string();
    }

    let mut result = String::with_capacity(original_content.len());
    let mut spectrum_frags: Vec<(&str, f64, f64)> = Vec::new();
    let mut lines = original_content.lines();

    while let Some(line) = lines.next() {
        if !is_ions_marker(line, "BEGIN IONS") {
            result.push_str(line);
            result.push('\n');
            continue;
        }

        let mut pepmass: Option<f64> = None;
        let mut end_line: Option<&str> = None;
        spectrum_frags.clear();
        result.push_str(line);
        result.push('\n');

        for spec_line in lines.by_ref() {
            if is_ions_marker(spec_line, "END IONS") {
                end_line = Some(spec_line);
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
            let params = RecalibrationParams {
                delta: ms2_peak - pm,
                pepmass: pm,
                model,
            };
            for &(frag, mz, intensity) in &spectrum_frags {
                write_recalibrated_fragment_line(frag, mz, intensity, &mut result, params);
            }
        } else {
            for &(frag, _, _) in &spectrum_frags {
                result.push_str(frag);
                result.push('\n');
            }
        }

        if let Some(end_marker) = end_line {
            result.push_str(end_marker);
            result.push('\n');
        }
    }

    result
}

/// Applies `model` to `fragments` in place given MS1 and MS2 precursor *m/z*.
pub fn recalibrate_fragments(
    fragments: &mut [Peak],
    precursor_ms1: f64,
    precursor_ms2: f64,
    model: CalibrationModel,
) {
    if !precursor_ms1.is_finite() || !precursor_ms2.is_finite() || precursor_ms1 <= 0.0 {
        return;
    }

    match model {
        CalibrationModel::None => {}
        CalibrationModel::OrbitrapPPM { lambda } => {
            apply_orbitrap_correction(fragments, precursor_ms1, precursor_ms2, lambda);
        }
        CalibrationModel::TOFDa { lambda } => {
            apply_tof_correction(fragments, precursor_ms1, precursor_ms2, lambda);
        }
    }
}
