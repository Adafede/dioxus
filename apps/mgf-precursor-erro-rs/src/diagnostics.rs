// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the mgf-precursor-erro-rs project

/// Diagnostics and plotting utilities for precursor error analysis before and after recalibration.
///
/// This module provides tools to compute and visualize precursor error distributions,
/// allowing users to assess the impact of recalibration on mass accuracy.
use std::collections::BTreeMap;

use crate::plotting::data::usize_to_f64;

/// Statistics about precursor errors for diagnostic purposes.
#[derive(Clone, Debug)]
pub struct RecalibrationStats {
    /// Precursor errors (in ppm) from PEPMASS vs theoretical.
    pub error_ppm_ms1: Vec<f64>,

    /// Precursor errors (in Da) from PEPMASS vs theoretical.
    pub error_da_ms1: Vec<f64>,

    /// Calibration delta (MS2 - MS1) in ppm, sampled for plotting.
    pub delta_ppm_ms2_ms1: Vec<f64>,

    /// Calibration delta (MS2 - MS1) in Da, sampled for plotting.
    pub delta_da_ms2_ms1: Vec<f64>,

    /// Precursor errors (in ppm) before recalibration, sampled for plotting.
    pub error_ppm_before: Vec<f64>,

    /// Precursor errors (in ppm) after recalibration, sampled for plotting.
    pub error_ppm_after: Vec<f64>,

    /// Precursor errors (in Da) before recalibration, sampled for plotting.
    pub error_da_before: Vec<f64>,

    /// Precursor errors (in Da) after recalibration, sampled for plotting.
    pub error_da_after: Vec<f64>,

    /// PEPMASS header precursor m/z values (estimated MS1).
    pub precursor_ms1_values: Vec<f64>,

    /// Observed precursor m/z values from MS2 before recalibration.
    pub precursor_ms2_before: Vec<f64>,

    /// Observed precursor m/z values from MS2 after recalibration.
    pub precursor_ms2_after: Vec<f64>,

    /// Number of precursors included in the diagnostic sample.
    pub sample_count: usize,

    /// Total number of precursors processed (before sampling).
    pub total_count: usize,

    /// Mean error in ppm (Stage 1: PEPMASS vs theory).
    pub mean_error_ppm_ms1: f64,

    /// Mean calibration delta (Stage 3: MS2 - MS1) in ppm.
    pub mean_delta_ppm_ms2_ms1: f64,

    /// Mean error in ppm before recalibration.
    pub mean_error_ppm_before: f64,

    /// Mean error in ppm after recalibration.
    pub mean_error_ppm_after: f64,

    /// Mean error in Da (Stage 1: PEPMASS vs theory).
    pub mean_error_da_ms1: f64,

    /// Mean calibration delta (Stage 3: MS2 - MS1) in Da.
    pub mean_delta_da_ms2_ms1: f64,

    /// Mean error in Da before recalibration.
    pub mean_error_da_before: f64,

    /// Mean error in Da after recalibration.
    pub mean_error_da_after: f64,

    /// Mean absolute error in ppm before recalibration.
    pub mae_error_ppm_before: f64,

    /// Mean absolute error in ppm after recalibration.
    pub mae_error_ppm_after: f64,

    /// Mean absolute error in Da before recalibration.
    pub mae_error_da_before: f64,

    /// Mean absolute error in Da after recalibration.
    pub mae_error_da_after: f64,

    /// RMS (root-mean-square) error in ppm (Stage 1: PEPMASS vs theory).
    pub rms_error_ppm_ms1: f64,

    /// RMS calibration delta (Stage 3: MS2 - MS1) in ppm.
    pub rms_delta_ppm_ms2_ms1: f64,

    /// RMS (root-mean-square) error in ppm before recalibration.
    pub rms_error_ppm_before: f64,

    /// RMS (root-mean-square) error in ppm after recalibration.
    pub rms_error_ppm_after: f64,

    /// RMS (root-mean-square) error in Da (Stage 1: PEPMASS vs theory).
    pub rms_error_da_ms1: f64,

    /// RMS calibration delta (Stage 3: MS2 - MS1) in Da.
    pub rms_delta_da_ms2_ms1: f64,

    /// RMS (root-mean-square) error in Da before recalibration.
    pub rms_error_da_before: f64,

    /// RMS (root-mean-square) error in Da after recalibration.
    pub rms_error_da_after: f64,

    /// Maximum absolute error in ppm before recalibration.
    pub max_abs_error_ppm_before: f64,

    /// Maximum absolute error in ppm after recalibration.
    pub max_abs_error_ppm_after: f64,

    /// Adduct family categorization for errors before recalibration.
    pub error_by_adduct_before: BTreeMap<String, Vec<f64>>,

    /// Adduct family categorization for errors after recalibration.
    pub error_by_adduct_after: BTreeMap<String, Vec<f64>>,

    stream_ppm_ms1: RunningStats,
    stream_delta_ppm: RunningStats,
    stream_ppm_before: RunningStats,
    stream_ppm_after: RunningStats,
    stream_da_ms1: RunningStats,
    stream_delta_da: RunningStats,
    stream_da_before: RunningStats,
    stream_da_after: RunningStats,
}

#[derive(Clone, Copy, Debug, Default)]
struct RunningStats {
    count: usize,
    sum: f64,
    sum_abs: f64,
    sum_sq: f64,
    max_abs: f64,
}

impl RunningStats {
    fn push(&mut self, value: f64) {
        if !value.is_finite() {
            return;
        }
        self.count += 1;
        self.sum += value;
        let abs = value.abs();
        self.sum_abs += abs;
        self.sum_sq = value.mul_add(value, self.sum_sq);
        self.max_abs = self.max_abs.max(abs);
    }

    fn finish(self, samples: &[f64]) -> (f64, f64, f64, f64) {
        if self.count == 0 {
            let mae = if samples.is_empty() {
                0.0
            } else {
                samples.iter().map(|v| v.abs()).sum::<f64>() / usize_to_f64(samples.len())
            };
            return (
                compute_mean(samples),
                compute_rms(samples),
                mae,
                compute_max_abs(samples),
            );
        }
        let n = usize_to_f64(self.count);
        (
            self.sum / n,
            (self.sum_sq / n).sqrt(),
            self.sum_abs / n,
            self.max_abs,
        )
    }
}

#[derive(Clone, Copy, Debug)]
pub struct RecalibrationMeasurement<'a> {
    pub error_ppm_ms1: f64,
    pub delta_ppm_ms2_ms1: f64,
    pub error_ppm_before: f64,
    pub error_ppm_after: f64,
    pub error_da_ms1: f64,
    pub delta_da_ms2_ms1: f64,
    pub error_da_before: f64,
    pub error_da_after: f64,
    pub precursor_ms1: f64,
    pub precursor_ms2_before: f64,
    pub precursor_ms2_after: f64,
    pub adduct_family: Option<&'a str>,
    pub max_samples: usize,
}

impl Default for RecalibrationStats {
    fn default() -> Self {
        Self {
            error_ppm_ms1: Vec::new(),
            error_da_ms1: Vec::new(),
            delta_ppm_ms2_ms1: Vec::new(),
            delta_da_ms2_ms1: Vec::new(),
            error_ppm_before: Vec::new(),
            error_ppm_after: Vec::new(),
            error_da_before: Vec::new(),
            error_da_after: Vec::new(),
            precursor_ms1_values: Vec::new(),
            precursor_ms2_before: Vec::new(),
            precursor_ms2_after: Vec::new(),
            sample_count: 0,
            total_count: 0,
            mean_error_ppm_ms1: 0.0,
            mean_delta_ppm_ms2_ms1: 0.0,
            mean_error_ppm_before: 0.0,
            mean_error_ppm_after: 0.0,
            mean_error_da_ms1: 0.0,
            mean_delta_da_ms2_ms1: 0.0,
            mean_error_da_before: 0.0,
            mean_error_da_after: 0.0,
            mae_error_ppm_before: 0.0,
            mae_error_ppm_after: 0.0,
            mae_error_da_before: 0.0,
            mae_error_da_after: 0.0,
            rms_error_ppm_ms1: 0.0,
            rms_delta_ppm_ms2_ms1: 0.0,
            rms_error_ppm_before: 0.0,
            rms_error_ppm_after: 0.0,
            rms_error_da_ms1: 0.0,
            rms_delta_da_ms2_ms1: 0.0,
            rms_error_da_before: 0.0,
            rms_error_da_after: 0.0,
            max_abs_error_ppm_before: 0.0,
            max_abs_error_ppm_after: 0.0,
            error_by_adduct_before: BTreeMap::new(),
            error_by_adduct_after: BTreeMap::new(),
            stream_ppm_ms1: RunningStats::default(),
            stream_delta_ppm: RunningStats::default(),
            stream_ppm_before: RunningStats::default(),
            stream_ppm_after: RunningStats::default(),
            stream_da_ms1: RunningStats::default(),
            stream_delta_da: RunningStats::default(),
            stream_da_before: RunningStats::default(),
            stream_da_after: RunningStats::default(),
        }
    }
}

impl RecalibrationStats {
    /// Creates a new empty diagnostics structure.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    fn record_before_after_stream(
        &mut self,
        error_ppm_before: f64,
        error_ppm_after: f64,
        error_da_before: f64,
        error_da_after: f64,
    ) {
        self.stream_ppm_before.push(error_ppm_before);
        self.stream_ppm_after.push(error_ppm_after);
        self.stream_da_before.push(error_da_before);
        self.stream_da_after.push(error_da_after);
    }

    /// Adds a precursor error measurement to the diagnostics.
    ///
    /// # Arguments
    /// * `error_ppm_before` - Error in ppm before recalibration
    /// * `error_ppm_after` - Error in ppm after recalibration
    /// * `error_da_before` - Error in Da before recalibration
    /// * `error_da_after` - Error in Da after recalibration
    /// * `adduct_family` - Optional adduct family label for categorization
    /// * `max_samples` - Maximum number of samples to keep
    pub fn push_error(
        &mut self,
        error_ppm_before: f64,
        error_ppm_after: f64,
        error_da_before: f64,
        error_da_after: f64,
        adduct_family: Option<&str>,
        max_samples: usize,
    ) {
        self.total_count += 1;
        self.record_before_after_stream(
            error_ppm_before,
            error_ppm_after,
            error_da_before,
            error_da_after,
        );

        // Sample for plotting
        if self.sample_count < max_samples {
            self.error_ppm_before.push(error_ppm_before);
            self.error_ppm_after.push(error_ppm_after);
            self.error_da_before.push(error_da_before);
            self.error_da_after.push(error_da_after);
            self.sample_count += 1;
        } else {
            // Reservoir sampling for additional points
            let index = reservoir_index(self.total_count);
            if index < max_samples {
                write_reservoir_slot(&mut self.error_ppm_before, index, error_ppm_before);
                write_reservoir_slot(&mut self.error_ppm_after, index, error_ppm_after);
                write_reservoir_slot(&mut self.error_da_before, index, error_da_before);
                write_reservoir_slot(&mut self.error_da_after, index, error_da_after);
            }
        }

        // Track by adduct family if provided
        if let Some(family) = adduct_family {
            self.record_adduct_errors(family, error_ppm_before, error_ppm_after);
        }
    }

    /// Adds a complete measurement including precursor m/z values.
    pub fn push_measurement(&mut self, measurement: RecalibrationMeasurement<'_>) {
        let RecalibrationMeasurement {
            error_ppm_ms1,
            delta_ppm_ms2_ms1,
            error_ppm_before,
            error_ppm_after,
            error_da_ms1,
            delta_da_ms2_ms1,
            error_da_before,
            error_da_after,
            precursor_ms1,
            precursor_ms2_before,
            precursor_ms2_after,
            adduct_family,
            max_samples,
        } = measurement;

        self.total_count += 1;
        self.stream_ppm_ms1.push(error_ppm_ms1);
        self.stream_delta_ppm.push(delta_ppm_ms2_ms1);
        self.stream_da_ms1.push(error_da_ms1);
        self.stream_delta_da.push(delta_da_ms2_ms1);
        self.record_before_after_stream(
            error_ppm_before,
            error_ppm_after,
            error_da_before,
            error_da_after,
        );

        // Sample for plotting
        if self.sample_count < max_samples {
            self.error_ppm_ms1.push(error_ppm_ms1);
            self.delta_ppm_ms2_ms1.push(delta_ppm_ms2_ms1);
            self.error_ppm_before.push(error_ppm_before);
            self.error_ppm_after.push(error_ppm_after);
            self.error_da_ms1.push(error_da_ms1);
            self.delta_da_ms2_ms1.push(delta_da_ms2_ms1);
            self.error_da_before.push(error_da_before);
            self.error_da_after.push(error_da_after);
            self.precursor_ms1_values.push(precursor_ms1);
            self.precursor_ms2_before.push(precursor_ms2_before);
            self.precursor_ms2_after.push(precursor_ms2_after);
            self.sample_count += 1;
        } else {
            // Reservoir sampling for additional points
            let index = reservoir_index(self.total_count);
            if index < max_samples {
                write_reservoir_slot(&mut self.error_ppm_ms1, index, error_ppm_ms1);
                write_reservoir_slot(&mut self.delta_ppm_ms2_ms1, index, delta_ppm_ms2_ms1);
                write_reservoir_slot(&mut self.error_ppm_before, index, error_ppm_before);
                write_reservoir_slot(&mut self.error_ppm_after, index, error_ppm_after);
                write_reservoir_slot(&mut self.error_da_ms1, index, error_da_ms1);
                write_reservoir_slot(&mut self.delta_da_ms2_ms1, index, delta_da_ms2_ms1);
                write_reservoir_slot(&mut self.error_da_before, index, error_da_before);
                write_reservoir_slot(&mut self.error_da_after, index, error_da_after);
                write_reservoir_slot(&mut self.precursor_ms1_values, index, precursor_ms1);
                write_reservoir_slot(&mut self.precursor_ms2_before, index, precursor_ms2_before);
                write_reservoir_slot(&mut self.precursor_ms2_after, index, precursor_ms2_after);
            }
        }

        // Track by adduct family if provided
        if let Some(family) = adduct_family {
            self.record_adduct_errors(family, error_ppm_before, error_ppm_after);
        }
    }

    fn record_adduct_errors(&mut self, family: &str, error_ppm_before: f64, error_ppm_after: f64) {
        if let Some(series) = self.error_by_adduct_before.get_mut(family) {
            series.push(error_ppm_before);
        } else {
            self.error_by_adduct_before
                .insert(family.to_string(), vec![error_ppm_before]);
        }
        if let Some(series) = self.error_by_adduct_after.get_mut(family) {
            series.push(error_ppm_after);
        } else {
            self.error_by_adduct_after
                .insert(family.to_string(), vec![error_ppm_after]);
        }
    }

    /// Computes summary statistics from the processed stream (falling back to
    /// sampled vectors if stats were populated directly).
    pub fn compute_statistics(&mut self) {
        (self.mean_error_ppm_ms1, self.rms_error_ppm_ms1, _, _) =
            self.stream_ppm_ms1.finish(&self.error_ppm_ms1);
        (
            self.mean_delta_ppm_ms2_ms1,
            self.rms_delta_ppm_ms2_ms1,
            _,
            _,
        ) = self.stream_delta_ppm.finish(&self.delta_ppm_ms2_ms1);
        (
            self.mean_error_ppm_before,
            self.rms_error_ppm_before,
            self.mae_error_ppm_before,
            self.max_abs_error_ppm_before,
        ) = self.stream_ppm_before.finish(&self.error_ppm_before);
        (
            self.mean_error_ppm_after,
            self.rms_error_ppm_after,
            self.mae_error_ppm_after,
            self.max_abs_error_ppm_after,
        ) = self.stream_ppm_after.finish(&self.error_ppm_after);

        (self.mean_error_da_ms1, self.rms_error_da_ms1, _, _) =
            self.stream_da_ms1.finish(&self.error_da_ms1);
        (self.mean_delta_da_ms2_ms1, self.rms_delta_da_ms2_ms1, _, _) =
            self.stream_delta_da.finish(&self.delta_da_ms2_ms1);
        (
            self.mean_error_da_before,
            self.rms_error_da_before,
            self.mae_error_da_before,
            _,
        ) = self.stream_da_before.finish(&self.error_da_before);
        (
            self.mean_error_da_after,
            self.rms_error_da_after,
            self.mae_error_da_after,
            _,
        ) = self.stream_da_after.finish(&self.error_da_after);
    }

    /// Returns the improvement in mean absolute error after recalibration (in ppm).
    #[must_use]
    pub fn mean_error_improvement_ppm(&self) -> f64 {
        self.mae_error_ppm_before - self.mae_error_ppm_after
    }

    /// Returns the improvement in RMS error after recalibration (in ppm).
    #[must_use]
    pub fn rms_error_improvement_ppm(&self) -> f64 {
        self.rms_error_ppm_before - self.rms_error_ppm_after
    }

    /// Returns the improvement in mean absolute error after recalibration (in Da).
    #[must_use]
    pub fn mean_error_improvement_da(&self) -> f64 {
        self.mae_error_da_before - self.mae_error_da_after
    }

    /// Returns the improvement in RMS error after recalibration (in Da).
    #[must_use]
    pub fn rms_error_improvement_da(&self) -> f64 {
        self.rms_error_da_before - self.rms_error_da_after
    }
}

/// Computes the mean of a slice of values.
fn compute_mean(values: &[f64]) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    let sum: f64 = values.iter().sum();
    sum / usize_to_f64(values.len())
}

/// Computes the root-mean-square (RMS) of absolute values.
fn compute_rms(values: &[f64]) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    let sum_sq: f64 = values.iter().map(|v| v * v).sum();
    (sum_sq / usize_to_f64(values.len())).sqrt()
}

/// Computes the maximum absolute value.
fn compute_max_abs(values: &[f64]) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    values.iter().map(|v| v.abs()).fold(0.0, f64::max)
}

fn reservoir_index(seen: usize) -> usize {
    let stream_index = u64::try_from(seen).unwrap_or(u64::MAX).max(1);
    let mixed = stream_index
        .wrapping_mul(0x9e37_79b9_7f4a_7c15)
        .wrapping_add(0xbf58_476d_1ce4_e5b9);
    usize::try_from(mixed % stream_index).unwrap_or(0)
}

/// Overwrites reservoir slot `index` of one error series.
///
/// Every series reaching the reservoir path has been pushed exactly once per
/// `sample_count` increment, so it already holds at least `max_samples` values
/// by the time `index < max_samples` holds — the slot exists. Routing all ten
/// writes through this helper keeps that invariant enforced at the point of
/// the write instead of relying on a bound checked elsewhere.
fn write_reservoir_slot(series: &mut [f64], index: usize, value: f64) {
    if let Some(slot) = series.get_mut(index) {
        *slot = value;
    }
}

#[cfg(test)]
#[allow(clippy::indexing_slicing)]
// Tests reach into the vectors they just filled themselves; indexing past
// their own literal fixture means the behaviour under test regressed, and the
// panic is a perfectly good report of that.
mod tests;
