// The `tests` tests, extracted from `format.rs`.
//
// They were inline, and together they were several times the size of the
// code they cover. `super` is still the `format` module, so
// `use super::*` below reaches exactly what it did before the move.
use super::*;

#[test]
fn detects_mgf_by_extension() {
    let format = LipidFormat::from_path("data.mgf");
    assert_eq!(format, Some(LipidFormat::Mgf));
}

#[test]
fn detects_smiles_by_extension() {
    let format = LipidFormat::from_path("compounds.smi");
    assert_eq!(format, Some(LipidFormat::Smiles));
}

#[test]
fn detects_mgf_by_content() {
    let mgf_content = "BEGIN IONS\nTITLE=spectrum_1\nEND IONS";
    let format = LipidFormat::detect_from_content(mgf_content);
    assert_eq!(format, Some(LipidFormat::Mgf));
}

#[test]
fn detects_smiles_by_content() {
    let smiles_content = "CCCCCCCCCCCCCCCC(=O)O\nCC(C)CC(N)C(=O)O";
    let format = LipidFormat::detect_from_content(smiles_content);
    assert_eq!(format, Some(LipidFormat::Smiles));
}

#[test]
fn returns_correct_extensions() {
    assert_eq!(LipidFormat::Mgf.extension(), "mgf");
    assert_eq!(LipidFormat::Smiles.extension(), "smi");
}
