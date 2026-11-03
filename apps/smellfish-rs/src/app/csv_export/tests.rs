// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the smellfish-rs project

// The `tests` tests, extracted from `csv_export.rs`.
//
// They were inline, and together they were several times the size of the
// code they cover. `super` is still the `csv_export` module, so
// `use super::*` below reaches exactly what it did before the move.
use super::category;

#[test]
fn categorizes_high_quality_pubchem() {
    let verdict = "🌿 PubChem + strong NP evidence — Ertl score +1.50 with 3 NP substituent(s) + 2 NP motif(s).";
    assert_eq!(category(verdict), "likely");
}

#[test]
fn categorizes_moderate_quality_pubchem() {
    let verdict =
        "📚 PubChem hit with NP signals — Ertl score +0.80, 2 substituent(s), 1 NP motif(s).";
    assert_eq!(category(verdict), "neutral");
}

#[test]
fn categorizes_weak_pubchem() {
    let verdict = "📚 PubChem hit — weak NP evidence (Ertl score +0.26).";
    assert_eq!(category(verdict), "caution");
}

#[test]
fn categorizes_lotus_backed() {
    let verdict = "🌿 LOTUS-backed (Ertl score +1.23).";
    assert_eq!(category(verdict), "likely");
}

#[test]
fn categorizes_novel_candidate() {
    let verdict =
        "🌿 Likely hit — strong NP-likeness (+3.43) + NP-like scaffold, not yet in databases.";
    assert_eq!(category(verdict), "likely");
}

#[test]
fn categorizes_fishy() {
    let verdict = "👃 Smells fishy (Ertl score -1.23). Citation needed.";
    assert_eq!(category(verdict), "fishy");
}

#[test]
fn categorizes_synthetic_leaning_as_caution() {
    let verdict = "🟧 Synthetic-leaning structure (Ertl score +2.50).";
    assert_eq!(category(verdict), "caution");
    assert_ne!(category(verdict), "likely");
    assert_ne!(category(verdict), "fishy");
}

#[test]
fn categorizes_lotus_scaffold_hint_as_skeptical() {
    let verdict =
        "👃 Citation needed — LOTUS scaffold hint, insufficient corroboration (Ertl +2.50).";
    assert_eq!(category(verdict), "skeptical");
}

#[test]
fn escape_csv_wraps_fields_with_commas() {
    let escaped = super::escape_csv("hello, world");
    assert_eq!(escaped, "\"hello, world\"");
}

#[test]
fn escape_csv_does_not_wrap_simple_fields() {
    let escaped = super::escape_csv("hello");
    assert_eq!(escaped, "hello");
}

#[test]
fn escape_csv_escapes_internal_quotes() {
    let escaped = super::escape_csv("say \"hi\"");
    assert_eq!(escaped, "\"say \"\"hi\"\"\"");
}

#[test]
fn escape_csv_wraps_fields_with_a_newline() {
    // A newline inside a value silently splits the row in two: the fields
    // after it land on the next line under the wrong header. RFC 4180 says a
    // writer quotes any field holding a line break, so this is the assertion
    // that the writer does.
    let escaped = super::escape_csv("two\nlines");
    assert_eq!(escaped, "\"two\nlines\"");
}

#[test]
fn escape_csv_wraps_fields_with_a_carriage_return() {
    // `\r` on its own is also a line break to a CSV reader, so a bare CR is
    // quoted the same way a bare LF is.
    let escaped = super::escape_csv("old\rmac");
    assert_eq!(escaped, "\"old\rmac\"");
}

#[test]
fn build_csv_formats_rows_and_headers() {
    let row = crate::model::MoleculeRow {
        index: 1,
        label: "Caffeine, test".to_string(),
        smiles: "CN1C=NC2=C1C(=O)N(C(=O)N2C)C".to_string(),
        #[cfg(target_arch = "wasm32")]
        inchikey: String::new(),
        svg: None,
        motifs: Vec::new(),
        substituents_counts: std::collections::BTreeMap::from([
            ("methyl".to_string(), 3),
            ("ethyl".to_string(), 1),
        ]),
        lotus_scaffolds: Vec::new(),
        #[cfg(target_arch = "wasm32")]
        lotus_taxa: Vec::new(),
        lotus_compounds: vec!["Q60235".to_string()],
        lotus_compounds_with_taxa: std::collections::BTreeSet::new(),
        pubchem_cids: vec!["2519".to_string()],
        np_likeness: 1.25,
        np_label: "NP-like".to_string(),
        np_confidence: 0.85,
        np_score_available: true,
        ring_family: "Purine".to_string(),
        evidence_notes: Vec::new(),
        motif_context: String::new(),
        chemist_checks: vec![crate::model::ChemistCheck {
            name: "NP-likeness",
            status: "pass",
            detail: "ok".to_string(),
        }],
        verdict: "🌿 LOTUS-backed (Ertl score +1.25).".to_string(),
        error: None,
        #[cfg(target_arch = "wasm32")]
        descriptors: crate::model::RdkitDescriptors::default(),
        #[cfg(target_arch = "wasm32")]
        stereo_tags: Vec::new(),
        num_atoms: 14,
        motif_hits: Vec::new(),
    };

    let csv = super::build_csv(&[row]);
    assert!(
        csv.starts_with(
            "label,smiles,np_score,np_label,np_confidence,ring_family,substituents,locus,verdict_category,chemist_checks\n"
        ),
        "CSV should start with header row"
    );
    assert!(
        csv.contains("\"Caffeine, test\",CN1C=NC2=C1C(=O)N(C(=O)N2C)C,1.250,NP-like,85%,Purine,ethyl(1);methyl(3),Q60235;2519,likely,NP-likeness:pass"),
        "CSV row should match expected formatting: {csv}"
    );
}
