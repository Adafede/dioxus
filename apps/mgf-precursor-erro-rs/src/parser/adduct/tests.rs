// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the mgf-precursor-erro-rs project

use super::*;

fn assert_mz_close(actual: f64, expected: f64, label: &str) {
    assert!(
        (actual - expected).abs() < 1e-5,
        "{label}: expected {expected:.6}, got {actual:.6}"
    );
}

#[test]
fn theoretical_precursor_mz_common_and_divalent_adducts() {
    let neutral = 500.0;
    assert_mz_close(
        expected_precursor_mz(neutral, Some("[M+H]+"), None, None),
        neutral + HYDROGEN_MASS - ELECTRON_MASS,
        "[M+H]+",
    );
    assert_mz_close(
        expected_precursor_mz(neutral, Some("[M-H]-"), None, None),
        neutral - HYDROGEN_MASS + ELECTRON_MASS,
        "[M-H]-",
    );
    assert_mz_close(
        expected_precursor_mz(neutral, Some("[M+Na]+"), None, None),
        neutral + SODIUM_MASS - ELECTRON_MASS,
        "[M+Na]+",
    );
    assert_mz_close(
        expected_precursor_mz(neutral, Some("[M+NH4]+"), None, None),
        neutral + AMMONIUM_MASS - ELECTRON_MASS,
        "[M+NH4]+",
    );
    assert_mz_close(
        expected_precursor_mz(neutral, Some("[M+Mg]2+"), None, None),
        (neutral + 23.985_041_7 - 2.0 * ELECTRON_MASS) / 2.0,
        "[M+Mg]2+",
    );
    assert_mz_close(
        expected_precursor_mz(neutral, Some("[M+Ca]2+"), None, None),
        (neutral + 39.962_590_98 - 2.0 * ELECTRON_MASS) / 2.0,
        "[M+Ca]2+",
    );
    assert_mz_close(
        expected_precursor_mz(neutral, Some("[2M+H]+"), None, None),
        2.0 * neutral + HYDROGEN_MASS - ELECTRON_MASS,
        "[2M+H]+",
    );
}

#[test]
fn formate_acetate_and_acetonitrile_adducts_agree_across_notations() {
    let neutral = 400.0;
    let formate_mz = expected_precursor_mz(neutral, Some("[M+HCOO]-"), None, None);
    assert_mz_close(
        formate_mz,
        expected_precursor_mz(neutral, Some("[M+FA-H]-"), None, None),
        "[M+FA-H]- vs [M+HCOO]-",
    );
    assert_mz_close(
        formate_mz,
        expected_precursor_mz(neutral, Some("[M+FA]-"), None, None),
        "[M+FA]- vs [M+HCOO]-",
    );
    assert_mz_close(
        formate_mz,
        neutral + 44.997_654_29 + ELECTRON_MASS,
        "[M+FA-H]- theoretical",
    );

    let acetate_mz = expected_precursor_mz(neutral, Some("[M+Hac-H]-"), None, None);
    assert_mz_close(
        acetate_mz,
        expected_precursor_mz(neutral, Some("[M+CH3COO]-"), None, None),
        "[M+CH3COO]- vs [M+Hac-H]-",
    );
    assert_mz_close(
        acetate_mz,
        expected_precursor_mz(neutral, Some("[M+Hac]-"), None, None),
        "[M+Hac]- vs [M+Hac-H]-",
    );

    let acn_mz = expected_precursor_mz(neutral, Some("[M+ACN+H]+"), None, None);
    assert_mz_close(acn_mz, neutral + 41.026_549_10 + PROTON_MASS, "[M+ACN+H]+");
}

#[test]
fn malformed_and_missing_molecule_adducts_are_rejected() {
    for bad in [
        "",
        "M+H]+",
        "[M+H",
        "[+H]+",
        "[M+H]+foo",
        "[M+Bogus]+",
        "[[M+H]]+",
    ] {
        assert!(!is_supported_adduct(bad), "should reject {bad:?}");
    }
    assert!(is_supported_adduct("[M+Mg]2+"));
    assert!(is_supported_adduct("[M+ACN+H]+"));
}
