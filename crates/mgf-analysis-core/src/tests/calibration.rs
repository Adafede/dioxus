// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the mgf-precursor-erro-rs project

use super::assert_close;
use crate::{
    CalibrationModel, MgfAnalysisError, Peak, ShrinkageLambda, recalibrate_fragment_mz,
    recalibrate_fragments,
};

#[test]
fn calibration_model_and_shrinkage_lambda_validation() {
    assert!(!CalibrationModel::None.is_active());
    assert_eq!(CalibrationModel::None.lambda(), None);

    let orb = CalibrationModel::orbitrap(0.75);
    assert!(orb.is_active());
    assert_eq!(orb.lambda(), Some(0.75));

    let tof = CalibrationModel::tof(0.5);
    assert!(tof.is_active());
    assert_eq!(tof.lambda(), Some(0.5));
    assert_eq!(tof.with_lambda(ShrinkageLambda::FULL).lambda(), Some(1.0));

    assert!(matches!(
        ShrinkageLambda::new(f64::NAN),
        Err(MgfAnalysisError::InvalidShrinkageLambda(_))
    ));
    assert!(matches!(
        ShrinkageLambda::new(1.5),
        Err(MgfAnalysisError::InvalidShrinkageLambda(_))
    ));
    assert_eq!(ShrinkageLambda::clamped(2.0).get(), 1.0);
    assert_eq!(ShrinkageLambda::clamped(f64::NAN).get(), 0.0);
}

#[test]
fn tof_and_orbitrap_fragment_corrections() {
    let mut tof_frags = vec![Peak::new(100.0, 1000.0), Peak::new(250.0, 500.0)];
    recalibrate_fragments(&mut tof_frags, 500.0, 500.012, CalibrationModel::tof(0.5));
    assert_close(tof_frags[0].mz, 99.994, 1e-10, "tof 0");
    assert_close(tof_frags[1].mz, 249.994, 1e-10, "tof 1");
    assert_close(tof_frags[0].intensity, 1000.0, 1e-12, "intensity kept");

    let mut orb_frags = vec![Peak::new(200.0, 1000.0), Peak::new(400.0, 500.0)];
    recalibrate_fragments(
        &mut orb_frags,
        500.0,
        500.005,
        CalibrationModel::orbitrap(1.0),
    );
    assert_close(orb_frags[0].mz, 199.998, 1e-6, "orbitrap 0");
    assert_close(orb_frags[1].mz, 399.996, 1e-6, "orbitrap 1");
}

#[test]
fn edge_cases_non_finite_negligible_error_and_clamping() {
    let original = vec![Peak::new(100.0, 1000.0)];
    for (ms1, ms2, model) in [
        (500.0, 500.012, CalibrationModel::None),
        (500.0, 500.012, CalibrationModel::tof(0.0)),
        (f64::NAN, 500.0, CalibrationModel::tof(1.0)),
        (500.0, f64::NAN, CalibrationModel::tof(1.0)),
        (500.0, f64::INFINITY, CalibrationModel::tof(1.0)),
        (0.0, 500.0, CalibrationModel::tof(1.0)),
        (500.0, 500.0 + 1e-15, CalibrationModel::tof(1.0)),
        (500.0, 500.0 + 1e-14, CalibrationModel::orbitrap(1.0)),
    ] {
        let mut copy = original.clone();
        recalibrate_fragments(&mut copy, ms1, ms2, model);
        assert_eq!(copy, original);
    }

    let mut clamped = original.clone();
    recalibrate_fragments(&mut clamped, 500.0, 500.01, CalibrationModel::tof(2.0));
    let mut full = original;
    recalibrate_fragments(&mut full, 500.0, 500.01, CalibrationModel::tof(1.0));
    assert_eq!(clamped, full);

    assert_close(
        recalibrate_fragment_mz(250.5, f64::NAN, 500.0, CalibrationModel::tof(1.0)),
        250.5,
        1e-12,
        "nan delta",
    );
    assert_close(
        recalibrate_fragment_mz(250.5, 0.01, 500.0, CalibrationModel::orbitrap(f64::NAN)),
        250.5,
        1e-12,
        "nan lambda",
    );
}
