// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the smellfish-rs project

use crate::SmellfishError;
use crate::csv::{escape_csv, format_csv_rows, parse_csv_rows};
use crate::model::{
    CheckStatus, ChemistCheck, LotusCompound, MoleculeRow, NpScore, RingFamily, Verdict,
    VerdictCategory,
};
use std::collections::BTreeMap;

#[test]
fn detects_smiles_and_label_columns() -> Result<(), SmellfishError> {
    let rows = parse_csv_rows("name,smiles\nalpha,C1CCCCC1\n")?;
    assert_eq!(rows.len(), 1);
    let row = rows.first().ok_or(SmellfishError::NoDataRows)?;
    assert_eq!(row.index, 1);
    assert_eq!(row.label, "alpha");
    assert_eq!(row.smiles, "C1CCCCC1");
    Ok(())
}

#[test]
fn falls_back_to_generated_labels_and_plain_smiles() -> Result<(), SmellfishError> {
    let single = parse_csv_rows("smiles\nCCO\n")?;
    assert_eq!(single.first().map(|r| r.label.as_str()), Some("Molecule 1"));

    let plain = parse_csv_rows("CCO\nC1CCCCC1\n")?;
    assert_eq!(plain.len(), 2);
    assert_eq!(plain.first().map(|r| r.smiles.as_str()), Some("CCO"));
    assert_eq!(plain.get(1).map(|r| r.smiles.as_str()), Some("C1CCCCC1"));
    Ok(())
}

#[test]
fn parses_quoted_fields_bom_and_multiline_cells() -> Result<(), SmellfishError> {
    let rows = parse_csv_rows("name,smiles\r\n\"alpha, \"\"1\"\"\",C1CCCCC1\r\n")?;
    assert_eq!(rows.len(), 1);
    assert_eq!(rows.first().map(|r| r.label.as_str()), Some("alpha, \"1\""));

    let bom = parse_csv_rows("\u{feff}>smiles,name\nCCO,\"Line 1\nLine 2\"\n")?;
    assert_eq!(bom.first().map(|r| r.smiles.as_str()), Some("CCO"));
    assert_eq!(
        bom.first().map(|r| r.label.as_str()),
        Some("Line 1\nLine 2")
    );
    Ok(())
}

#[test]
fn rejects_empty_header_only_and_unterminated_quote() {
    assert!(matches!(
        parse_csv_rows("   \n\t "),
        Err(SmellfishError::EmptyInput)
    ));
    assert!(matches!(
        parse_csv_rows("name,smiles\n"),
        Err(SmellfishError::NoDataRows)
    ));
    assert!(matches!(
        parse_csv_rows("name,smiles\n\"unterminated,CCO"),
        Err(SmellfishError::UnterminatedCsvQuote)
    ));
}

#[test]
fn detect_column_prefers_exact_id_and_falls_back_to_col_zero() -> Result<(), SmellfishError> {
    let rows = parse_csv_rows("lipid,id,smiles\nx,ID1,CCO\n")?;
    assert_eq!(rows.first().map(|r| r.label.as_str()), Some("ID1"));

    let fallback = parse_csv_rows("structure_col,other\nCCN,foo\n")?;
    assert_eq!(fallback.first().map(|r| r.smiles.as_str()), Some("CCN"));
    assert_eq!(
        fallback.first().map(|r| r.label.as_str()),
        Some("Molecule 1")
    );
    Ok(())
}

#[test]
fn escape_csv_handles_rfc4180_and_line_breaks() {
    assert_eq!(escape_csv("hello"), "hello");
    assert_eq!(escape_csv("hello, world"), "\"hello, world\"");
    assert_eq!(escape_csv("say \"hi\""), "\"say \"\"hi\"\"\"");
    assert_eq!(escape_csv("two\nlines"), "\"two\nlines\"");
    assert_eq!(escape_csv("old\rmac"), "\"old\rmac\"");
}

#[test]
fn format_csv_rows_formats_rows_and_neutralizes_formulas() {
    let row = MoleculeRow {
        index: 1,
        label: "Caffeine, test".to_string(),
        smiles: "CN1C=NC2=C1C(=O)N(C(=O)N2C)C".to_string(),
        substituents_counts: BTreeMap::from([("methyl".to_string(), 3), ("ethyl".to_string(), 1)]),
        lotus_compounds: vec![LotusCompound {
            qid: "Q60235".to_string(),
            has_taxa: false,
        }],
        pubchem_cids: vec!["2519".to_string()],
        np_score: Some(NpScore {
            score: 1.25,
            confidence: 0.85,
        }),
        np_label: "NP-like".to_string(),
        ring_family: RingFamily::Heteroaromatic,
        chemist_checks: vec![ChemistCheck {
            name: "NP-likeness",
            status: CheckStatus::Pass,
            detail: "ok".to_string(),
        }],
        verdict: Verdict::new(
            VerdictCategory::Likely,
            "🌿 LOTUS-backed (Ertl score +1.25).",
        ),
        num_atoms: 14,
        ..MoleculeRow::default()
    };

    let csv = format_csv_rows(&[row]);
    assert!(
        csv.contains("\"Caffeine, test\",CN1C=NC2=C1C(=O)N(C(=O)N2C)C,1.250,NP-like,85%,fused heteroaromatic scaffold,ethyl(1);methyl(3),Q60235;2519,likely,NP-likeness:pass"),
        "unexpected CSV row: {csv}"
    );

    let formula_row = MoleculeRow {
        label: "=cmd|'/C calc'!A0".to_string(),
        smiles: "+CCO".to_string(),
        lotus_compounds: vec![LotusCompound {
            qid: "@evil".to_string(),
            has_taxa: false,
        }],
        np_score: Some(NpScore {
            score: -1.25,
            confidence: 0.90,
        }),
        np_label: "Synthetic".to_string(),
        ring_family: RingFamily::Acyclic,
        chemist_checks: vec![ChemistCheck {
            name: "\tCheck",
            status: CheckStatus::Pass,
            detail: "ok".to_string(),
        }],
        verdict: Verdict::new(VerdictCategory::Fishy, "fishy"),
        ..MoleculeRow::default()
    };
    let exported = format_csv_rows(&[formula_row]);
    let data_line = exported.lines().nth(1).unwrap_or("");
    assert!(
        data_line.starts_with("'=cmd|'/C calc'!A0,'+CCO,-1.250,Synthetic,90%,acyclic,"),
        "text cells neutralized while numeric score stays numeric: {data_line}"
    );
    assert!(data_line.contains(",'@evil,"), "{data_line}");
    assert!(data_line.contains(",'\tCheck:pass"), "{data_line}");
}
