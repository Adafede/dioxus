// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the mgf-precursor-erro-rs project

//! Streaming [`PrecursorStats`] accumulator definition and query methods.

use std::collections::BTreeMap;

use super::{ErrorBucket, HighErrorSmilesDetail, MedianTracker, PlotPoint, WarningDetail};
use crate::numeric::reservoir_index;

/// Streaming precursor mass-error accumulator.
///
/// Reservoir arrays (`MAX_ECDF_POINTS`, `MAX_PLOT_POINTS`) and scalar moments use
/// bounded memory, whereas exact running medians keep `O(N)` dual-heap trackers.
#[derive(Clone, Debug, Default)]
pub struct PrecursorStats {
    /// Spectra with both a precursor measurement and usable reference mass + supported adduct.
    pub spectra: usize,
    /// Total spectrum blocks encountered (including skipped).
    pub total_spectra: usize,
    /// Blocks skipped due to unresolved mass or unsupported adduct.
    pub skipped_spectra: usize,
    /// Count of spectra skipped due to unparsable SMILES/formula.
    pub unparsed_smiles: usize,
    /// Unparsable SMILES counts keyed by SMILES text.
    pub unparsed_smiles_warnings: BTreeMap<String, WarningDetail>,
    /// Minimum observed precursor `m/z`.
    pub observed_precursor_min: f64,
    /// Maximum observed precursor `m/z`.
    pub observed_precursor_max: f64,
    /// Running mean observed precursor `m/z`.
    pub observed_precursor_mean: f64,
    /// Exact running median observed precursor `m/z`.
    pub observed_precursor_median: f64,
    /// Mean absolute error (Da).
    pub abs_error_da_mean: f64,
    /// Median absolute error (Da).
    pub abs_error_da_median: f64,
    /// Root-mean-square error (Da).
    pub abs_error_da_rms: f64,
    /// Mean absolute error (ppm).
    pub abs_error_ppm_mean: f64,
    /// Median absolute error (ppm).
    pub abs_error_ppm_median: f64,
    /// Root-mean-square error (ppm).
    pub abs_error_ppm_rms: f64,
    /// Spectra with `<= 0.1 mDa` absolute error.
    pub within_0_1_mda: usize,
    /// Spectra in `(0.1, 0.5] mDa`.
    pub within_0_5_mda: usize,
    /// Spectra in `(0.5, 1.0] mDa`.
    pub within_1_mda: usize,
    /// Spectra in `(1.0, 5.0] mDa`.
    pub within_5_mda: usize,
    /// Spectra with `> 5.0 mDa` absolute error.
    pub above_5_mda: usize,
    /// Spectra with `<= 0.5 ppm` absolute error.
    pub within_0_5_ppm: usize,
    /// Spectra in `(0.5, 1.0] ppm`.
    pub within_1_ppm: usize,
    /// Spectra in `(1.0, 5.0] ppm`.
    pub within_5_ppm: usize,
    /// Spectra in `(5.0, 10.0] ppm`.
    pub within_10_ppm: usize,
    /// Spectra with `> 10.0 ppm` absolute error.
    pub above_10_ppm: usize,
    /// Reservoir-sampled absolute errors in Da (`MAX_ECDF_POINTS`).
    pub absolute_error_da_values: Vec<f64>,
    /// Reservoir-sampled absolute errors in ppm (`MAX_ECDF_POINTS`).
    pub absolute_error_ppm_values: Vec<f64>,
    pub(super) absolute_error_da_sample_seen: usize,
    pub(super) absolute_error_ppm_sample_seen: usize,
    /// Reservoir-sampled scatter points (`MAX_PLOT_POINTS`).
    pub plot_points: Vec<PlotPoint>,
    pub(super) plot_point_stream_seen: usize,
    pub(super) observed_precursor_median_tracker: MedianTracker,
    pub(super) abs_error_da_median_tracker: MedianTracker,
    pub(super) abs_error_ppm_median_tracker: MedianTracker,
    /// Unrecognized adduct counts keyed by raw `ADDUCT=` text.
    pub unrecognized_adducts: BTreeMap<String, usize>,
    /// Spectra exceeding `0.01 Da` error keyed by SMILES.
    pub high_error_smiles: BTreeMap<String, HighErrorSmilesDetail>,
    /// `true` when raw MGF retention exceeded `MAX_RAW_RETENTION_BYTES`.
    pub raw_retention_capped: bool,
}

pub(super) fn push_reservoir<T>(
    value: T,
    values: &mut Vec<T>,
    seen: &mut usize,
    max_points: usize,
) {
    *seen = seen.saturating_add(1);
    if values.len() < max_points {
        values.push(value);
        return;
    }
    let idx = reservoir_index(*seen);
    if idx < max_points
        && let Some(slot) = values.get_mut(idx)
    {
        *slot = value;
    }
}

impl PrecursorStats {
    /// Returns the cumulative count for the given tolerance [`ErrorBucket`].
    #[must_use]
    pub const fn cumulative_bucket_count(&self, bucket: ErrorBucket) -> usize {
        match bucket {
            ErrorBucket::Within0_1Mda => self.within_0_1_mda,
            ErrorBucket::Within0_5Mda => self.within_0_1_mda + self.within_0_5_mda,
            ErrorBucket::Within1Mda => {
                self.within_0_1_mda + self.within_0_5_mda + self.within_1_mda
            }
            ErrorBucket::Within5Mda => {
                self.within_0_1_mda + self.within_0_5_mda + self.within_1_mda + self.within_5_mda
            }
            ErrorBucket::Above5Mda => self.above_5_mda,
            ErrorBucket::Within0_5Ppm => self.within_0_5_ppm,
            ErrorBucket::Within1Ppm => self.within_0_5_ppm + self.within_1_ppm,
            ErrorBucket::Within5Ppm => self.within_0_5_ppm + self.within_1_ppm + self.within_5_ppm,
            ErrorBucket::Within10Ppm => {
                self.within_0_5_ppm + self.within_1_ppm + self.within_5_ppm + self.within_10_ppm
            }
            ErrorBucket::Above10Ppm => self.above_10_ppm,
        }
    }

    /// Unparsable SMILES warnings sorted by descending count then lexicographically.
    #[must_use]
    pub fn sorted_unparsed_warnings(&self) -> Vec<(&str, &WarningDetail)> {
        let mut items: Vec<_> = self
            .unparsed_smiles_warnings
            .iter()
            .map(|(k, v)| (k.as_str(), v))
            .collect();
        items.sort_by(|(ls, ld), (rs, rd)| rd.count.cmp(&ld.count).then_with(|| ls.cmp(rs)));
        items
    }

    /// Unrecognized adducts sorted by descending count then lexicographically.
    #[must_use]
    pub fn sorted_unrecognized_adducts(&self) -> Vec<(&str, usize)> {
        let mut items: Vec<_> = self
            .unrecognized_adducts
            .iter()
            .map(|(k, &v)| (k.as_str(), v))
            .collect();
        items.sort_by(|(la, lc), (ra, rc)| rc.cmp(lc).then_with(|| la.cmp(ra)));
        items
    }

    /// High-error SMILES (`> 0.01 Da`) sorted by descending max error, count, then SMILES.
    #[must_use]
    pub fn sorted_high_error_smiles(&self) -> Vec<(&str, &HighErrorSmilesDetail)> {
        let mut items: Vec<_> = self
            .high_error_smiles
            .iter()
            .map(|(k, v)| (k.as_str(), v))
            .collect();
        items.sort_by(|(ls, ld), (rs, rd)| {
            rd.max_abs_error_da
                .unwrap_or_default()
                .total_cmp(&ld.max_abs_error_da.unwrap_or_default())
                .then_with(|| rd.count.cmp(&ld.count))
                .then_with(|| ls.cmp(rs))
        });
        items
    }
}
