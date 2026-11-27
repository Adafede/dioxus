// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the mgf-precursor-erro-rs project

//! Streaming `RecalibrationStats` accumulator and synchronized reservoir sampling.

use super::model::{RecalibrationMeasurement, RecalibrationStats, RunningStats};
use crate::numeric::{reservoir_index, usize_to_f64, write_reservoir_slot};

#[derive(Clone, Copy)]
enum ReservoirAction {
    Append,
    Replace(usize),
}

fn next_reservoir_action(
    sample_count: &mut usize,
    total_count: usize,
    max_samples: usize,
) -> Option<ReservoirAction> {
    if *sample_count < max_samples {
        *sample_count += 1;
        Some(ReservoirAction::Append)
    } else {
        let idx = reservoir_index(total_count);
        (idx < max_samples).then_some(ReservoirAction::Replace(idx))
    }
}

fn apply_reservoir_slot(vec: &mut Vec<f64>, action: ReservoirAction, value: f64) {
    match action {
        ReservoirAction::Append => vec.push(value),
        ReservoirAction::Replace(idx) => write_reservoir_slot(vec, idx, value),
    }
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
        let stats = if self.count == 0 {
            let mut acc = Self::default();
            for &value in samples {
                acc.push(value);
            }
            acc
        } else {
            self
        };
        if stats.count == 0 {
            return (0.0, 0.0, 0.0, 0.0);
        }
        let n = usize_to_f64(stats.count);
        (
            stats.sum / n,
            (stats.sum_sq / n).sqrt(),
            stats.sum_abs / n,
            stats.max_abs,
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

        if let Some(action) =
            next_reservoir_action(&mut self.sample_count, self.total_count, max_samples)
        {
            apply_reservoir_slot(&mut self.error_ppm_before, action, error_ppm_before);
            apply_reservoir_slot(&mut self.error_ppm_after, action, error_ppm_after);
            apply_reservoir_slot(&mut self.error_da_before, action, error_da_before);
            apply_reservoir_slot(&mut self.error_da_after, action, error_da_after);
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

        if let Some(action) = next_reservoir_action(
            &mut self.sample_count,
            self.total_count,
            measurement.max_samples,
        ) {
            apply_reservoir_slot(&mut self.error_ppm_ms1, action, measurement.error_ppm_ms1);
            apply_reservoir_slot(
                &mut self.error_ppm_before,
                action,
                measurement.error_ppm_before,
            );
            apply_reservoir_slot(
                &mut self.error_ppm_after,
                action,
                measurement.error_ppm_after,
            );
            apply_reservoir_slot(&mut self.error_da_ms1, action, measurement.error_da_ms1);
            apply_reservoir_slot(
                &mut self.error_da_before,
                action,
                measurement.error_da_before,
            );
            apply_reservoir_slot(&mut self.error_da_after, action, measurement.error_da_after);
        }
        if let Some(family) = measurement.adduct_family {
            self.record_adduct_error(family, measurement.error_ppm_before);
        }
    }

    fn record_adduct_error(&mut self, family: &str, err_before: f64) {
        if let Some(bucket) = self.error_by_adduct_before.get_mut(family) {
            bucket.push(err_before);
        } else {
            self.error_by_adduct_before
                .insert(family.to_string(), vec![err_before]);
        }
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
