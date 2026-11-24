// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the mgf-precursor-erro-rs project

//! Streaming `RecalibrationStats` accumulator and synchronized reservoir sampling.

use super::model::RunningStats;
use super::{
    RecalibrationMeasurement, RecalibrationStats, compute_max_abs, compute_mean, compute_rms,
};
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
        self.stream_da_before.push(error_da_before);
        self.stream_da_after.push(error_da_after);

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
            self.record_adduct_errors(family, error_ppm_before, error_ppm_after);
        }
    }

    /// Pushes a complete four-stage measurement with aligned precursor `m/z` values.
    pub fn push_measurement(&mut self, measurement: RecalibrationMeasurement<'_>) {
        self.total_count += 1;
        self.stream_ppm_ms1.push(measurement.error_ppm_ms1);
        self.stream_delta_ppm.push(measurement.delta_ppm_ms2_ms1);
        self.stream_ppm_before.push(measurement.error_ppm_before);
        self.stream_ppm_after.push(measurement.error_ppm_after);
        self.stream_da_ms1.push(measurement.error_da_ms1);
        self.stream_delta_da.push(measurement.delta_da_ms2_ms1);
        self.stream_da_before.push(measurement.error_da_before);
        self.stream_da_after.push(measurement.error_da_after);

        if self.sample_count < measurement.max_samples {
            self.error_ppm_ms1.push(measurement.error_ppm_ms1);
            self.delta_ppm_ms2_ms1.push(measurement.delta_ppm_ms2_ms1);
            self.error_ppm_before.push(measurement.error_ppm_before);
            self.error_ppm_after.push(measurement.error_ppm_after);
            self.error_da_ms1.push(measurement.error_da_ms1);
            self.delta_da_ms2_ms1.push(measurement.delta_da_ms2_ms1);
            self.error_da_before.push(measurement.error_da_before);
            self.error_da_after.push(measurement.error_da_after);
            self.precursor_ms1_values.push(measurement.precursor_ms1);
            self.precursor_ms2_before
                .push(measurement.precursor_ms2_before);
            self.precursor_ms2_after
                .push(measurement.precursor_ms2_after);
            self.sample_count += 1;
        } else {
            let idx = reservoir_index(self.total_count);
            if idx < measurement.max_samples {
                write_reservoir_slot(&mut self.error_ppm_ms1, idx, measurement.error_ppm_ms1);
                write_reservoir_slot(
                    &mut self.delta_ppm_ms2_ms1,
                    idx,
                    measurement.delta_ppm_ms2_ms1,
                );
                write_reservoir_slot(
                    &mut self.error_ppm_before,
                    idx,
                    measurement.error_ppm_before,
                );
                write_reservoir_slot(&mut self.error_ppm_after, idx, measurement.error_ppm_after);
                write_reservoir_slot(&mut self.error_da_ms1, idx, measurement.error_da_ms1);
                write_reservoir_slot(
                    &mut self.delta_da_ms2_ms1,
                    idx,
                    measurement.delta_da_ms2_ms1,
                );
                write_reservoir_slot(&mut self.error_da_before, idx, measurement.error_da_before);
                write_reservoir_slot(&mut self.error_da_after, idx, measurement.error_da_after);
                write_reservoir_slot(
                    &mut self.precursor_ms1_values,
                    idx,
                    measurement.precursor_ms1,
                );
                write_reservoir_slot(
                    &mut self.precursor_ms2_before,
                    idx,
                    measurement.precursor_ms2_before,
                );
                write_reservoir_slot(
                    &mut self.precursor_ms2_after,
                    idx,
                    measurement.precursor_ms2_after,
                );
            }
        }
        if let Some(family) = measurement.adduct_family {
            self.record_adduct_errors(
                family,
                measurement.error_ppm_before,
                measurement.error_ppm_after,
            );
        }
    }

    fn record_adduct_errors(&mut self, family: &str, err_before: f64, err_after: f64) {
        self.error_by_adduct_before
            .entry(family.to_string())
            .or_default()
            .push(err_before);
        self.error_by_adduct_after
            .entry(family.to_string())
            .or_default()
            .push(err_after);
    }

    /// Finalizes stream-wide summary statistics (mean, MAE, RMS, max).
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

    /// MAE reduction in Da after recalibration.
    #[must_use]
    pub fn mean_error_improvement_da(&self) -> f64 {
        self.mae_error_da_before - self.mae_error_da_after
    }

    /// RMS reduction in Da after recalibration.
    #[must_use]
    pub fn rms_error_improvement_da(&self) -> f64 {
        self.rms_error_da_before - self.rms_error_da_after
    }
}
