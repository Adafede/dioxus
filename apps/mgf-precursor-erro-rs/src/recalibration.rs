/// Represents different calibration models for MS2 fragment recalibration.
///
/// This module provides a generic interface for applying scan-wide calibration corrections
/// to fragment peaks based on the discrepancy between the precursor measured in MS1 and
/// the precursor reported in the MS2 scan.
///
/// The precursor discrepancy is treated as an estimate of a latent scan-wide calibration error,
/// and a shrinkage parameter λ (lambda) controls what fraction of the error is applied.
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub enum CalibrationModel {
    /// No recalibration is applied.
    #[default]
    None,

    /// Orbitrap-style recalibration using parts-per-million (ppm).
    ///
    /// The error is computed as:
    /// ```ignore
    /// delta_ppm = 1e6 * (precursor_ms2 - precursor_ms1) / precursor_ms1
    /// ```
    ///
    /// Each fragment is corrected as:
    /// ```ignore
    /// fragment_corrected = fragment * (1 - λ * delta_ppm / 1e6)
    /// ```
    ///
    /// # Parameters
    /// - `lambda`: Shrinkage parameter in [0, 1].
    ///   - 0 = no correction
    ///   - 1 = full precursor transfer
    ///   - intermediate = shrinkage estimator
    OrbitrapPPM { lambda: f64 },

    /// Time-of-flight-style recalibration using absolute mass difference in Da.
    ///
    /// The error is computed as:
    /// ```ignore
    /// delta_da = precursor_ms2 - precursor_ms1
    /// ```
    ///
    /// Each fragment is corrected as:
    /// ```ignore
    /// fragment_corrected = fragment - λ * delta_da
    /// ```
    ///
    /// # Parameters
    /// - `lambda`: Shrinkage parameter in [0, 1].
    ///   - 0 = no correction
    ///   - 1 = full precursor transfer
    ///   - intermediate = shrinkage estimator
    TOFDa { lambda: f64 },
}

impl CalibrationModel {
    /// Returns whether this model applies any correction.
    #[must_use]
    pub const fn is_active(&self) -> bool {
        !matches!(self, Self::None)
    }

    /// Returns the lambda parameter if applicable, otherwise None.
    #[must_use]
    pub const fn lambda(&self) -> Option<f64> {
        match self {
            Self::None => None,
            Self::OrbitrapPPM { lambda } | Self::TOFDa { lambda } => Some(*lambda),
        }
    }
}

/// A simple struct representing a mass spectrometry peak (fragment).
#[derive(Clone, Debug, Copy, PartialEq)]
pub struct Peak {
    /// The m/z value of the peak.
    pub mz: f64,
    /// The intensity of the peak.
    pub intensity: f64,
}

impl Peak {
    /// Creates a new peak with the given m/z and intensity.
    #[must_use]
    pub const fn new(mz: f64, intensity: f64) -> Self {
        Self { mz, intensity }
    }
}

/// Recalibrates fragment m/z values based on the precursor discrepancy.
///
/// This function applies the calibration model to all fragments, modifying their m/z values
/// in-place while preserving intensities and peak ordering.
///
/// # Arguments
///
/// * `fragments` - A slice of peaks to recalibrate. m/z values are modified in place.
/// * `precursor_ms1` - The precursor m/z measured in MS1 (reference).
/// * `precursor_ms2` - The precursor m/z reported in the MS2 scan.
/// * `model` - The calibration model to apply.
///
/// # Behavior
///
/// - If `model` is `None`, the function returns immediately without modification.
/// - If either precursor is missing (NaN or infinite), no correction is applied.
/// - Corrections smaller than floating-point precision (< 1e-14) are skipped.
/// - Fragment intensities are never modified.
/// - Fragment ordering is preserved.
///
/// # Numerical stability
///
/// For Orbitrap (ppm) model:
/// - A ppm error < 1e-7 (parts per million) is considered negligible and skipped.
///
/// For TOF (Da) model:
/// - A Da error < 1e-14 is considered negligible and skipped.
pub fn recalibrate_fragments(
    fragments: &mut [Peak],
    precursor_ms1: f64,
    precursor_ms2: f64,
    model: CalibrationModel,
) {
    // Skip if no model is active
    if !model.is_active() {
        return;
    }

    // Skip if either precursor is missing or invalid
    if !precursor_ms1.is_finite() || !precursor_ms2.is_finite() {
        return;
    }

    // Skip if precursor values are zero (would cause division by zero or nonsensical correction)
    if precursor_ms1.abs() < f64::EPSILON {
        return;
    }

    match model {
        CalibrationModel::None => {
            // Already handled above
        }
        CalibrationModel::OrbitrapPPM { lambda } => {
            apply_orbitrap_correction(fragments, precursor_ms1, precursor_ms2, lambda);
        }
        CalibrationModel::TOFDa { lambda } => {
            apply_tof_correction(fragments, precursor_ms1, precursor_ms2, lambda);
        }
    }
}

/// Applies Orbitrap-style (ppm) correction to fragments.
fn apply_orbitrap_correction(
    fragments: &mut [Peak],
    precursor_ms1: f64,
    precursor_ms2: f64,
    lambda: f64,
) {
    // Compute error in ppm
    let delta_ppm = 1e6 * (precursor_ms2 - precursor_ms1) / precursor_ms1;

    // Skip if error is negligible (< 1e-7 ppm, i.e., < 1e-13 in relative terms)
    if delta_ppm.abs() < 1e-7 {
        return;
    }

    // Clamp lambda to [0, 1] for safety
    let lambda_clamped = lambda.clamp(0.0, 1.0);

    // Compute the fractional correction factor: (1 - λ * delta_ppm / 1e6)
    let correction_factor = 1.0 - lambda_clamped * delta_ppm / 1e6;

    // Apply correction to each fragment
    for peak in fragments {
        if peak.mz.is_finite() && peak.mz > 0.0 {
            peak.mz *= correction_factor;
        }
    }
}

/// Applies TOF-style (absolute Da) correction to fragments.
fn apply_tof_correction(
    fragments: &mut [Peak],
    precursor_ms1: f64,
    precursor_ms2: f64,
    lambda: f64,
) {
    // Compute error in Da
    let delta_da = precursor_ms2 - precursor_ms1;

    // Skip if error is negligible (< 1e-14 Da)
    if delta_da.abs() < 1e-14 {
        return;
    }

    // Clamp lambda to [0, 1] for safety
    let lambda_clamped = lambda.clamp(0.0, 1.0);

    // Compute the correction: λ * delta_da
    let correction = lambda_clamped * delta_da;

    // Apply correction to each fragment
    for peak in fragments {
        if peak.mz.is_finite() {
            peak.mz -= correction;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calibration_model_none() {
        assert!(!CalibrationModel::None.is_active());
        assert_eq!(CalibrationModel::None.lambda(), None);
    }

    #[test]
    fn test_calibration_model_orbitrap() {
        let model = CalibrationModel::OrbitrapPPM { lambda: 0.75 };
        assert!(model.is_active());
        assert_eq!(model.lambda(), Some(0.75));
    }

    #[test]
    fn test_calibration_model_tof() {
        let model = CalibrationModel::TOFDa { lambda: 0.5 };
        assert!(model.is_active());
        assert_eq!(model.lambda(), Some(0.5));
    }

    #[test]
    fn test_no_correction_with_none_model() {
        let mut fragments = vec![Peak::new(100.0, 1000.0), Peak::new(250.0, 500.0)];
        let original = fragments.clone();

        recalibrate_fragments(&mut fragments, 500.0, 500.0120, CalibrationModel::None);

        assert_eq!(fragments, original);
    }

    #[test]
    fn test_no_correction_lambda_zero() {
        let mut fragments = vec![Peak::new(100.0, 1000.0), Peak::new(250.0, 500.0)];
        let original = fragments.clone();

        recalibrate_fragments(
            &mut fragments,
            500.0,
            500.0120,
            CalibrationModel::TOFDa { lambda: 0.0 },
        );

        // With lambda=0, the correction factor should be 1.0 (no change)
        assert_eq!(fragments, original);
    }

    #[test]
    fn test_tof_full_correction() {
        let mut fragments = vec![Peak::new(100.000, 1000.0), Peak::new(250.000, 500.0)];

        let precursor_ms1 = 500.0000;
        let precursor_ms2 = 500.0120;
        let delta = precursor_ms2 - precursor_ms1; // +0.0120 Da

        recalibrate_fragments(
            &mut fragments,
            precursor_ms1,
            precursor_ms2,
            CalibrationModel::TOFDa { lambda: 1.0 },
        );

        // Expected corrections:
        // fragment_corrected = fragment - 1.0 * delta
        let expected_0 = 100.000 - delta;
        let expected_1 = 250.000 - delta;

        assert!((fragments[0].mz - expected_0).abs() < 1e-10);
        assert!((fragments[1].mz - expected_1).abs() < 1e-10);

        // Intensities should be unchanged
        assert!((fragments[0].intensity - 1000.0).abs() < f64::EPSILON);
        assert!((fragments[1].intensity - 500.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_tof_partial_correction() {
        let mut fragments = vec![Peak::new(100.000, 1000.0), Peak::new(250.000, 500.0)];

        let precursor_ms1 = 500.0000;
        let precursor_ms2 = 500.0120;
        let delta = precursor_ms2 - precursor_ms1; // +0.0120 Da

        recalibrate_fragments(
            &mut fragments,
            precursor_ms1,
            precursor_ms2,
            CalibrationModel::TOFDa { lambda: 0.5 },
        );

        // With lambda=0.5, half the correction is applied
        let expected_0 = 0.5f64.mul_add(-delta, 100.000);
        let expected_1 = 0.5f64.mul_add(-delta, 250.000);

        assert!((fragments[0].mz - expected_0).abs() < 1e-10);
        assert!((fragments[1].mz - expected_1).abs() < 1e-10);
    }

    #[test]
    fn test_orbitrap_full_correction() {
        let mut fragments = vec![Peak::new(200.000, 1000.0), Peak::new(400.000, 500.0)];

        let precursor_ms1 = 500.0000;
        let precursor_ms2 = 500.0050;
        let delta_ppm = 1e6 * (precursor_ms2 - precursor_ms1) / precursor_ms1; // ~10 ppm

        recalibrate_fragments(
            &mut fragments,
            precursor_ms1,
            precursor_ms2,
            CalibrationModel::OrbitrapPPM { lambda: 1.0 },
        );

        // fragment_corrected = fragment * (1 - λ * delta_ppm / 1e6)
        let correction_factor = 1.0 - 1.0 * delta_ppm / 1e6;
        let expected_0 = 200.000 * correction_factor;
        let expected_1 = 400.000 * correction_factor;

        assert!((fragments[0].mz - expected_0).abs() < 1e-10);
        assert!((fragments[1].mz - expected_1).abs() < 1e-10);

        // Intensities should be unchanged
        assert!((fragments[0].intensity - 1000.0).abs() < f64::EPSILON);
        assert!((fragments[1].intensity - 500.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_orbitrap_partial_correction() {
        let mut fragments = vec![Peak::new(200.000, 1000.0), Peak::new(400.000, 500.0)];

        let precursor_ms1 = 500.0000;
        let precursor_ms2 = 500.0050;
        let delta_ppm = 1e6 * (precursor_ms2 - precursor_ms1) / precursor_ms1; // ~10 ppm

        recalibrate_fragments(
            &mut fragments,
            precursor_ms1,
            precursor_ms2,
            CalibrationModel::OrbitrapPPM { lambda: 0.5 },
        );

        // With lambda=0.5, half the correction is applied
        let correction_factor = 1.0 - 0.5 * delta_ppm / 1e6;
        let expected_0 = 200.000 * correction_factor;
        let expected_1 = 400.000 * correction_factor;

        assert!((fragments[0].mz - expected_0).abs() < 1e-10);
        assert!((fragments[1].mz - expected_1).abs() < 1e-10);
    }

    #[test]
    fn test_missing_precursor_ms1() {
        let mut fragments = vec![Peak::new(100.0, 1000.0)];
        let original = fragments.clone();

        recalibrate_fragments(
            &mut fragments,
            f64::NAN,
            500.0,
            CalibrationModel::TOFDa { lambda: 1.0 },
        );

        assert_eq!(fragments, original);
    }

    #[test]
    fn test_missing_precursor_ms2() {
        let mut fragments = vec![Peak::new(100.0, 1000.0)];
        let original = fragments.clone();

        recalibrate_fragments(
            &mut fragments,
            500.0,
            f64::NAN,
            CalibrationModel::TOFDa { lambda: 1.0 },
        );

        assert_eq!(fragments, original);
    }

    #[test]
    fn test_infinite_precursor() {
        let mut fragments = vec![Peak::new(100.0, 1000.0)];
        let original = fragments.clone();

        recalibrate_fragments(
            &mut fragments,
            500.0,
            f64::INFINITY,
            CalibrationModel::TOFDa { lambda: 1.0 },
        );

        assert_eq!(fragments, original);
    }

    #[test]
    fn test_zero_precursor_ms1() {
        let mut fragments = vec![Peak::new(100.0, 1000.0)];
        let original = fragments.clone();

        recalibrate_fragments(
            &mut fragments,
            0.0,
            500.0,
            CalibrationModel::TOFDa { lambda: 1.0 },
        );

        assert_eq!(fragments, original);
    }

    #[test]
    fn test_negligible_error_tof() {
        let mut fragments = vec![Peak::new(100.0, 1000.0)];
        let original = fragments.clone();

        // Error smaller than 1e-14 should be ignored
        recalibrate_fragments(
            &mut fragments,
            500.0,
            500.0 + 1e-15,
            CalibrationModel::TOFDa { lambda: 1.0 },
        );

        assert_eq!(fragments, original);
    }

    #[test]
    fn test_negligible_error_orbitrap() {
        let mut fragments = vec![Peak::new(100.0, 1000.0)];
        let original = fragments.clone();

        // Error smaller than 1e-7 ppm should be ignored
        let precursor_ms1 = 500.0;
        // To get error < 1e-7 ppm: 1e6 * delta / precursor_ms1 < 1e-7
        // delta < 1e-7 * precursor_ms1 / 1e6 = 1e-7 * 500 / 1e6 = 5e-14
        let precursor_ms2 = precursor_ms1 + 1e-14;

        recalibrate_fragments(
            &mut fragments,
            precursor_ms1,
            precursor_ms2,
            CalibrationModel::OrbitrapPPM { lambda: 1.0 },
        );

        assert_eq!(fragments, original);
    }

    #[test]
    fn test_lambda_clamping() {
        let mut fragments = vec![Peak::new(100.0, 1000.0)];
        let original = fragments.clone();

        // Lambda > 1.0 should be clamped to 1.0
        recalibrate_fragments(
            &mut fragments,
            500.0,
            500.01,
            CalibrationModel::TOFDa { lambda: 2.0 },
        );

        let mut fragments_ref = original;
        recalibrate_fragments(
            &mut fragments_ref,
            500.0,
            500.01,
            CalibrationModel::TOFDa { lambda: 1.0 },
        );

        assert_eq!(fragments, fragments_ref);
    }

    #[test]
    fn test_negative_error() {
        let mut fragments = vec![Peak::new(100.0, 1000.0)];

        let precursor_ms1 = 500.0;
        let precursor_ms2 = 499.99; // Error is negative

        recalibrate_fragments(
            &mut fragments,
            precursor_ms1,
            precursor_ms2,
            CalibrationModel::TOFDa { lambda: 1.0 },
        );

        // Should correct in the positive direction
        let delta = precursor_ms2 - precursor_ms1; // negative
        let expected = 100.0 - delta; // 100.0 - (-0.01) = 100.01
        assert!((fragments[0].mz - expected).abs() < 1e-10);
    }

    #[test]
    fn test_peak_ordering_preserved() {
        let mut fragments = vec![
            Peak::new(100.0, 100.0),
            Peak::new(200.0, 200.0),
            Peak::new(150.0, 150.0),
            Peak::new(300.0, 300.0),
        ];

        let precursor_ms1 = 500.0;
        let precursor_ms2 = 500.01;

        recalibrate_fragments(
            &mut fragments,
            precursor_ms1,
            precursor_ms2,
            CalibrationModel::TOFDa { lambda: 1.0 },
        );

        // Check that relative ordering is preserved
        assert!(fragments[0].mz < fragments[2].mz);
        assert!(fragments[2].mz < fragments[1].mz);
        assert!(fragments[1].mz < fragments[3].mz);

        // Check that intensities are in their original positions
        assert!((fragments[0].intensity - 100.0).abs() < f64::EPSILON);
        assert!((fragments[1].intensity - 200.0).abs() < f64::EPSILON);
        assert!((fragments[2].intensity - 150.0).abs() < f64::EPSILON);
        assert!((fragments[3].intensity - 300.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_empty_fragments() {
        let mut fragments = vec![];

        // Should not panic or error
        recalibrate_fragments(
            &mut fragments,
            500.0,
            500.01,
            CalibrationModel::TOFDa { lambda: 1.0 },
        );

        assert!(fragments.is_empty());
    }

    #[test]
    fn test_invalid_fragment_mz() {
        let mut fragments = vec![
            Peak::new(100.0, 1000.0),
            Peak::new(f64::NAN, 500.0),
            Peak::new(200.0, 600.0),
        ];

        recalibrate_fragments(
            &mut fragments,
            500.0,
            500.01,
            CalibrationModel::TOFDa { lambda: 1.0 },
        );

        // Valid fragments should be corrected
        assert!((fragments[0].mz - 100.0).abs() > f64::EPSILON); // Changed
        assert!((fragments[2].mz - 200.0).abs() > f64::EPSILON); // Changed

        // Invalid fragment should remain NaN
        assert!(fragments[1].mz.is_nan());
    }

    #[test]
    fn test_example_from_spec_tof() {
        let mut fragments = vec![Peak::new(100.000, 1.0), Peak::new(250.000, 1.0)];

        // Example from spec:
        // precursor_ms1 = 500.0000
        // precursor_ms2 = 500.0120
        // delta = +0.0120 Da
        //
        // fragment 100.000 → 99.988
        // fragment 250.000 → 249.988

        recalibrate_fragments(
            &mut fragments,
            500.0000,
            500.0120,
            CalibrationModel::TOFDa { lambda: 1.0 },
        );

        assert!((fragments[0].mz - 99.988).abs() < 1e-6);
        assert!((fragments[1].mz - 249.988).abs() < 1e-6);
    }

    #[test]
    fn test_example_from_spec_orbitrap() {
        let mut fragments = vec![Peak::new(200.000, 1.0)];

        // Example from spec:
        // precursor_ms1 = 500.0000
        // precursor_ms2 = 500.0050
        // delta = 10 ppm
        //
        // fragment 200.000 → 199.998

        recalibrate_fragments(
            &mut fragments,
            500.0000,
            500.0050,
            CalibrationModel::OrbitrapPPM { lambda: 1.0 },
        );

        assert!((fragments[0].mz - 199.998).abs() < 1e-6);
    }
}
