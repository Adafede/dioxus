// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the mgf-precursor-erro-rs project

//! Streaming `RecalibrationStats` accumulator and synchronized reservoir sampling.

use super::model::{RecalibrationMeasurement, RecalibrationStats, RunningStats};
use super::{compute_max_abs, compute_mean, compute_rms};
use crate::numeric::{reservoir_index, usize_to_f64, write_reservoir_slot};

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
            let finite: Vec<f64> = samples.iter().copied().filter(|v| v.is_finite()).collect();
            let mae = if finite.is_empty() {
                0.0
            } else {
                finite.iter().map(|v| v.abs()).sum::<f64>() / usize_to_f64(finite.len())
            };
            return (
                compute_mean(&finite),
                compute_rms(&finite),
                mae,
                compute_max_abs(&finite),
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

impl RecalibrationStats {
    /// Creates an empty [`RecalibrationStats`] accumulator.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Pushes pre/post error pairs into the accumulator.
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
        self.stream_ppm_before.push(error_ppm_before);
        self.stream_ppm_after.push(error_ppm_after);

        if self.sample_count < max_samples {
            self.error_ppm_before.push(error_ppm_before);
            self.error_ppm_after.push(error_ppm_after);
            self.error_da_before.push(error_da_before);
            self.error_da_after.push(error_da_after);
            self.sample_count += 1;
        } else {
            let idx = reservoir_index(self.total_count);
            if idx < max_samples {
                write_reservoir_slot(&mut self.error_ppm_before, idx, error_ppm_before);
                write_reservoir_slot(&mut self.error_ppm_after, idx, error_ppm_after);
                write_reservoir_slot(&mut self.error_da_before, idx, error_da_before);
                write_reservoir_slot(&mut self.error_da_after, idx, error_da_after);
            }
        }
        if let Some(family) = adduct_family {
            self.record_adduct_error(family, error_ppm_before);
        }
    }

    /// Pushes a complete measurement paired with its MS1 reference baseline.
    pub(crate) fn push_measurement(&mut self, measurement: RecalibrationMeasurement<'_>) {
        self.total_count += 1;
        self.stream_ppm_before.push(measurement.error_ppm_before);
        self.stream_ppm_after.push(measurement.error_ppm_after);

        if self.sample_count < measurement.max_samples {
            self.error_ppm_ms1.push(measurement.error_ppm_ms1);
            self.error_ppm_before.push(measurement.error_ppm_before);
            self.error_ppm_after.push(measurement.error_ppm_after);
            self.error_da_ms1.push(measurement.error_da_ms1);
            self.error_da_before.push(measurement.error_da_before);
            self.error_da_after.push(measurement.error_da_after);
            self.sample_count += 1;
        } else {
            let idx = reservoir_index(self.total_count);
            if idx < measurement.max_samples {
                write_reservoir_slot(&mut self.error_ppm_ms1, idx, measurement.error_ppm_ms1);
                write_reservoir_slot(
                    &mut self.error_ppm_before,
                    idx,
                    measurement.error_ppm_before,
                );
                write_reservoir_slot(&mut self.error_ppm_after, idx, measurement.error_ppm_after);
                write_reservoir_slot(&mut self.error_da_ms1, idx, measurement.error_da_ms1);
                write_reservoir_slot(&mut self.error_da_before, idx, measurement.error_da_before);
                write_reservoir_slot(&mut self.error_da_after, idx, measurement.error_da_after);
            }
        }
        if let Some(family) = measurement.adduct_family {
            self.record_adduct_error(family, measurement.error_ppm_before);
        }
    }

    fn record_adduct_error(&mut self, family: &str, err_before: f64) {
        self.error_by_adduct_before
            .entry(family.to_string())
            .or_default()
            .push(err_before);
    }

    /// Finalizes stream-wide summary statistics (mean, MAE, RMS, max).
    pub fn compute_statistics(&mut self) {
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
    }

    /// MAE reduction in ppm after recalibration.
    #[must_use]
    pub fn mean_error_improvement_ppm(&self) -> f64 {
        self.mae_error_ppm_before - self.mae_error_ppm_after
    }

    /// RMS reduction in ppm after recalibration.
    #[must_use]
    pub fn rms_error_improvement_ppm(&self) -> f64 {
        self.rms_error_ppm_before - self.rms_error_ppm_after
    }
}
