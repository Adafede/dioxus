// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the mgf-precursor-erro-rs project

//! Streaming MGF ingestion delegating block parsing to [`mgf::MgfLineParser`].

use std::collections::HashMap;
use std::hash::BuildHasher;

use mgf::{MgfBlock, MgfLineParser, is_ions_marker};

use crate::adduct::{
    AdductFamily, expected_precursor_mz, is_supported_adduct, normalize_adduct_label,
};
use crate::mass::{exact_mass_from_formula_cached, exact_mass_from_smiles_cached};
use crate::metrics::stats::PrecursorStats;
use crate::metrics::{ErrorMeasurement, WarningDetail};
use crate::recalibration::parsing::closest_ms2_precursor_mz;

/// Upper bound (`64 MiB`) on retained raw MGF text kept in memory for recalibrated export.
pub const MAX_RAW_RETENTION_BYTES: usize = 64 * 1024 * 1024;

/// Appends `line` to `raw_content` unless doing so would exceed `max_bytes`.
///
/// Once the cap is exceeded, clears and shrinks `raw_content` and sets
/// `raw_retention_capped = true` so streaming diagnostics cover the whole file
/// without unbounded buffer growth.
pub fn append_retained_line(
    raw_content: &mut String,
    raw_retention_capped: &mut bool,
    line: &str,
    max_bytes: usize,
) {
    if *raw_retention_capped {
        return;
    }
    let incoming = line.len().saturating_add(1);
    if raw_content.len().saturating_add(incoming) > max_bytes {
        raw_content.clear();
        raw_content.shrink_to_fit();
        *raw_retention_capped = true;
        return;
    }
    raw_content.push_str(line);
    raw_content.push('\n');
}

/// Incremental MGF line ingestor backed by [`MgfLineParser`] and cached mass lookups.
#[derive(Debug)]
pub struct LineIngestor {
    parser: MgfLineParser,
    metrics: PrecursorStats,
    smiles_cache: HashMap<String, Option<f64>>,
    formula_cache: HashMap<String, Option<f64>>,
}

impl Default for LineIngestor {
    fn default() -> Self {
        Self {
            parser: MgfLineParser::new(),
            metrics: PrecursorStats::default(),
            smiles_cache: HashMap::new(),
            formula_cache: HashMap::new(),
        }
    }
}

impl LineIngestor {
    /// Feeds one MGF line into the underlying parser, scoring completed `END IONS` blocks.
    pub fn feed_line(&mut self, line: &str) {
        if let Some(block) = self.parser.push_line(line)
            && is_ions_marker(line, "END IONS")
        {
            record_block(
                &mut self.metrics,
                &block,
                &mut self.smiles_cache,
                &mut self.formula_cache,
            );
        }
    }

    /// Finishes ingestion and returns the accumulated [`PrecursorStats`].
    #[must_use]
    pub fn finish(self) -> PrecursorStats {
        self.metrics
    }
}

/// Drives an in-memory MGF line iterator through [`LineIngestor`].
#[cfg(test)]
pub(crate) fn ingest_lines<'a>(
    lines: impl Iterator<Item = &'a str>,
    mut on_block_line: impl FnMut(u64),
) -> PrecursorStats {
    let mut ingestor = LineIngestor::default();
    for (index, line) in lines.enumerate() {
        ingestor.feed_line(line);
        if is_ions_marker(line, "END IONS") {
            on_block_line(u64::try_from(index).unwrap_or(u64::MAX));
        }
    }
    ingestor.finish()
}

/// Evaluates one parsed [`MgfBlock`] and folds it into `metrics`.
pub(crate) fn record_block<S: BuildHasher>(
    metrics: &mut PrecursorStats,
    block: &MgfBlock,
    smiles_cache: &mut HashMap<String, Option<f64>, S>,
    formula_cache: &mut HashMap<String, Option<f64>, S>,
) {
    let Some(observed_precursor) = block.header.effective_precursor_mz() else {
        return;
    };

    let Some(reference_mass) = compute_reference_mass(block, smiles_cache, formula_cache) else {
        record_unparsable_smiles(metrics, block);
        return;
    };

    let adduct_text = block.header.adduct.as_deref().unwrap_or("").trim();
    if !adduct_text.is_empty() && !is_supported_adduct(adduct_text) {
        metrics.total_spectra += 1;
        metrics.skipped_spectra += 1;
        *metrics
            .unrecognized_adducts
            .entry(adduct_text.to_string())
            .or_insert(0) += 1;
        return;
    }

    let adduct_label = normalize_adduct_label(adduct_text);
    record_observed_metrics(
        metrics,
        observed_precursor,
        reference_mass,
        adduct_label,
        block,
    );
}

fn compute_reference_mass<S: BuildHasher>(
    block: &MgfBlock,
    smiles_cache: &mut HashMap<String, Option<f64>, S>,
    formula_cache: &mut HashMap<String, Option<f64>, S>,
) -> Option<f64> {
    block
        .header
        .reference_mass()
        .or_else(|| {
            block
                .header
                .formula
                .as_deref()
                .and_then(|value| exact_mass_from_formula_cached(value, formula_cache))
        })
        .or_else(|| {
            block
                .header
                .smiles
                .as_deref()
                .and_then(|value| exact_mass_from_smiles_cached(value, smiles_cache))
        })
}

fn record_unparsable_smiles(metrics: &mut PrecursorStats, block: &MgfBlock) {
    metrics.total_spectra += 1;
    metrics.skipped_spectra += 1;
    metrics.unparsed_smiles += 1;
    if let Some(smiles_text) = block
        .header
        .smiles
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        if let Some(detail) = metrics.unparsed_smiles_warnings.get_mut(smiles_text) {
            detail.count = detail.count.saturating_add(1);
            if detail.formula.is_none() {
                detail.formula.clone_from(&block.header.formula);
            }
        } else {
            metrics.unparsed_smiles_warnings.insert(
                smiles_text.to_string(),
                WarningDetail {
                    count: 1,
                    formula: block.header.formula.clone(),
                },
            );
        }
    }
}

fn record_observed_metrics(
    metrics: &mut PrecursorStats,
    observed_precursor: f64,
    reference_mass: f64,
    adduct_label: &str,
    block: &MgfBlock,
) {
    let expected_mz = expected_precursor_mz(
        reference_mass,
        block.header.adduct.as_deref(),
        block.header.charge.as_deref(),
        block.header.ion_mode.as_deref(),
    );
    let expected_mz = if expected_mz.is_finite() {
        expected_mz
    } else {
        reference_mass
    };
    let error_da = observed_precursor - expected_mz;
    let abs_error_da = error_da.abs();
    let ppm = if expected_mz.abs() > f64::EPSILON {
        error_da / expected_mz * 1_000_000.0
    } else {
        f64::NAN
    };
    let ms2_precursor_peak =
        closest_ms2_precursor_mz(block.peaks.iter().map(|peak| peak.mz), observed_precursor);

    metrics.record_error(ErrorMeasurement {
        abs_error_da,
        abs_ppm: ppm.abs(),
        adduct_family: AdductFamily::from_label(adduct_label),
        ppm_error: ppm,
        signed_error_da: error_da,
        pepmass_header: observed_precursor,
        ms2_precursor_peak,
        smiles: block.header.smiles.as_deref(),
        calculated_mass: Some(reference_mass),
        expected_mass: Some(expected_mz),
        formula: block.header.formula.as_deref(),
    });
}
