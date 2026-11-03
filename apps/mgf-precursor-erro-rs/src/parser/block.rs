// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the mgf-precursor-erro-rs project

//! Streaming MGF block parser: turns raw MGF `BEGIN IONS` blocks into
//! [`PrecursorStats`]. Depends on the `mass` and `adduct` layers for
//! reference-mass derivation and adduct classification.

use std::collections::HashMap;

#[cfg(target_arch = "wasm32")]
use upload::UploadError;
#[cfg(target_arch = "wasm32")]
use web_sys::{Blob, console};
// Only `scan_blob_with_progress` needs the browser. Everything else here —
// `BlockParseState`, the line consumer, the adduct/mass helpers — is plain
// `&str` arithmetic and runs off-wasm, which is what makes it testable.
// `console` is the only browser symbol either module uses at runtime, and each
// call site is individually gated.

use crate::metrics::{AdductFamily, ErrorMeasurement, PrecursorStats, WarningDetail};

use super::adduct::{expected_precursor_mz, is_supported_adduct, normalize_adduct_label};
use super::mass::{exact_mass_from_formula_cached, exact_mass_from_smiles_cached};

#[derive(Clone, Debug, Default)]
pub(crate) struct BlockParseState {
    #[cfg(test)]
    observed_precursor_raw: Option<String>,
    observed_precursor: Option<f64>,
    reference_mass: Option<f64>,
    charge: Option<String>,
    adduct: Option<String>,
    ion_mode: Option<String>,
    smiles: Option<String>,
    formula: Option<String>,
    fragment_peaks: Vec<f64>,
}

/// Parses the first whitespace-separated float in an MGF value field.
///
/// `PEPMASS` carries an optional second field — the precursor intensity — per
/// the format, so `PEPMASS=400.0 1234.5` is valid input. Parsing the whole
/// remainder as one float fails, and a spectrum whose `observed_precursor`
/// never gets set is dropped from the statistics entirely, without being
/// counted as skipped and without a warning. Taking the leading token keeps
/// the intensity visible in `observed_precursor_raw` and still scores the
/// spectrum.
fn parse_leading_float(value: &str) -> Option<f64> {
    value.split_whitespace().next()?.parse::<f64>().ok()
}

impl BlockParseState {
    pub(crate) fn clear(&mut self) {
        #[cfg(test)]
        {
            self.observed_precursor_raw = None;
        }
        self.observed_precursor = None;
        self.reference_mass = None;
        self.charge = None;
        self.adduct = None;
        self.ion_mode = None;
        self.smiles = None;
        self.formula = None;
        self.fragment_peaks.clear();
    }

    pub(crate) fn consume_line(&mut self, line: &str) {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed == "BEGIN IONS" || trimmed == "END IONS" {
            return;
        }

        if let Some(stripped) = trimmed.strip_prefix("PRECURSOR_MZ=") {
            #[cfg(test)]
            {
                self.observed_precursor_raw = Some(stripped.to_string());
            }
            self.observed_precursor = parse_leading_float(stripped);
            return;
        }

        if let Some(stripped) = trimmed.strip_prefix("PEPMASS=") {
            #[cfg(test)]
            {
                self.observed_precursor_raw = Some(stripped.to_string());
            }
            self.observed_precursor = parse_leading_float(stripped);
            return;
        }

        if let Some(stripped) = trimmed.strip_prefix("EXACTMASS=") {
            if let Ok(value) = stripped.parse::<f64>() {
                self.reference_mass = Some(value);
            }
            return;
        }

        if let Some(stripped) = trimmed.strip_prefix("MOLECULEMASS=") {
            if let Ok(value) = stripped.parse::<f64>()
                && self.reference_mass.is_none()
            {
                self.reference_mass = Some(value);
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

    /// Extract MS2 precursor peak from fragment list.
    /// Returns the closest fragment to PEPMASS if within ~0.02 Da (~100 ppm), otherwise None.
    #[must_use]
    pub(crate) fn get_ms2_precursor_peak(&self, pepmass_header: f64) -> Option<f64> {
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

#[cfg_attr(
    not(any(test, target_arch = "wasm32")),
    allow(
        dead_code,
        reason = "reached from wasm32 builds and from the tests below"
    )
)]
#[derive(Default)]
struct LineIngestor {
    current_state: BlockParseState,
    current_is_in_block: bool,
    metrics: PrecursorStats,
    smiles_cache: HashMap<String, Option<f64>>,
    formula_cache: HashMap<String, Option<f64>>,
}

#[cfg_attr(
    not(any(test, target_arch = "wasm32")),
    allow(
        dead_code,
        reason = "reached from wasm32 builds and from the tests below"
    )
)]
impl LineIngestor {
    fn feed_line(&mut self, index: usize, line: &str) {
        // A leading BOM is the first byte of the first line and is category Cf,
        // so `trim` does not remove it. Left in place, `"\u{feff}BEGIN IONS"`
        // never matches the literal, `current_is_in_block` stays false for the
        // rest of the file, and every spectrum is discarded silently: no error,
        // and a Summary panel that reads `0`. Strip it on the first line only.
        let trimmed = if index == 0 {
            line.trim_start_matches('\u{feff}')
        } else {
            line
        }
        .trim();
        if trimmed.is_empty() {
            return;
        }

        if trimmed == "BEGIN IONS" {
            self.current_state.clear();
            self.current_is_in_block = true;
            return;
        }

        if !self.current_is_in_block {
            return;
        }

        self.current_state.consume_line(trimmed);

        if trimmed == "END IONS" {
            record_block_state(
                &mut self.metrics,
                &self.current_state,
                &mut self.smiles_cache,
                &mut self.formula_cache,
            );
            self.current_state.clear();
            self.current_is_in_block = false;
        }
    }

    fn finish(self) -> PrecursorStats {
        self.metrics
    }
}

/// Drives a whole MGF document through the block state machine, one line at a
/// time.
#[cfg(test)]
pub(crate) fn ingest_lines<'a>(
    lines: impl Iterator<Item = &'a str>,
    mut on_progress: impl FnMut(u64),
) -> PrecursorStats {
    let mut ingestor = LineIngestor::default();
    for (index, line) in lines.enumerate() {
        ingestor.feed_line(index, line);
        if line.trim() == "END IONS" {
            on_progress(u64::try_from(index).unwrap_or(u64::MAX));
        }
    }
    ingestor.finish()
}

/// Streaming MGF parser that reads the blob in chunks via [`upload::BlobLines`],
/// yielding one line at a time so each line is parsed on the fly without
/// collecting a `Vec<String>` of all lines or reading the blob a second time.
///
/// # Errors
/// Returns an error when the blob cannot be read.
#[cfg(target_arch = "wasm32")]
#[allow(clippy::future_not_send)] // async fn captures non-Send browser futures
pub(crate) async fn scan_blob_with_progress(
    blob: &Blob,
    on_progress: impl FnMut(u64, u64),
) -> std::result::Result<(PrecursorStats, String), UploadError> {
    let mut reader = upload::BlobLines::new(upload::BlobSource::new(blob), on_progress);
    let mut ingestor = LineIngestor::default();
    let mut raw_content = String::new();
    let mut index = 0usize;

    while let Some(line) = reader.next_line().await? {
        ingestor.feed_line(index, &line);
        raw_content.push_str(&line);
        raw_content.push('\n');
        index += 1;
    }

    Ok((ingestor.finish(), raw_content))
}

fn compute_reference_mass<S: std::hash::BuildHasher>(
    state: &BlockParseState,
    smiles_cache: &mut HashMap<String, Option<f64>, S>,
    formula_cache: &mut HashMap<String, Option<f64>, S>,
) -> Option<f64> {
    state
        .reference_mass
        .or_else(|| {
            state
                .formula
                .as_deref()
                .and_then(|value| exact_mass_from_formula_cached(value, formula_cache))
        })
        .or_else(|| {
            state
                .smiles
                .as_deref()
                .and_then(|value| exact_mass_from_smiles_cached(value, smiles_cache))
        })
}

/// Record metrics for a block whose reference mass could not be derived from
/// the precursor's SMILES or formula.
fn record_unparsable_smiles(metrics: &mut PrecursorStats, state: &BlockParseState) {
    metrics.total_spectra += 1;
    metrics.skipped_spectra += 1;
    metrics.unparsed_smiles += 1;
    if let Some(smiles_text) = state
        .smiles
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        if let Some(detail) = metrics.unparsed_smiles_warnings.get_mut(smiles_text) {
            detail.count = detail.count.saturating_add(1);
            if detail.formula.is_none() {
                detail.formula = state.formula.as_deref().map(str::to_string);
            }
        } else {
            #[cfg(target_arch = "wasm32")]
            console::warn_1(
                &format!(
                    "Unable to derive reference mass from SMILES/formula for: {smiles_text} (formula: {})",
                    state.formula.as_deref().unwrap_or("n/a")
                )
                .into(),
            );
            metrics.unparsed_smiles_warnings.insert(
                smiles_text.to_string(),
                WarningDetail {
                    count: 1,
                    formula: state.formula.as_deref().map(str::to_string),
                },
            );
        }
    }
}

/// Record metrics for an observed precursor that has a usable reference mass.
fn record_observed_metrics(
    metrics: &mut PrecursorStats,
    observed_precursor: f64,
    reference_mass: f64,
    adduct_label: &str,
    state: &BlockParseState,
) {
    let expected_precursor_mz = expected_precursor_mz(
        reference_mass,
        state.adduct.as_deref(),
        state.charge.as_deref(),
        state.ion_mode.as_deref(),
    );
    let expected_precursor_mz = if expected_precursor_mz.is_finite() {
        expected_precursor_mz
    } else {
        reference_mass
    };
    let error_da = observed_precursor - expected_precursor_mz;
    let abs_error_da = error_da.abs();
    let ppm = if expected_precursor_mz.abs() > f64::EPSILON {
        error_da / expected_precursor_mz * 1_000_000.0
    } else {
        f64::NAN
    };
    let abs_ppm = ppm.abs();

    metrics.record_error(ErrorMeasurement {
        abs_error_da,
        abs_ppm,
        adduct_family: AdductFamily::from_label(adduct_label),
        ppm_error: ppm,
        signed_error_da: error_da,
        pepmass_header: observed_precursor,
        ms2_precursor_peak: state.get_ms2_precursor_peak(observed_precursor),
        smiles: state.smiles.as_deref(),
        calculated_mass: Some(reference_mass),
        expected_mass: Some(expected_precursor_mz),
        formula: state.formula.as_deref(),
    });
}

fn record_block_state<S: std::hash::BuildHasher>(
    metrics: &mut PrecursorStats,
    state: &BlockParseState,
    smiles_cache: &mut HashMap<String, Option<f64>, S>,
    formula_cache: &mut HashMap<String, Option<f64>, S>,
) {
    let Some(observed_precursor) = state.observed_precursor else {
        return;
    };

    let Some(reference_mass) = compute_reference_mass(state, smiles_cache, formula_cache) else {
        record_unparsable_smiles(metrics, state);
        return;
    };

    let adduct_text = state.adduct.as_deref().unwrap_or("").trim();
    if !adduct_text.is_empty() && !is_supported_adduct(adduct_text) {
        metrics.total_spectra += 1;
        metrics.skipped_spectra += 1;
        if let Some(count) = metrics.unrecognized_adducts.get_mut(adduct_text) {
            *count += 1;
        } else {
            metrics
                .unrecognized_adducts
                .insert(adduct_text.to_string(), 1);
        }
        return;
    }

    let adduct_label = normalize_adduct_label(adduct_text);
    record_observed_metrics(
        metrics,
        observed_precursor,
        reference_mass,
        adduct_label,
        state,
    );
}

#[cfg(test)]
mod tests;
