// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! MS2 fragment recalibration over MGF content.
//!
//! Split by responsibility (previously a single 833-line file):
//! - [`types`] — `CalibrationModel` / `Peak` definitions.
//! - [`parsing`] — MGF line parsing (pepmass directives + fragment lines).
//! - [`calibration`] — per-fragment calibration math.
//! - [`generator`] — MGF round-tripping + bulk recalibration orchestration.

mod calibration;
mod generator;
mod parsing;
mod types;

pub use calibration::recalibrate_fragment_mz;
pub use generator::{
    generate_recalibrated_mgf, recalibrate_fragments, write_fragments_as_is,
    write_recalibrated_fragments,
};
pub use parsing::{extract_pepmass_from_line, find_ms2_precursor_peak, is_fragment_line};
pub use types::{CalibrationModel, Peak};

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

    #[test]
    fn test_generate_recalibrated_mgf_with_tof() {
        let input = r"BEGIN IONS
TITLE=test
PEPMASS=500.0000
CHARGE=1
100.0 50
200.0 100
500.01 200
250.0 150
END IONS";

        let model = CalibrationModel::TOFDa { lambda: 1.0 };
        let output = generate_recalibrated_mgf(input, model, None);

        assert_ne!(input, output);
        assert!(output.contains("99.99") || output.contains("199.99"));
    }

    #[test]
    fn test_generate_recalibrated_mgf_no_model() {
        let input = r"BEGIN IONS
TITLE=test
PEPMASS=500.0000
CHARGE=1
100.0 50
END IONS";

        let model = CalibrationModel::None;
        let output = generate_recalibrated_mgf(input, model, None);

        assert_eq!(input, output);
    }
}
