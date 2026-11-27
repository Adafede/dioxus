// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the lipid-selecto-rs project

use crate::LipidFormat;

#[test]
fn detects_mgf_by_extension() {
    assert_eq!(LipidFormat::from_path("data.mgf"), Some(LipidFormat::Mgf));
    assert_eq!(LipidFormat::from_path("DATA.MGF"), Some(LipidFormat::Mgf));
    assert_eq!(LipidFormat::from_path("notes.txt"), None);
}

#[test]
fn detects_smiles_by_extension() {
    assert_eq!(
        LipidFormat::from_path("compounds.smi"),
        Some(LipidFormat::Smiles)
    );
    assert_eq!(
        LipidFormat::from_path("compounds.SMILES"),
        Some(LipidFormat::Smiles)
    );
    assert_eq!(
        LipidFormat::from_path("compounds.smi.txt"),
        Some(LipidFormat::Smiles)
    );
}

#[test]
fn canonical_extensions_match_expected_suffixes() {
    assert_eq!(LipidFormat::Mgf.extension(), "mgf");
    assert_eq!(LipidFormat::Smiles.extension(), "smi");
}
