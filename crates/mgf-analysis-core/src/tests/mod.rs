// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the mgf-precursor-erro-rs project

mod adduct;
mod calibration;
mod diagnostics;
mod generator;
mod ingestor;
mod mass;
mod metrics;

pub(crate) fn assert_close(actual: f64, expected: f64, tol: f64, label: &str) {
    assert!(
        (actual - expected).abs() < tol,
        "{label}: expected {expected:.7}, got {actual:.7}"
    );
}
