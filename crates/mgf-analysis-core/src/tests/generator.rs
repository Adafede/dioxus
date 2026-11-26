// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the mgf-precursor-erro-rs project

use crate::{CalibrationModel, generate_recalibrated_mgf};

#[test]
fn generate_recalibrated_mgf_no_model_returns_verbatim() {
    let input = "BEGIN IONS\nTITLE=test\nPEPMASS=500.0000\n100.0 50\nEND IONS";
    assert_eq!(
        generate_recalibrated_mgf(input, CalibrationModel::None),
        input
    );
}

#[test]
fn generate_recalibrated_mgf_with_tof_and_orbitrap() {
    let tof_in = "BEGIN IONS\nTITLE=test\nPEPMASS=500.0000\nCHARGE=1\n100.0 50\n200.0 100\n500.01 200\nEND IONS";
    let tof_out = generate_recalibrated_mgf(tof_in, CalibrationModel::tof(1.0));
    assert_ne!(tof_in, tof_out);
    assert!(tof_out.contains("99.99") && tof_out.contains("199.99"));

    let orb_in = "\
BEGIN IONS
TITLE=orb
PEPMASS=500.0000 999
500.0050 200.0 extra_col
200.0000 50.5 keep_tag
END IONS
BEGIN IONS
TITLE=no_ms2_precursor
PEPMASS=500.0000
150.0000 25.0
END IONS";
    let orb_out = generate_recalibrated_mgf(orb_in, CalibrationModel::orbitrap(1.0));
    assert!(orb_out.contains("TITLE=orb\nPEPMASS=500.0000 999\n"));
    assert!(
        orb_out
            .lines()
            .any(|line| line.starts_with("199.998") && line.ends_with(" 50.5 keep_tag")),
        "unexpected output: {orb_out:?}"
    );
    assert!(orb_out.contains("TITLE=no_ms2_precursor\nPEPMASS=500.0000\n150.0000 25.0\nEND IONS"));
}

#[test]
fn generate_recalibrated_mgf_preserves_block_boundaries_global_params_and_separators() {
    let multi = "BEGIN IONS\nTITLE=a\nPEPMASS=500.0\n100.0 5\nEND IONS\n\
                 BEGIN IONS\nTITLE=b\nPEPMASS=600.0\n100.0 5\nEND IONS\n\
                 BEGIN IONS\nTITLE=c\nPEPMASS=700.0\n100.0 5\nEND IONS";
    let out = generate_recalibrated_mgf(multi, CalibrationModel::tof(1.0));
    assert_eq!(out.matches("BEGIN IONS").count(), 3);
    assert_eq!(out.matches("END IONS").count(), 3);
    for title in ["a", "b", "c"] {
        assert!(out.contains(&format!("TITLE={title}")));
    }
    assert!(!out.contains("\n\n"));

    let with_sep = "COM=test\nBEGIN IONS\nTITLE=a\nPEPMASS=500.0\n100.0 5\nEND IONS\n\n\
                    BEGIN IONS\nTITLE=b\nPEPMASS=600.0\n100.0 5\nEND IONS";
    let sep_out = generate_recalibrated_mgf(with_sep, CalibrationModel::tof(1.0));
    assert!(sep_out.starts_with("COM=test\n"));
    assert!(sep_out.contains("END IONS\n\nBEGIN IONS"));

    let unterminated = "BEGIN IONS\nTITLE=open\nPEPMASS=500.0\n500.01 10\n100.0 5";
    let unterm_out = generate_recalibrated_mgf(unterminated, CalibrationModel::tof(1.0));
    assert!(
        !unterm_out.contains("END IONS"),
        "unterminated EOF block must not get synthetic END IONS: {unterm_out:?}"
    );
}
