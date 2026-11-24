// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the mgf-precursor-erro-rs project

//! Summary and measurement structs for precursor recalibration diagnostics.

use std::collections::BTreeMap;

/// Running moment accumulator for stream-wide mean/RMS/MAE/max-abs statistics.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(super) struct RunningStats {
    pub(super) count: usize,
    pub(super) sum: f64,
    pub(super) sum_abs: f64,
    pub(super) sum_sq: f64,
    pub(super) max_abs: f64,
}

/// Complete recalibration measurement folded into [`RecalibrationStats`].
#[derive(Clone, Copy, Debug)]
pub(crate) struct RecalibrationMeasurement<'a> {
    /// MS1 error in ppm.
    pub(crate) error_ppm_ms1: f64,
    /// Calibration delta (`MS2 - MS1`) in ppm.
    pub(crate) delta_ppm_ms2_ms1: f64,
    /// MS2 error before correction (ppm).
    pub(crate) error_ppm_before: f64,
    /// MS2 error after correction (ppm).
    pub(crate) error_ppm_after: f64,
    /// MS1 error in Da.
    pub(crate) error_da_ms1: f64,
    /// Calibration delta (`MS2 - MS1`) in Da.
    pub(crate) delta_da_ms2_ms1: f64,
    /// MS2 error before correction (Da).
    pub(crate) error_da_before: f64,
    /// MS2 error after correction (Da).
    pub(crate) error_da_after: f64,
    /// MS1 precursor `m/z`.
    pub(crate) precursor_ms1: f64,
    /// MS2 precursor `m/z` before correction.
    pub(crate) precursor_ms2_before: f64,
    /// MS2 precursor `m/z` after correction.
    pub(crate) precursor_ms2_after: f64,
    /// Optional adduct family label.
    pub(crate) adduct_family: Option<&'a str>,
    /// Reservoir sample cap per series.
    pub(crate) max_samples: usize,
}

/// Summary and sampled distribution statistics before and after recalibration.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct RecalibrationStats {
    /// Sampled MS1 precursor errors (ppm).
    pub error_ppm_ms1: Vec<f64>,
    /// Sampled `MS2 - MS1` deltas (ppm).
    pub delta_ppm_ms2_ms1: Vec<f64>,
    /// Sampled pre-recalibration errors (ppm).
    pub error_ppm_before: Vec<f64>,
    /// Sampled post-recalibration errors (ppm).
    pub error_ppm_after: Vec<f64>,
    /// Sampled MS1 precursor errors (Da).
    pub error_da_ms1: Vec<f64>,
    /// Sampled `MS2 - MS1` deltas (Da).
    pub delta_da_ms2_ms1: Vec<f64>,
    /// Sampled pre-recalibration errors (Da).
    pub error_da_before: Vec<f64>,
    /// Sampled post-recalibration errors (Da).
    pub error_da_after: Vec<f64>,
    /// Sampled MS1 precursor `m/z` values.
    pub precursor_ms1_values: Vec<f64>,
    /// Sampled MS2 precursor `m/z` values before correction.
    pub precursor_ms2_before: Vec<f64>,
    /// Sampled MS2 precursor `m/z` values after correction.
    pub precursor_ms2_after: Vec<f64>,
    /// Number of retained diagnostic samples.
    pub sample_count: usize,
    /// Total precursors processed before sampling.
    pub total_count: usize,
    /// Mean MS1 error (ppm).
    pub mean_error_ppm_ms1: f64,
    /// Mean calibration delta (ppm).
    pub mean_delta_ppm_ms2_ms1: f64,
    /// Mean error before recalibration (ppm).
    pub mean_error_ppm_before: f64,
    /// Mean error after recalibration (ppm).
    pub mean_error_ppm_after: f64,
    /// Mean MS1 error (Da).
    pub mean_error_da_ms1: f64,
    /// Mean calibration delta (Da).
    pub mean_delta_da_ms2_ms1: f64,
    /// Mean error before recalibration (Da).
    pub mean_error_da_before: f64,
    /// Mean error after recalibration (Da).
    pub mean_error_da_after: f64,
    /// Mean absolute error before recalibration (ppm).
    pub mae_error_ppm_before: f64,
    /// Mean absolute error after recalibration (ppm).
    pub mae_error_ppm_after: f64,
    /// Mean absolute error before recalibration (Da).
    pub mae_error_da_before: f64,
    /// Mean absolute error after recalibration (Da).
    pub mae_error_da_after: f64,
    /// RMS MS1 error (ppm).
    pub rms_error_ppm_ms1: f64,
    /// RMS calibration delta (ppm).
    pub rms_delta_ppm_ms2_ms1: f64,
    /// RMS error before recalibration (ppm).
    pub rms_error_ppm_before: f64,
    /// RMS error after recalibration (ppm).
    pub rms_error_ppm_after: f64,
    /// RMS MS1 error (Da).
    pub rms_error_da_ms1: f64,
    /// RMS calibration delta (Da).
    pub rms_delta_da_ms2_ms1: f64,
    /// RMS error before recalibration (Da).
    pub rms_error_da_before: f64,
    /// RMS error after recalibration (Da).
    pub rms_error_da_after: f64,
    /// Maximum absolute error before recalibration (ppm).
    pub max_abs_error_ppm_before: f64,
    /// Maximum absolute error after recalibration (ppm).
    pub max_abs_error_ppm_after: f64,
    /// Per-adduct errors before recalibration (ppm).
    pub error_by_adduct_before: BTreeMap<String, Vec<f64>>,
    /// Per-adduct errors after recalibration (ppm).
    pub error_by_adduct_after: BTreeMap<String, Vec<f64>>,
    pub(super) stream_ppm_ms1: RunningStats,
    pub(super) stream_delta_ppm: RunningStats,
    pub(super) stream_ppm_before: RunningStats,
    pub(super) stream_ppm_after: RunningStats,
    pub(super) stream_da_ms1: RunningStats,
    pub(super) stream_delta_da: RunningStats,
    pub(super) stream_da_before: RunningStats,
    pub(super) stream_da_after: RunningStats,
}
