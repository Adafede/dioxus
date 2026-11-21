// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the mgf-precursor-erro-rs project

// The statistics: an `f64` that can go in a `BinaryHeap`, the running median
// that uses it, and the `PrecursorStats` accumulator both feed.
//
// Extracted from `mod.rs` for the same reason as `plot.rs`. It depends on
// `plot` for the two fields that are histograms and the sample points; those
// are the only names it borrows.
// Only `PlotPoint` and the two `*Detail` maps are available on the host. The
// rest -- the histograms, the sample and measurement types, the two caps -- are
// wasm-gated, because the plotters that consume them are.
use super::plot::{
    ErrorMeasurement, HighErrorSmilesDetail, HighErrorSmilesUpdate, MAX_ECDF_POINTS,
    MAX_PLOT_POINTS, PlotPoint, WarningDetail, usize_to_f64,
};
use std::cmp::Reverse;
use std::collections::BTreeMap;
use std::collections::BinaryHeap;

#[derive(Clone, Copy, Debug)]
pub(crate) struct OrderedF64(pub f64);

impl PartialEq for OrderedF64 {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl Eq for OrderedF64 {}

impl Ord for OrderedF64 {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.total_cmp(&other.0)
    }
}

impl PartialOrd for OrderedF64 {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Clone, Debug, Default)]
pub(crate) struct MedianTracker {
    lower: BinaryHeap<OrderedF64>,
    upper: BinaryHeap<Reverse<OrderedF64>>,
}

impl MedianTracker {
    pub(crate) fn push(&mut self, value: f64) {
        if self.lower.peek().is_none_or(|entry| value <= entry.0) {
            self.lower.push(OrderedF64(value));
        } else {
            self.upper.push(Reverse(OrderedF64(value)));
        }

        if self.lower.len() > self.upper.len() + 1 {
            if let Some(OrderedF64(value)) = self.lower.pop() {
                self.upper.push(Reverse(OrderedF64(value)));
            }
        } else if self.upper.len() > self.lower.len()
            && let Some(Reverse(OrderedF64(value))) = self.upper.pop()
        {
            self.lower.push(OrderedF64(value));
        }
    }

    #[cfg(test)]
    pub(crate) fn merge(&mut self, other: &Self) {
        for &OrderedF64(value) in &other.lower {
            self.push(value);
        }
        for &Reverse(OrderedF64(value)) in &other.upper {
            self.push(value);
        }
    }

    #[must_use]
    pub(crate) fn median(&self) -> f64 {
        if self.lower.is_empty() && self.upper.is_empty() {
            0.0
        } else if self.lower.len() > self.upper.len() {
            self.lower.peek().map_or(0.0, |entry| entry.0)
        } else {
            let lower = self.lower.peek().map_or(0.0, |entry| entry.0);
            let upper = self.upper.peek().map_or(0.0, |entry| entry.0.0);
            lower.midpoint(upper)
        }
    }
}

#[derive(Clone, Debug, Default)]
pub(crate) struct PrecursorStats {
    pub spectra: usize,
    pub total_spectra: usize,
    pub skipped_spectra: usize,
    pub spectra_with_reference_mass: usize,
    pub unparsed_smiles: usize,
    pub unparsed_smiles_warnings: BTreeMap<String, WarningDetail>,
    pub observed_precursor_min: f64,
    pub observed_precursor_max: f64,
    pub observed_precursor_mean: f64,
    pub observed_precursor_median: f64,
    pub observed_precursor_median_tracker: MedianTracker,
    pub abs_error_da_min: f64,
    pub abs_error_da_max: f64,
    pub abs_error_da_mean: f64,
    pub abs_error_da_median: f64,
    pub abs_error_da_median_tracker: MedianTracker,
    pub abs_error_da_rms: f64,
    pub abs_error_ppm_min: f64,
    pub abs_error_ppm_max: f64,
    pub abs_error_ppm_mean: f64,
    pub abs_error_ppm_median: f64,
    pub abs_error_ppm_median_tracker: MedianTracker,
    pub abs_error_ppm_rms: f64,
    pub within_0_1_da: usize,
    pub within_0_5_da: usize,
    pub within_1_da: usize,
    pub within_5_da: usize,
    pub above_5_da: usize,
    pub within_0_5_ppm: usize,
    pub within_1_ppm: usize,
    pub within_5_ppm: usize,
    pub within_10_ppm: usize,
    pub above_10_ppm: usize,
    pub absolute_error_da_values: Vec<f64>,
    pub absolute_error_ppm_values: Vec<f64>,
    pub absolute_error_da_sample_seen: usize,
    pub absolute_error_ppm_sample_seen: usize,
    pub plot_points: Vec<PlotPoint>,
    pub plot_point_stream_seen: usize,
    pub unrecognized_adducts: BTreeMap<String, usize>,
    pub high_error_smiles: BTreeMap<String, HighErrorSmilesDetail>,
    pub raw_retention_capped: bool,
}

fn push_reservoir<T>(value: T, values: &mut Vec<T>, seen: &mut usize, max_points: usize) {
    *seen = seen.saturating_add(1);
    if values.len() < max_points {
        values.push(value);
        return;
    }
    let stream_index = u64::try_from(*seen).unwrap_or(u64::MAX);
    let replacement_index = usize::try_from(
        (stream_index
            .wrapping_mul(0x9e37_79b9_7f4a_7c15)
            .wrapping_add(0xbf58_476d_1ce4_e5b9))
            % stream_index,
    )
    .unwrap_or(0);
    if replacement_index < max_points
        && let Some(existing) = values.get_mut(replacement_index)
    {
        *existing = value;
    }
}

impl PrecursorStats {
    pub(super) fn push_sampled_value(value: f64, values: &mut Vec<f64>, seen: &mut usize) {
        push_reservoir(value, values, seen, MAX_ECDF_POINTS);
    }

    pub(super) fn push_plot_point(&mut self, point: PlotPoint) {
        push_reservoir(
            point,
            &mut self.plot_points,
            &mut self.plot_point_stream_seen,
            MAX_PLOT_POINTS,
        );
    }

    pub(crate) fn record_error(&mut self, measurement: ErrorMeasurement<'_>) {
        let ErrorMeasurement {
            abs_error_da,
            abs_ppm,
            adduct_family,
            ppm_error,
            signed_error_da,
            pepmass_header,
            ms2_precursor_peak,
            smiles,
            calculated_mass,
            expected_mass,
            formula,
        } = measurement;

        let prev_spectra = usize_to_f64(self.spectra);
        self.total_spectra += 1;
        self.spectra += 1;
        self.spectra_with_reference_mass += 1;
        let total_spectra = usize_to_f64(self.spectra);

        if prev_spectra == 0.0 {
            self.observed_precursor_min = pepmass_header;
            self.observed_precursor_max = pepmass_header;
            self.abs_error_da_min = abs_error_da;
            self.abs_error_da_max = abs_error_da;
            self.abs_error_ppm_min = abs_ppm;
            self.abs_error_ppm_max = abs_ppm;
        } else {
            self.observed_precursor_min = self.observed_precursor_min.min(pepmass_header);
            self.observed_precursor_max = self.observed_precursor_max.max(pepmass_header);
            self.abs_error_da_min = self.abs_error_da_min.min(abs_error_da);
            self.abs_error_da_max = self.abs_error_da_max.max(abs_error_da);
            self.abs_error_ppm_min = self.abs_error_ppm_min.min(abs_ppm);
            self.abs_error_ppm_max = self.abs_error_ppm_max.max(abs_ppm);
        }

        self.observed_precursor_mean = self
            .observed_precursor_mean
            .mul_add(prev_spectra, pepmass_header)
            / total_spectra;
        self.abs_error_da_mean =
            self.abs_error_da_mean.mul_add(prev_spectra, abs_error_da) / total_spectra;
        let current_da_rms_sq = self.abs_error_da_rms * self.abs_error_da_rms;
        self.abs_error_da_rms =
            (current_da_rms_sq.mul_add(prev_spectra, abs_error_da * abs_error_da) / total_spectra)
                .sqrt();

        self.abs_error_ppm_mean =
            self.abs_error_ppm_mean.mul_add(prev_spectra, abs_ppm) / total_spectra;
        let current_ppm_rms_sq = self.abs_error_ppm_rms * self.abs_error_ppm_rms;
        self.abs_error_ppm_rms =
            (current_ppm_rms_sq.mul_add(prev_spectra, abs_ppm * abs_ppm) / total_spectra).sqrt();

        Self::push_sampled_value(
            abs_error_da,
            &mut self.absolute_error_da_values,
            &mut self.absolute_error_da_sample_seen,
        );
        Self::push_sampled_value(
            abs_ppm,
            &mut self.absolute_error_ppm_values,
            &mut self.absolute_error_ppm_sample_seen,
        );
        self.observed_precursor_median_tracker.push(pepmass_header);
        self.abs_error_da_median_tracker.push(abs_error_da);
        self.abs_error_ppm_median_tracker.push(abs_ppm);
        self.observed_precursor_median = self.observed_precursor_median_tracker.median();
        self.abs_error_da_median = self.abs_error_da_median_tracker.median();
        self.abs_error_ppm_median = self.abs_error_ppm_median_tracker.median();

        let abs_error_milli_da = abs_error_da * 1000.0;
        self.update_error_buckets(abs_error_milli_da, abs_ppm);
        self.update_high_error_smiles(HighErrorSmilesUpdate {
            abs_error_da,
            abs_ppm,
            pepmass_header,
            smiles,
            calculated_mass,
            expected_mass,
            formula,
        });

        self.push_plot_point(PlotPoint {
            adduct_family,
            pepmass_header,
            ms2_precursor_peak,
            signed_error_da,
            signed_error_ppm: ppm_error,
            expected_mass,
        });
    }

    fn update_error_buckets(&mut self, abs_error_milli_da: f64, abs_ppm: f64) {
        match abs_error_milli_da {
            value if value <= 0.1 => self.within_0_1_da = self.within_0_1_da.saturating_add(1),
            value if value <= 0.5 => self.within_0_5_da = self.within_0_5_da.saturating_add(1),
            value if value <= 1.0 => self.within_1_da = self.within_1_da.saturating_add(1),
            value if value <= 5.0 => self.within_5_da = self.within_5_da.saturating_add(1),
            _ => self.above_5_da = self.above_5_da.saturating_add(1),
        }

        match abs_ppm {
            value if value <= 0.5 => self.within_0_5_ppm = self.within_0_5_ppm.saturating_add(1),
            value if value <= 1.0 => self.within_1_ppm = self.within_1_ppm.saturating_add(1),
            value if value <= 5.0 => self.within_5_ppm = self.within_5_ppm.saturating_add(1),
            value if value <= 10.0 => self.within_10_ppm = self.within_10_ppm.saturating_add(1),
            _ => self.above_10_ppm = self.above_10_ppm.saturating_add(1),
        }
    }

    fn update_high_error_smiles(&mut self, update: HighErrorSmilesUpdate<'_>) {
        let Some(smiles) = update
            .smiles
            .map(str::trim)
            .filter(|value| !value.is_empty())
        else {
            return;
        };
        if update.abs_error_da <= 0.01 {
            return;
        }

        if let Some(entry) = self.high_error_smiles.get_mut(smiles) {
            entry.count = entry.count.saturating_add(1);
            if entry.calculated_mass.is_none() && update.calculated_mass.is_some() {
                entry.calculated_mass = update.calculated_mass;
            }
            if entry.expected_mass.is_none() && update.expected_mass.is_some() {
                entry.expected_mass = update.expected_mass;
            }
            if entry.formula.is_none() {
                entry.formula = update.formula.map(str::to_string);
            }
            if entry.observed_precursor_mz.is_none() {
                entry.observed_precursor_mz = Some(update.pepmass_header);
            }
            if entry
                .max_abs_error_da
                .is_none_or(|existing| update.abs_error_da > existing)
            {
                entry.max_abs_error_da = Some(update.abs_error_da);
                entry.max_abs_error_ppm = Some(update.abs_ppm);
                if update.calculated_mass.is_some() {
                    entry.calculated_mass = update.calculated_mass;
                }
                if update.expected_mass.is_some() {
                    entry.expected_mass = update.expected_mass;
                }
                entry.observed_precursor_mz = Some(update.pepmass_header);
            }
            return;
        }

        self.high_error_smiles.insert(
            smiles.to_owned(),
            HighErrorSmilesDetail {
                count: 1,
                calculated_mass: update.calculated_mass,
                expected_mass: update.expected_mass,
                formula: update.formula.map(str::to_string),
                max_abs_error_da: Some(update.abs_error_da),
                max_abs_error_ppm: Some(update.abs_ppm),
                observed_precursor_mz: Some(update.pepmass_header),
            },
        );
    }
}
