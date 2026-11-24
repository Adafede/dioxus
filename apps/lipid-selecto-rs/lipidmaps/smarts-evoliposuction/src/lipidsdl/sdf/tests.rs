// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the lipid-selecto-rs project

//! Unit tests for the streaming SDF parser and LMSD TSV converter.

#![expect(clippy::unwrap_used)] // tests unwrap fixtures to fail-fast rather than assert on the error
#![expect(clippy::indexing_slicing)] // tests reach into fixture rows and columns by index

use super::*;

const SAMPLE_SDF: &str = r"  CDK     2DS

  20 21  0  0  0  0  0  0  0  0999 V2000
    2.3120    0.1280    0.0000 C  0  0  0  0  0  0  0  0  0  0  0  0
M  END
> <LM_ID>
LMFA00000001

> <NAME>
Test fatty acid

> <SMILES>
C(C(C(C(=O)O)O)O)O

$$$$

  CDK     2DS

  15 16  0  0  0  0  0  0  0  0999 V2000
    1.2340    0.5670    0.0000 C  0  0  0  0  0  0  0  0  0  0  0  0
M  END
> <LM_ID>
LMFA00000002

> <NAME>
Missing SMILES lipid

> <SMILES>

$$$$
";

#[test]
fn parse_sdf_extracts_properties() {
    let parser = Parser::new(SAMPLE_SDF);
    let records: Vec<_> = parser.collect();
    assert_eq!(records.len(), 2);

    let rec1 = &records[0];
    assert_eq!(rec1.get("LM_ID"), Some("LMFA00000001"));
    assert_eq!(rec1.get("NAME"), Some("Test fatty acid"));
    assert_eq!(rec1.get("SMILES"), Some("C(C(C(C(=O)O)O)O)O"));
}

#[test]
fn to_lmsd_tsv_filters_empty_smiles() {
    let (tsv, count) = lipidmaps::to_lmsd_tsv(SAMPLE_SDF);
    assert_eq!(count, 1);
    assert!(tsv.contains("LMFA00000001\tTest fatty acid"));
    assert!(!tsv.contains("LMFA00000002"));
}

#[test]
fn to_lmsd_tsv_has_all_columns() {
    let (tsv, _) = lipidmaps::to_lmsd_tsv(SAMPLE_SDF);
    let header_line = tsv.lines().next().unwrap();
    for col in lipidmaps::COLUMNS {
        assert!(
            header_line.contains(col),
            "column {col} missing from header"
        );
    }
    assert_eq!(lipidmaps::COLUMNS.len(), 19);
}

#[test]
fn empty_sdf_yields_empty_tsv() {
    let (tsv, count) = lipidmaps::to_lmsd_tsv("");
    assert_eq!(count, 0);
    assert!(tsv.starts_with("LM_ID\tNAME"));
}

#[test]
fn to_lmsd_tsv_handles_missing_properties() {
    let sdf = "> <LM_ID>\nLMFA00000003\n\n> <SMILES>\nC(=O)O\n\n$$$$\n";
    let (tsv, count) = lipidmaps::to_lmsd_tsv(sdf);
    assert_eq!(count, 1);
    let lines: Vec<&str> = tsv.lines().collect();
    assert_eq!(lines.len(), 2);
    let field_count = lines[1].split('\t').count();
    assert_eq!(field_count, lipidmaps::COLUMNS.len());
}

#[test]
fn to_lmsd_tsv_multi_line_value() {
    let sdf = "> <LM_ID>\nLMFA00000004\n\n> <SYNONYMS>\nSyn1\nSyn2\n\n> <SMILES>\nCC\n\n$$$$\n";
    let (tsv, count) = lipidmaps::to_lmsd_tsv(sdf);
    assert_eq!(count, 1);
    let lines: Vec<&str> = tsv.lines().collect();
    let cols: Vec<&str> = lines[1].split('\t').collect();
    let syn_idx = lipidmaps::COLUMNS
        .iter()
        .position(|&c| c == "SYNONYMS")
        .unwrap();
    assert_eq!(cols[syn_idx], "Syn1; Syn2");
}

#[test]
fn parse_sdf_supports_extra_header_spaces_and_sanitizes_tabs_and_crlf() {
    let sdf = ">  <LM_ID>\r\nLMFA01010001\r\n\r\n>   <SYNONYMS>\r\na\tb\r\nc\r\n\r\n>  <SMILES>\r\nCC\r\n\r\n$$$$\r\n";
    let (tsv, count) = lipidmaps::to_lmsd_tsv(sdf);
    assert_eq!(count, 1);
    let lines: Vec<&str> = tsv.lines().collect();
    assert_eq!(lines.len(), 2, "CRLF inside value must not split TSV rows");
    let cols: Vec<&str> = lines[1].split('\t').collect();
    assert_eq!(cols.len(), lipidmaps::COLUMNS.len());
    assert_eq!(cols[0], "LMFA01010001");
    let syn_idx = lipidmaps::COLUMNS
        .iter()
        .position(|&c| c == "SYNONYMS")
        .unwrap();
    assert_eq!(cols[syn_idx], "a b; c");
}
