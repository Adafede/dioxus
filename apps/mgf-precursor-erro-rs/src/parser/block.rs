// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Streaming MGF block parser: turns raw MGF `BEGIN IONS` blocks into
//! [`PrecursorStats`]. Depends on the `mass` and `adduct` layers for
//! reference-mass derivation and adduct classification.

use std::collections::{HashMap, HashSet};

#[cfg(target_arch = "wasm32")]
use crate::metrics::merge_precursor_stats;
use upload::UploadError;
#[cfg(target_arch = "wasm32")]
use web_sys::{Blob, console};

use crate::metrics::{
    AdductFamily, ErrorMeasurement, PlotPointSample, PrecursorStats, WarningDetail,
};

use super::adduct::{
    expected_precursor_mz, is_excluded_adduct, is_supported_adduct, normalize_adduct_label,
};
use super::mass::{exact_mass_from_formula_cached, exact_mass_from_smiles_cached};

#[derive(Clone, Debug, Default)]
pub struct BlockParseState {
    observed_precursor_raw: Option<String>,
    observed_precursor: Option<f64>,
    reference_mass: Option<f64>,
    reference_mass_source: Option<String>,
    charge: Option<String>,
    adduct: Option<String>,
    ion_mode: Option<String>,
    smiles: Option<String>,
    formula: Option<String>,
    feature_id: Option<String>,
    scans: Option<String>,
    fragment_peaks: Vec<f64>,
}

impl BlockParseState {
    pub fn consume_line(&mut self, line: &str) {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed == "BEGIN IONS" || trimmed == "END IONS" {
            return;
        }

        if let Some(stripped) = trimmed.strip_prefix("PRECURSOR_MZ=") {
            self.observed_precursor_raw = Some(stripped.to_string());
            if let Ok(value) = stripped.parse::<f64>() {
                self.observed_precursor = Some(value);
            }
            return;
        }

        if let Some(stripped) = trimmed.strip_prefix("PEPMASS=") {
            self.observed_precursor_raw = Some(stripped.to_string());
            if let Ok(value) = stripped.parse::<f64>() {
                self.observed_precursor = Some(value);
            }
            return;
        }

        if let Some(stripped) = trimmed.strip_prefix("EXACTMASS=") {
            if let Ok(value) = stripped.parse::<f64>() {
                self.reference_mass = Some(value);
                self.reference_mass_source = Some("EXACTMASS".to_string());
            }
            return;
        }

        if let Some(stripped) = trimmed.strip_prefix("MOLECULEMASS=") {
            if let Ok(value) = stripped.parse::<f64>()
                && self.reference_mass.is_none()
            {
                self.reference_mass = Some(value);
                self.reference_mass_source = Some("MOLECULEMASS".to_string());
            }
            return;
        }

        if let Some(stripped) = trimmed.strip_prefix("CHARGE=") {
            self.charge = Some(stripped.to_string());
            return;
        }

        if let Some(stripped) = trimmed.strip_prefix("SMILES=") {
            self.smiles = Some(stripped.trim().to_string());
            return;
        }

        if let Some(stripped) = trimmed.strip_prefix("FORMULA=") {
            self.formula = Some(stripped.trim().to_string());
            return;
        }

        if let Some(stripped) = trimmed.strip_prefix("ADDUCT=") {
            self.adduct = Some(stripped.to_string());
            return;
        }

        if let Some(stripped) = trimmed.strip_prefix("IONMODE=") {
            self.ion_mode = Some(stripped.to_string());
            return;
        }

        if let Some(stripped) = trimmed.strip_prefix("FEATURE_ID=") {
            self.feature_id = Some(stripped.to_string());
            return;
        }

        if let Some(stripped) = trimmed.strip_prefix("SCANS=") {
            self.scans = Some(stripped.to_string());
            return;
        }

        if let Some(stripped) = trimmed.strip_prefix("EXTRACTSCAN=") {
            if self.feature_id.is_none() {
                self.feature_id = Some(stripped.to_string());
            }
            if self.scans.is_none() {
                self.scans = Some(stripped.to_string());
            }
            return;
        }

        // Parse fragment line: "m/z intensity" (not a header)
        if !trimmed.contains('=')
            && trimmed.contains(char::is_whitespace)
            && let Some(mz) = trimmed
                .split_whitespace()
                .next()
                .and_then(|value| value.parse::<f64>().ok())
            && mz > 0.0
        {
            self.fragment_peaks.push(mz);
        }
    }

    pub fn consume_block_lines(&mut self, block_lines: &[String]) {
        for line in block_lines {
            self.consume_line(line);
        }
    }

    /// Extract MS2 precursor peak from fragment list.
    /// Returns the closest fragment to PEPMASS if within ~0.02 Da (~100 ppm), otherwise None.
    #[must_use]
    pub fn get_ms2_precursor_peak(&self, pepmass_header: f64) -> Option<f64> {
        const TOLERANCE_DA: f64 = 0.02;

        self.fragment_peaks
            .iter()
            .copied()
            .min_by(|a, b| {
                let dist_a = (a - pepmass_header).abs();
                let dist_b = (b - pepmass_header).abs();
                dist_a
                    .partial_cmp(&dist_b)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .and_then(|closest| {
                let delta_da = (closest - pepmass_header).abs();
                let delta_ppm = if pepmass_header.abs() > f64::EPSILON {
                    delta_da * 1e6 / pepmass_header
                } else {
                    f64::INFINITY
                };

                // Return only if within both Da and ppm thresholds
                if delta_da <= TOLERANCE_DA && delta_ppm <= 100.0 {
                    Some(closest)
                } else {
                    None
                }
            })
    }
}

/// Streaming MGF parser that reads the blob in chunks via [`upload::BlobLines`],
/// yielding one line at a time so memory stays bounded by chunk size (16 MiB)
/// regardless of file size.
///
/// # Errors
/// Returns an error when the blob cannot be read or a scan block cannot be
/// processed.
#[cfg(target_arch = "wasm32")]
pub async fn scan_blob_with_progress(
    blob: &Blob,
    mut on_progress: impl FnMut(u64, u64),
) -> std::result::Result<PrecursorStats, UploadError> {
    let mut reader = upload::BlobLines::new(blob, move |processed, total| {
        on_progress(processed, total);
    });

    let mut current_state = BlockParseState::default();
    let mut current_is_in_block = false;
    let mut metrics = PrecursorStats::default();
    let mut plot_sample = PlotPointSample::default();
    let mut smiles_cache = HashMap::new();
    let mut formula_cache = HashMap::new();
    let mut logged_failures = HashSet::new();

    while let Some(line) = reader.next_line().await? {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        if trimmed == "BEGIN IONS" {
            current_state = BlockParseState::default();
            current_is_in_block = true;
            continue;
        }

        if !current_is_in_block {
            continue;
        }

        current_state.consume_line(trimmed);

        if trimmed == "END IONS" {
            let mut block_plot_sample = Some(&mut plot_sample);
            if let Some(result) = process_block_state(
                &current_state,
                &mut smiles_cache,
                &mut formula_cache,
                &mut logged_failures,
                &mut block_plot_sample,
            )? {
                metrics = merge_precursor_stats(metrics, &result);
            }
            current_state = BlockParseState::default();
            current_is_in_block = false;
        }
    }

    metrics.plot_points = plot_sample.points;
    metrics.plot_point_stream_seen = plot_sample.seen;
    Ok(metrics)
}

/// Process a single MGF block into precursor metrics.
///
/// # Errors
/// Returns an error when the parser cannot produce a valid scan result.
pub fn process_block<S: std::hash::BuildHasher>(
    block_lines: &[String],
    smiles_cache: &mut HashMap<String, Option<f64>, S>,
    formula_cache: &mut HashMap<String, Option<f64>, S>,
    logged_failures: &mut HashSet<String, std::collections::hash_map::RandomState>,
    plot_sample: Option<&mut PlotPointSample>,
) -> std::result::Result<Option<PrecursorStats>, UploadError> {
    let mut state = BlockParseState::default();
    state.consume_block_lines(block_lines);
    let use_external_sample = plot_sample.is_some();
    let mut local_plot_sample = PlotPointSample::default();
    let mut sample_ref = if use_external_sample {
        plot_sample
    } else {
        Some(&mut local_plot_sample)
    };
    let result = process_block_state(
        &state,
        smiles_cache,
        formula_cache,
        logged_failures,
        &mut sample_ref,
    )?;
    Ok(result.map(|mut metrics| {
        if let Some(plot_sample) = sample_ref.as_ref() {
            metrics.plot_points.clone_from(&plot_sample.points);
            metrics.plot_point_stream_seen = plot_sample.seen;
        }
        metrics
    }))
}

#[allow(
    clippy::field_reassign_with_default,
    clippy::too_many_lines,
    clippy::unnecessary_wraps
)]
fn process_block_state<S: std::hash::BuildHasher>(
    state: &BlockParseState,
    smiles_cache: &mut HashMap<String, Option<f64>, S>,
    formula_cache: &mut HashMap<String, Option<f64>, S>,
    logged_failures: &mut HashSet<String, std::collections::hash_map::RandomState>,
    plot_sample: &mut Option<&mut PlotPointSample>,
) -> std::result::Result<Option<PrecursorStats>, UploadError> {
    let Some(observed_precursor) = state.observed_precursor else {
        return Ok(None);
    };

    let reference_mass = state
        .reference_mass
        .map(|mass| {
            (
                mass,
                state
                    .reference_mass_source
                    .clone()
                    .or_else(|| Some("unknown".to_string())),
            )
        })
        .or_else(|| {
            state
                .formula
                .as_deref()
                .and_then(|value| {
                    exact_mass_from_formula_cached(value, formula_cache, logged_failures)
                })
                .map(|mass| (mass, Some("FORMULA".to_string())))
        })
        .or_else(|| {
            let parsed_smiles = state.smiles.as_deref().and_then(|value| {
                exact_mass_from_smiles_cached(value, smiles_cache, logged_failures)
            });
            if state.smiles.is_some() && parsed_smiles.is_none() {
                return None;
            }
            parsed_smiles.map(|mass| (mass, Some("SMILES".to_string())))
        });

    let Some((reference_mass, reference_mass_source)) = reference_mass else {
        let mut metrics = PrecursorStats::default();
        metrics.total_spectra = 1;
        metrics.skipped_spectra = 1;
        if let Some(smiles_text) = state
            .smiles
            .as_deref()
            .filter(|value| !value.trim().is_empty())
        {
            let trimmed_smiles = smiles_text.trim();
            metrics.unparsed_smiles = 1;
            metrics
                .unparsed_smiles_warnings
                .entry(trimmed_smiles.to_string())
                .and_modify(|detail| detail.count = detail.count.saturating_add(1))
                .or_insert_with(|| WarningDetail {
                    count: 1,
                    formula: state.formula.as_deref().map(str::to_string),
                });
            let warning_key = format!(
                "missing-reference-mass:{}|{}",
                trimmed_smiles,
                state.formula.as_deref().unwrap_or("n/a")
            );
            if logged_failures.insert(warning_key) {
                #[cfg(target_arch = "wasm32")]
                console::warn_1(&format!("Unable to derive reference mass from SMILES/formula for: {trimmed_smiles} (formula: {})", state.formula.as_deref().unwrap_or("n/a")).into());
            }
        }
        return Ok(Some(metrics));
    };

    let reference_mass_source = reference_mass_source.unwrap_or_else(|| "unknown".to_string());
    let reference_mass_label = state.adduct.as_deref().map_or_else(
        || reference_mass_source.clone(),
        |adduct| format!("{reference_mass_source} + {adduct}"),
    );
    let adduct_label = normalize_adduct_label(state.adduct.as_deref().unwrap_or("unknown"));
    let adduct_text = state.adduct.as_deref().unwrap_or("").trim();
    let adduct_is_excluded = is_excluded_adduct(adduct_text);
    let adduct_is_supported = adduct_text.is_empty() || is_supported_adduct(adduct_text);
    if adduct_is_excluded || !adduct_is_supported && !adduct_text.is_empty() {
        let mut metrics = PrecursorStats::default();
        metrics.total_spectra = 1;
        metrics.skipped_spectra = 1;
        if !adduct_is_excluded && !adduct_text.is_empty() {
            metrics
                .unrecognized_adducts
                .entry(adduct_text.to_string())
                .and_modify(|count| *count += 1)
                .or_insert(1);
        }
        return Ok(Some(metrics));
    }

    let expected_precursor_mz = expected_precursor_mz(
        reference_mass,
        state.adduct.as_deref(),
        state.charge.as_deref(),
        state.ion_mode.as_deref(),
    )
    .unwrap_or(reference_mass);
    let error_da = observed_precursor - expected_precursor_mz;
    let abs_error_da = error_da.abs();
    let error_milli_da = abs_error_da * 1000.0;
    let ppm = if expected_precursor_mz.abs() > f64::EPSILON {
        error_da / expected_precursor_mz * 1_000_000.0
    } else {
        f64::NAN
    };
    let abs_ppm = ppm.abs();

    let mut metrics = PrecursorStats::default();
    metrics.total_spectra = 1;
    metrics.spectra = 1;
    metrics.spectra_with_reference_mass = 1;
    metrics.reference_mass_source = reference_mass_label;
    metrics.observed_precursor_min = observed_precursor;
    metrics.observed_precursor_max = observed_precursor;
    metrics.observed_precursor_mean = observed_precursor;
    metrics.abs_error_da_min = abs_error_da;
    metrics.abs_error_da_max = abs_error_da;
    metrics.abs_error_da_mean = abs_error_da;
    metrics.abs_error_da_rms = abs_error_da;
    metrics.abs_error_ppm_min = abs_ppm;
    metrics.abs_error_ppm_max = abs_ppm;
    metrics.abs_error_ppm_mean = abs_ppm;
    metrics.abs_error_ppm_rms = abs_ppm;
    metrics.signed_error_da_mean = error_da;
    metrics.signed_error_ppm_mean = ppm;
    metrics.record_error_with_plot_sample(
        ErrorMeasurement {
            abs_error_da,
            abs_ppm,
            adduct_family: AdductFamily::from_label(&adduct_label),
            ppm_error: ppm,
            signed_error_da: error_da,
            pepmass_header: observed_precursor, // PEPMASS from header (metadata block)
            ms2_precursor_peak: state.get_ms2_precursor_peak(observed_precursor), // MS2 precursor peak, closest to PEPMASS within tolerance
            smiles: state.smiles.as_deref(),
            calculated_mass: Some(reference_mass),
            expected_mass: Some(expected_precursor_mz),
            formula: state.formula.as_deref(),
        },
        plot_sample,
    );
    if error_milli_da <= 0.1 {
        metrics.within_0_1_da = 1;
    } else if error_milli_da <= 0.5 {
        metrics.within_0_5_da = 1;
    } else if error_milli_da <= 1.0 {
        metrics.within_1_da = 1;
    } else if error_milli_da <= 5.0 {
        metrics.within_5_da = 1;
    } else {
        metrics.above_5_da = 1;
    }
    if abs_ppm <= 0.5 {
        metrics.within_0_5_ppm = 1;
    } else if abs_ppm <= 1.0 {
        metrics.within_1_ppm = 1;
    } else if abs_ppm <= 5.0 {
        metrics.within_5_ppm = 1;
    } else if abs_ppm <= 10.0 {
        metrics.within_10_ppm = 1;
    } else {
        metrics.above_10_ppm = 1;
    }

    Ok(Some(metrics))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_full_precision_when_computing_error() {
        let state = BlockParseState {
            observed_precursor_raw: Some("100.1234".to_string()),
            observed_precursor: Some(100.1234),
            reference_mass: Some(100.123_456_789),
            reference_mass_source: Some("EXACTMASS".to_string()),
            charge: Some("1".to_string()),
            ..Default::default()
        };

        let mut smiles_cache = HashMap::new();
        let mut formula_cache = HashMap::new();
        let mut logged_failures = HashSet::new();
        let mut plot_sample = None;

        let metrics = process_block_state(
            &state,
            &mut smiles_cache,
            &mut formula_cache,
            &mut logged_failures,
            &mut plot_sample,
        )
        .expect("parser should succeed")
        .expect("metrics should be produced");

        let expected_error = 100.1234_f64 - 100.123_456_789_f64;
        assert!((metrics.sample_abs_error_da - expected_error.abs()).abs() < 1e-12);
    }
}
