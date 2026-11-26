// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the mgf-precursor-erro-rs project

use std::str::FromStr;

use molecular_formulas::prelude::*;

use super::assert_close;
use crate::adduct::parse::{
    AMMONIUM_MASS, ELECTRON_MASS, HYDROGEN_MASS, POTASSIUM_MASS, SODIUM_MASS,
    parse_adduct_mass_components, parse_adduct_term_mass, parse_charge_value,
    parse_polarity_tokens,
};
use crate::adduct::{
    Polarity, expected_precursor_mz, is_supported_adduct, normalize_adduct_label,
    parse_adduct_polarity,
};

#[test]
fn const_adduct_term_masses_match_molecular_formulas_exactly() {
    for (token, formula) in [
        ("FA", "CH2O2"),
        ("HCOO", "CHO2"),
        ("HCOONA", "CHNaO2"),
        ("HAC", "C2H4O2"),
        ("CH3COO", "C2H3O2"),
        ("ACN", "C2H3N"),
        ("DMSO", "C2H6OS"),
        ("ISOPROP", "C3H8O"),
        ("TFA", "C2HF3O2"),
        ("CF3COO", "C2F3O2"),
        ("MEOH", "CH4O"),
        ("H2O", "H2O"),
        ("NH3", "NH3"),
        ("CO", "CO"),
        ("CO2", "CO2"),
        ("O", "O"),
    ] {
        let actual = parse_adduct_term_mass(token).expect(token);
        let expected = ChemicalFormula::<u32, i32>::from_str(formula)
            .unwrap()
            .isotopologue_mass();
        assert_close(actual, expected, 1e-5, token);
    }
}

#[test]
fn theoretical_precursor_mz_common_and_divalent_adducts() {
    let neutral = 500.0;
    assert_close(
        expected_precursor_mz(neutral, Some("[M+H]+"), None, None),
        neutral + HYDROGEN_MASS - ELECTRON_MASS,
        1e-5,
        "[M+H]+",
    );
    assert_close(
        expected_precursor_mz(neutral, Some("[M-H]-"), None, None),
        neutral - HYDROGEN_MASS + ELECTRON_MASS,
        1e-5,
        "[M-H]-",
    );
    assert_close(
        expected_precursor_mz(neutral, Some("[M+Na]+"), None, None),
        neutral + SODIUM_MASS - ELECTRON_MASS,
        1e-5,
        "[M+Na]+",
    );
    assert_close(
        expected_precursor_mz(neutral, Some("[M+NH4]+"), None, None),
        neutral + 18.033_825_55,
        1e-5,
        "[M+NH4]+ exact IUPAC mass",
    );
    assert_close(
        expected_precursor_mz(neutral, Some("[M+Mg]2+"), None, None),
        (neutral + 23.985_041_7 - 2.0 * ELECTRON_MASS) / 2.0,
        1e-5,
        "[M+Mg]2+",
    );
    assert_close(
        expected_precursor_mz(neutral, Some("[M+Ca]2+"), None, None),
        (neutral + 39.962_590_98 - 2.0 * ELECTRON_MASS) / 2.0,
        1e-5,
        "[M+Ca]2+",
    );
    assert_close(
        expected_precursor_mz(neutral, Some("[2M+H]+"), None, None),
        2.0 * neutral + HYDROGEN_MASS - ELECTRON_MASS,
        1e-5,
        "[2M+H]+",
    );
}

#[test]
fn formate_acetate_and_acetonitrile_adducts_agree_across_notations() {
    let neutral = 400.0;
    let formate_mz = expected_precursor_mz(neutral, Some("[M+HCOO]-"), None, None);
    assert_close(
        formate_mz,
        expected_precursor_mz(neutral, Some("[M+FA-H]-"), None, None),
        1e-5,
        "[M+FA-H]- vs [M+HCOO]-",
    );
    assert_close(
        formate_mz,
        expected_precursor_mz(neutral, Some("[M+FA]-"), None, None),
        1e-5,
        "[M+FA]- vs [M+HCOO]-",
    );
    assert_close(
        formate_mz,
        neutral + 44.997_654_29 + ELECTRON_MASS,
        1e-5,
        "[M+FA-H]- theoretical",
    );

    let acetate_mz = expected_precursor_mz(neutral, Some("[M+Hac-H]-"), None, None);
    assert_close(
        acetate_mz,
        expected_precursor_mz(neutral, Some("[M+CH3COO]-"), None, None),
        1e-5,
        "[M+CH3COO]- vs [M+Hac-H]-",
    );
    assert_close(
        acetate_mz,
        expected_precursor_mz(neutral, Some("[M+Hac]-"), None, None),
        1e-5,
        "[M+Hac]- vs [M+Hac-H]-",
    );

    let acn_mz = expected_precursor_mz(neutral, Some("[M+ACN+H]+"), None, None);
    assert_close(
        acn_mz,
        neutral + 41.026_549_10 + HYDROGEN_MASS - ELECTRON_MASS,
        1e-5,
        "[M+ACN+H]+",
    );
}

#[test]
fn malformed_adducts_and_polarity_parse_as_expected() {
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
        assert_eq!(parse_adduct_mass_components(bad), None);
    }
    assert!(is_supported_adduct("[M+Mg]2+"));
    assert!(is_supported_adduct("[M+ACN+H]+"));

    let (multiplier, shift) = parse_adduct_mass_components("[2M+Na]+").unwrap();
    assert_close(multiplier, 2.0, 1e-12, "multiplier");
    assert_close(shift, SODIUM_MASS, 1e-12, "shift");
    assert_eq!(
        parse_adduct_polarity(Some("[2M+Na]+")),
        Some(Polarity::Positive)
    );
    assert_eq!(parse_charge_value(None, Some("[2M+Na]+")), Some(1.0));

    assert_eq!(
        parse_polarity_tokens(Some("+"), None),
        Some(Polarity::Positive)
    );
    assert_eq!(
        parse_polarity_tokens(None, Some("negative")),
        Some(Polarity::Negative)
    );
    assert_eq!(parse_polarity_tokens(None, Some("neutral")), None);
}

#[test]
fn multicharge_without_adduct_unbracketed_and_repeated_sign_charges() {
    for (neutral, adduct, charge, ion_mode, expected, label) in [
        (
            1000.0,
            None,
            Some("2+"),
            None,
            f64::midpoint(1000.0, 2.0 * (HYDROGEN_MASS - ELECTRON_MASS)),
            "CHARGE=2+ without ADDUCT",
        ),
        (
            1000.0,
            None,
            Some("2-"),
            Some("negative"),
            (1000.0 - 2.0 * (HYDROGEN_MASS - ELECTRON_MASS)) / 2.0,
            "CHARGE=2- negative without ADDUCT",
        ),
        (
            1000.0,
            None,
            Some("++"),
            None,
            f64::midpoint(1000.0, 2.0 * (HYDROGEN_MASS - ELECTRON_MASS)),
            "CHARGE=++ without ADDUCT",
        ),
        (
            1000.0,
            None,
            Some("2"),
            Some("positive"),
            f64::midpoint(1000.0, 2.0 * (HYDROGEN_MASS - ELECTRON_MASS)),
            "unsigned CHARGE=2 with IONMODE=positive",
        ),
        (
            1000.0,
            None,
            Some("2"),
            Some("negative"),
            (1000.0 - 2.0 * (HYDROGEN_MASS - ELECTRON_MASS)) / 2.0,
            "unsigned CHARGE=2 with IONMODE=negative",
        ),
        (
            500.0,
            Some("M+H"),
            None,
            None,
            500.0 + HYDROGEN_MASS - ELECTRON_MASS,
            "unbracketed M+H",
        ),
        (
            500.0,
            Some("M+Na"),
            None,
            None,
            500.0 + SODIUM_MASS - ELECTRON_MASS,
            "unbracketed M+Na",
        ),
        (
            500.0,
            Some("M-H"),
            None,
            None,
            500.0 - HYDROGEN_MASS + ELECTRON_MASS,
            "unbracketed M-H",
        ),
        (
            500.0,
            Some("[M+Bogus]+"),
            Some("1+"),
            None,
            500.0 + HYDROGEN_MASS - ELECTRON_MASS,
            "unparsable adduct fallback applies single electron adjustment",
        ),
    ] {
        let actual = expected_precursor_mz(neutral, adduct, charge, ion_mode);
        assert_close(actual, expected, 1e-6, label);
    }
}

#[test]
fn normalize_adduct_label_and_term_masses() {
    for (input, expected) in [
        ("", "unknown"),
        ("   ", "unknown"),
        ("[m]+", "[M]+"),
        ("[m]2+", "[M]2+"),
        ("[m]++", "[M]2+"),
        ("[m+h]", "[M+H]+"),
        ("[m+k]+", "[M+K]+"),
        ("[m+nh4]", "[M+NH4]+"),
        ("[m+na]", "[M+Na]+"),
        ("[m+2na]++", "[M+2Na]2+"),
        ("[m+2h]2+", "[M+2H]2+"),
        ("[m-h]1-", "[M-H]-"),
        ("[m-2h]--", "[M-2H]2-"),
        ("[4m-h]-", "[4M-H]-"),
        (" [M+ACN+H]+ ", "[M+ACN+H]+"),
    ] {
        assert_eq!(normalize_adduct_label(input), expected, "input={input:?}");
    }

    assert_close(
        parse_adduct_term_mass("H").unwrap(),
        HYDROGEN_MASS,
        1e-9,
        "H",
    );
    assert_close(
        parse_adduct_term_mass("2H").unwrap(),
        2.0 * HYDROGEN_MASS,
        1e-9,
        "2H",
    );
    assert_close(
        parse_adduct_term_mass("NA").unwrap(),
        SODIUM_MASS,
        1e-9,
        "NA",
    );
    assert_close(
        parse_adduct_term_mass("K").unwrap(),
        POTASSIUM_MASS,
        1e-9,
        "K",
    );
    assert_close(
        parse_adduct_term_mass("NH4").unwrap(),
        AMMONIUM_MASS,
        1e-9,
        "NH4",
    );
    assert_eq!(parse_adduct_term_mass("M"), None);
    assert_eq!(parse_adduct_term_mass(""), None);
    assert_eq!(parse_adduct_term_mass("0H"), None);
    assert_eq!(parse_adduct_term_mass("BogusTerm"), None);
    assert!(!is_supported_adduct("[M+M+H]+"));
    assert!(!is_supported_adduct("[M++H]+"));
    assert!(!is_supported_adduct("[0M+H]+"));
    assert!(!is_supported_adduct("[M+0H]+"));
    assert_eq!(parse_charge_value(Some("2+ and 3+"), None), Some(2.0));
    assert_eq!(parse_charge_value(Some("0"), None), None);
    assert_eq!(normalize_adduct_label("[M-H]--"), "[M-H]--");
    assert_eq!(normalize_adduct_label("[M+2H]+"), "[M+2H]+");
}
