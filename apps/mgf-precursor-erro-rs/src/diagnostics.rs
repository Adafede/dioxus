/// Diagnostics and plotting utilities for precursor error analysis before and after recalibration.
///
/// This module provides tools to compute and visualize precursor error distributions,
/// allowing users to assess the impact of recalibration on mass accuracy.
use std::collections::BTreeMap;

/// Statistics about precursor errors for diagnostic purposes.
#[derive(Clone, Debug)]
pub struct RecalibrationDiagnostics {
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

    /// RMS (root mean square) error in ppm (Stage 1: PEPMASS vs theory).
    pub rms_error_ppm_ms1: f64,

    /// RMS calibration delta (Stage 3: MS2 - MS1) in ppm.
    pub rms_delta_ppm_ms2_ms1: f64,

    /// RMS (root mean square) error in ppm before recalibration.
    pub rms_error_ppm_before: f64,

    /// RMS (root mean square) error in ppm after recalibration.
    pub rms_error_ppm_after: f64,

    /// RMS (root mean square) error in Da (Stage 1: PEPMASS vs theory).
    pub rms_error_da_ms1: f64,

    /// RMS calibration delta (Stage 3: MS2 - MS1) in Da.
    pub rms_delta_da_ms2_ms1: f64,

    /// RMS (root mean square) error in Da before recalibration.
    pub rms_error_da_before: f64,

    /// RMS (root mean square) error in Da after recalibration.
    pub rms_error_da_after: f64,

    /// Maximum absolute error in ppm before recalibration.
    pub max_abs_error_ppm_before: f64,

    /// Maximum absolute error in ppm after recalibration.
    pub max_abs_error_ppm_after: f64,

    /// Adduct family categorization for errors before recalibration.
    pub error_by_adduct_before: BTreeMap<String, Vec<f64>>,

    /// Adduct family categorization for errors after recalibration.
    pub error_by_adduct_after: BTreeMap<String, Vec<f64>>,

    /// All fragment m/z errors (in Da) before recalibration, sampled for ECDF.
    pub fragment_error_da_before: Vec<f64>,

    /// All fragment m/z errors (in Da) after recalibration, sampled for ECDF.
    pub fragment_error_da_after: Vec<f64>,

    /// All fragment m/z errors (in ppm) before recalibration, sampled for ECDF.
    pub fragment_error_ppm_before: Vec<f64>,

    /// All fragment m/z errors (in ppm) after recalibration, sampled for ECDF.
    pub fragment_error_ppm_after: Vec<f64>,
}

impl Default for RecalibrationDiagnostics {
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
            fragment_error_da_before: Vec::new(),
            fragment_error_da_after: Vec::new(),
            fragment_error_ppm_before: Vec::new(),
            fragment_error_ppm_after: Vec::new(),
        }
    }
}

impl RecalibrationDiagnostics {
    /// Creates a new empty diagnostics structure.
    pub fn new() -> Self {
        Self::default()
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

        // Sample for plotting
        if self.sample_count < max_samples {
            self.error_ppm_before.push(error_ppm_before);
            self.error_ppm_after.push(error_ppm_after);
            self.error_da_before.push(error_da_before);
            self.error_da_after.push(error_da_after);
            self.sample_count += 1;
        } else {
            // Reservoir sampling for additional points
            let index = (self.total_count as f64).sqrt() as usize % max_samples;
            if index < max_samples {
                self.error_ppm_before[index] = error_ppm_before;
                self.error_ppm_after[index] = error_ppm_after;
                self.error_da_before[index] = error_da_before;
                self.error_da_after[index] = error_da_after;
            }
        }

        // Track by adduct family if provided
        if let Some(family) = adduct_family {
            self.error_by_adduct_before
                .entry(family.to_string())
                .or_insert_with(Vec::new)
                .push(error_ppm_before);
            self.error_by_adduct_after
                .entry(family.to_string())
                .or_insert_with(Vec::new)
                .push(error_ppm_after);
        }
    }

    /// Adds a complete measurement including precursor m/z values.
    pub fn push_measurement(
        &mut self,
        error_ppm_ms1: f64,
        delta_ppm_ms2_ms1: f64,
        error_ppm_before: f64,
        error_ppm_after: f64,
        error_da_ms1: f64,
        delta_da_ms2_ms1: f64,
        error_da_before: f64,
        error_da_after: f64,
        precursor_ms1: f64,
        precursor_ms2_before: f64,
        precursor_ms2_after: f64,
        adduct_family: Option<&str>,
        max_samples: usize,
    ) {
        self.total_count += 1;

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
            let index = (self.total_count as f64).sqrt() as usize % max_samples;
            if index < max_samples {
                self.error_ppm_ms1[index] = error_ppm_ms1;
                self.delta_ppm_ms2_ms1[index] = delta_ppm_ms2_ms1;
                self.error_ppm_before[index] = error_ppm_before;
                self.error_ppm_after[index] = error_ppm_after;
                self.error_da_ms1[index] = error_da_ms1;
                self.delta_da_ms2_ms1[index] = delta_da_ms2_ms1;
            }
        }

        // Track by adduct family if provided
        if let Some(family) = adduct_family {
            self.error_by_adduct_before
                .entry(family.to_string())
                .or_insert_with(Vec::new)
                .push(error_ppm_before);
            self.error_by_adduct_after
                .entry(family.to_string())
                .or_insert_with(Vec::new)
                .push(error_ppm_after);
        }
    }

    /// Add fragment-level errors for ECDF plots.
    pub fn push_fragment_errors(
        &mut self,
        fragment_error_ppm_before: f64,
        fragment_error_ppm_after: f64,
        fragment_error_da_before: f64,
        fragment_error_da_after: f64,
        max_samples: usize,
    ) {
        // Sample for plotting
        if self.fragment_error_ppm_before.len() < max_samples {
            self.fragment_error_ppm_before
                .push(fragment_error_ppm_before);
            self.fragment_error_ppm_after.push(fragment_error_ppm_after);
            self.fragment_error_da_before.push(fragment_error_da_before);
            self.fragment_error_da_after.push(fragment_error_da_after);
        }
    }

    /// Computes summary statistics from the collected errors.
    pub fn compute_statistics(&mut self) {
        self.mean_error_ppm_ms1 = compute_mean(&self.error_ppm_ms1);
        self.mean_delta_ppm_ms2_ms1 = compute_mean(&self.delta_ppm_ms2_ms1);
        self.mean_error_ppm_before = compute_mean(&self.error_ppm_before);
        self.mean_error_ppm_after = compute_mean(&self.error_ppm_after);
        self.mean_error_da_ms1 = compute_mean(&self.error_da_ms1);
        self.mean_delta_da_ms2_ms1 = compute_mean(&self.delta_da_ms2_ms1);
        self.mean_error_da_before = compute_mean(&self.error_da_before);
        self.mean_error_da_after = compute_mean(&self.error_da_after);

        self.rms_error_ppm_ms1 = compute_rms(&self.error_ppm_ms1);
        self.rms_delta_ppm_ms2_ms1 = compute_rms(&self.delta_ppm_ms2_ms1);
        self.rms_error_ppm_before = compute_rms(&self.error_ppm_before);
        self.rms_error_ppm_after = compute_rms(&self.error_ppm_after);
        self.rms_error_da_ms1 = compute_rms(&self.error_da_ms1);
        self.rms_delta_da_ms2_ms1 = compute_rms(&self.delta_da_ms2_ms1);
        self.rms_error_da_before = compute_rms(&self.error_da_before);
        self.rms_error_da_after = compute_rms(&self.error_da_after);

        self.max_abs_error_ppm_before = compute_max_abs(&self.error_ppm_before);
        self.max_abs_error_ppm_after = compute_max_abs(&self.error_ppm_after);
    }

    /// Returns the improvement in mean absolute error after recalibration (in ppm).
    pub fn mean_error_improvement_ppm(&self) -> f64 {
        self.mean_error_ppm_before.abs() - self.mean_error_ppm_after.abs()
    }

    /// Returns the improvement in RMS error after recalibration (in ppm).
    pub fn rms_error_improvement_ppm(&self) -> f64 {
        self.rms_error_ppm_before - self.rms_error_ppm_after
    }

    /// Returns the improvement in mean absolute error after recalibration (in Da).
    pub fn mean_error_improvement_da(&self) -> f64 {
        self.mean_error_da_before.abs() - self.mean_error_da_after.abs()
    }

    /// Returns the improvement in RMS error after recalibration (in Da).
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
    sum / values.len() as f64
}

/// Computes the root mean square (RMS) of absolute values.
fn compute_rms(values: &[f64]) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    let sum_sq: f64 = values.iter().map(|v| v * v).sum();
    (sum_sq / values.len() as f64).sqrt()
}

/// Computes the maximum absolute value.
fn compute_max_abs(values: &[f64]) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    values.iter().map(|v| v.abs()).fold(0.0, f64::max)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_diagnostics() {
        let diag = RecalibrationDiagnostics::new();
        assert_eq!(diag.sample_count, 0);
        assert_eq!(diag.total_count, 0);
        assert_eq!(diag.mean_error_ppm_before, 0.0);
    }

    #[test]
    fn test_push_error() {
        let mut diag = RecalibrationDiagnostics::new();
        diag.push_error(10.0, 5.0, 0.05, 0.025, Some("protonated"), 100);

        assert_eq!(diag.total_count, 1);
        assert_eq!(diag.sample_count, 1);
        assert_eq!(diag.error_ppm_before[0], 10.0);
        assert_eq!(diag.error_ppm_after[0], 5.0);
    }

    #[test]
    fn test_compute_statistics() {
        let mut diag = RecalibrationDiagnostics::new();
        diag.push_error(10.0, 5.0, 0.05, 0.025, None, 100);
        diag.push_error(20.0, 10.0, 0.1, 0.05, None, 100);
        diag.compute_statistics();

        assert!((diag.mean_error_ppm_before - 15.0).abs() < 1e-6);
        assert!((diag.mean_error_ppm_after - 7.5).abs() < 1e-6);
    }

    #[test]
    fn test_improvement_calculation() {
        let mut diag = RecalibrationDiagnostics::new();
        diag.push_error(10.0, 5.0, 0.1, 0.05, None, 100);
        diag.push_error(10.0, 5.0, 0.1, 0.05, None, 100);
        diag.compute_statistics();

        let improvement = diag.mean_error_improvement_ppm();
        assert!(improvement > 0.0);
    }

    #[test]
    fn test_adduct_categorization() {
        let mut diag = RecalibrationDiagnostics::new();
        diag.push_error(10.0, 5.0, 0.05, 0.025, Some("protonated"), 100);
        diag.push_error(15.0, 8.0, 0.075, 0.04, Some("deprotonated"), 100);

        assert_eq!(diag.error_by_adduct_before.len(), 2);
        assert_eq!(diag.error_by_adduct_before["protonated"][0], 10.0);
        assert_eq!(diag.error_by_adduct_before["deprotonated"][0], 15.0);
    }

    #[test]
    fn test_max_samples_limit() {
        let mut diag = RecalibrationDiagnostics::new();
        for i in 0..150 {
            let error = (i as f64) * 0.1;
            diag.push_error(error, error / 2.0, error / 100.0, error / 200.0, None, 100);
        }

        assert_eq!(diag.sample_count, 100);
        assert_eq!(diag.total_count, 150);
    }

    #[test]
    fn test_compute_mean() {
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        assert!((compute_mean(&values) - 3.0).abs() < 1e-10);
    }

    #[test]
    fn test_compute_rms() {
        let values = vec![1.0, 2.0, 3.0];
        let expected = ((1.0 + 4.0 + 9.0) / 3.0_f64).sqrt();
        assert!((compute_rms(&values) - expected).abs() < 1e-10);
    }

    #[test]
    fn test_compute_max_abs() {
        let values = vec![1.0, -5.0, 3.0, -2.0];
        assert!((compute_max_abs(&values) - 5.0).abs() < 1e-10);
    }
}
