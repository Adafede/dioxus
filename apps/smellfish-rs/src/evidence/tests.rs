// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the smellfish-rs project

// The `tests` tests, extracted from `mod.rs`.
//
// They were inline, and together they were several times the size of the
// code they cover. `super` is still the `mod` module, so
// `use super::*` below reaches exactly what it did before the move.
use super::chemist::is_decoration_motif;
use super::*;
use crate::model::{DatasetMotifContext, RdkitDescriptors, RdkitMotifHit};
use std::collections::HashMap;

fn empty_descriptors() -> RdkitDescriptors {
    RdkitDescriptors::default()
}

fn empty_dataset_context() -> DatasetMotifContext {
    DatasetMotifContext::default()
}

fn motif_hit(label: &str, source_class: &str, kingdom: &str) -> RdkitMotifHit {
    RdkitMotifHit {
        label: label.to_string(),
        source_class: source_class.to_string(),
        kingdom: kingdom.to_string(),
        kingdoms: if kingdom.is_empty() {
            Vec::new()
        } else {
            vec![kingdom.to_string()]
        },
    }
}

#[test]
fn label_thresholds_match_ertl_distribution() {
    assert_eq!(np_likeness_label(5.0), "strong natural product");
    assert_eq!(np_likeness_label(2.0), "strong natural product");
    assert_eq!(np_likeness_label(0.8), "NP-ambiguous");
    assert_eq!(np_likeness_label(0.5), "NP-ambiguous");
    assert_eq!(np_likeness_label(0.0), "weak NP signals");
    assert_eq!(np_likeness_label(-0.5), "weak NP signals");
    assert_eq!(np_likeness_label(-1.0), "weak NP signals");
    assert_eq!(np_likeness_label(-1.5), "highly synthetic");
    assert_eq!(np_likeness_label(-5.0), "highly synthetic");
}

#[test]
fn label_boundary_just_above_strong_threshold() {
    assert_eq!(np_likeness_label(2.001), "strong natural product");
}

#[test]
fn label_boundary_just_below_strong_threshold() {
    assert_eq!(np_likeness_label(1.999), "NP-ambiguous");
}

#[test]
fn label_boundary_just_above_ambiguous_threshold() {
    assert_eq!(np_likeness_label(0.501), "NP-ambiguous");
}

#[test]
fn label_boundary_just_below_ambiguous_threshold() {
    assert_eq!(np_likeness_label(0.499), "weak NP signals");
}

#[test]
fn label_boundary_just_below_weak_threshold() {
    assert_eq!(np_likeness_label(-1.001), "highly synthetic");
}

#[test]
fn label_boundary_at_weak_threshold() {
    assert_eq!(np_likeness_label(-1.0), "weak NP signals");
}

#[test]
fn real_ertl_score_is_used() {
    let desc = RdkitDescriptors {
        fraction_csp3: Some(0.68),
        ring_count: Some(4.0),
        aromatic_ring_count: Some(0.0),
        aliphatic_ring_count: Some(2.0),
    };
    let assessment = assess_np_evidence(EvidenceInputs {
        descriptors: &desc,
        motifs: &["Steroid-like fused ring".to_string()],
        motif_hits: &[],
        stereo_tags: &["R/S".to_string(), "R/S".to_string()],
        np_score: Some(3.42),
        np_confidence: Some(0.75),
        dataset_context: &empty_dataset_context(),
        lotus_scaffolds: &[],
    });
    assert!((assessment.np_likeness - 3.42).abs() < 1e-9);
    assert_eq!(assessment.np_label, "strong natural product");
    assert_ne!(assessment.ring_family, "");
    assert!(assessment.np_confidence > 0.5);
    assert!(
        assessment
            .evidence_notes
            .iter()
            .any(|n| n.contains("Ertl NP-likeness score"))
    );
    assert!(
        assessment
            .evidence_notes
            .iter()
            .any(|n| n.contains("stereochemical"))
    );
}

#[test]
fn no_made_up_tpsa_rule() {
    let desc = RdkitDescriptors::default();
    let assessment = assess_np_evidence(EvidenceInputs {
        descriptors: &desc,
        motifs: &[],
        motif_hits: &[],
        stereo_tags: &[],
        np_score: Some(1.5),
        np_confidence: Some(0.8),
        dataset_context: &empty_dataset_context(),
        lotus_scaffolds: &[],
    });
    // No "polar enough" note should exist — that was a hallucinated rule.
    assert!(
        assessment
            .evidence_notes
            .iter()
            .all(|n| !n.contains("polar enough"))
    );
    assert!(
        assessment
            .evidence_notes
            .iter()
            .all(|n| !n.contains("TPSA"))
    );
}

#[test]
fn no_made_up_clogp_rule() {
    let desc = RdkitDescriptors::default();
    let assessment = assess_np_evidence(EvidenceInputs {
        descriptors: &desc,
        motifs: &[],
        motif_hits: &[],
        stereo_tags: &[],
        np_score: Some(-1.5),
        np_confidence: Some(0.9),
        dataset_context: &empty_dataset_context(),
        lotus_scaffolds: &[],
    });
    // No clogP-based notes should exist — those were hallucinated rules.
    assert!(
        assessment
            .evidence_notes
            .iter()
            .all(|n| !n.contains("clogp") && !n.contains("ClogP") && !n.contains("lipophilic"))
    );
}

#[test]
fn model_unavailable_is_handled() {
    let assessment = assess_np_evidence(EvidenceInputs {
        descriptors: &empty_descriptors(),
        motifs: &["Flavonoid core".to_string()],
        motif_hits: &[],
        stereo_tags: &[],
        np_score: None,
        np_confidence: None,
        dataset_context: &empty_dataset_context(),
        lotus_scaffolds: &[],
    });
    assert!((assessment.np_likeness - 0.0).abs() < 1e-9);
    assert_eq!(assessment.np_label, "weak NP signals");
    assert_ne!(assessment.ring_family, "");
    assert!(
        assessment
            .evidence_notes
            .iter()
            .any(|n| n.contains("model not loaded"))
    );
}

#[test]
fn dataset_common_motif_is_noted() {
    let desc = empty_descriptors();
    let mut ctx = DatasetMotifContext {
        motif_counts: HashMap::new(),
        common_threshold: 2,
    };
    ctx.motif_counts.insert("Flavonoid core".to_string(), 3);
    ctx.motif_counts.insert("Aldehyde".to_string(), 1);

    let assessment = assess_np_evidence(EvidenceInputs {
        descriptors: &desc,
        motifs: &["Flavonoid core".to_string(), "Aldehyde".to_string()],
        motif_hits: &[],
        stereo_tags: &[],
        np_score: Some(-0.5),
        np_confidence: Some(0.5),
        dataset_context: &ctx,
        lotus_scaffolds: &[],
    });
    assert!(
        assessment
            .evidence_notes
            .iter()
            .any(|n| n.contains("motif") || n.contains("Flavonoid"))
    );
}

#[test]
fn scaffold_hit_is_counted_as_structure() {
    let assessment = assess_np_evidence(EvidenceInputs {
        descriptors: &empty_descriptors(),
        motifs: &["geminal dimethyl".to_string()],
        motif_hits: &[motif_hit("geminal dimethyl", "natural", "plants")],
        stereo_tags: &[],
        np_score: Some(0.2),
        np_confidence: Some(0.9),
        dataset_context: &empty_dataset_context(),
        lotus_scaffolds: &[],
    });
    assert_ne!(assessment.motif_context, "no structural motifs detected");
    assert!(
        assessment
            .evidence_notes
            .iter()
            .any(|n| n.contains("motif balance"))
    );
}

#[test]
fn kingdom_enriched_hits_are_distinct() {
    let assessment = assess_np_evidence(EvidenceInputs {
        descriptors: &empty_descriptors(),
        motifs: &["geminal dimethyl".to_string(), "CC=C".to_string()],
        motif_hits: &[
            motif_hit("geminal dimethyl", "natural", "plants"),
            motif_hit("CC=C", "natural", ""),
        ],
        stereo_tags: &[],
        np_score: Some(1.1),
        np_confidence: Some(0.9),
        dataset_context: &empty_dataset_context(),
        lotus_scaffolds: &[],
    });
    assert!(
        assessment
            .evidence_notes
            .iter()
            .any(|n| n.contains("kingdom-enriched"))
    );
}

#[test]
fn decoration_motifs_stay_neutral() {
    assert!(!is_known_np_motif("Methoxy"));
    assert!(is_decoration_motif("Methoxy"));
    assert!(!is_scaffold_motif("Methoxy"));
}

#[test]
fn motif_balance_formats_true_percentages() {
    let assessment = assess_np_evidence(EvidenceInputs {
        descriptors: &empty_descriptors(),
        motifs: &[
            "Steroid".to_string(),
            "Lactone".to_string(),
            "Sugar".to_string(),
            "Sulfonamide".to_string(),
        ],
        motif_hits: &[
            motif_hit("Steroid", "natural", "plants"),
            motif_hit("Lactone", "natural", ""),
            motif_hit("Sugar", "natural", "fungi"),
            motif_hit("Sulfonamide", "synthetic", ""),
        ],
        stereo_tags: &[],
        np_score: Some(1.5),
        np_confidence: Some(0.9),
        dataset_context: &empty_dataset_context(),
        lotus_scaffolds: &[],
    });
    assert!(
        assessment
            .evidence_notes
            .iter()
            .any(|n| n.contains("(75% natural, 25% synthetic)")),
        "expected 75% natural / 25% synthetic in motif balance note: {:?}",
        assessment.evidence_notes
    );
}

fn sample_row() -> crate::model::MoleculeRow {
    crate::model::MoleculeRow {
        index: 1,
        label: "Sample".to_string(),
        smiles: "C".to_string(),
        #[cfg(target_arch = "wasm32")]
        inchikey: String::new(),
        svg: None,
        motifs: Vec::new(),
        substituents_counts: std::collections::BTreeMap::new(),
        lotus_scaffolds: Vec::new(),
        lotus_taxa: Vec::new(),
        lotus_compounds: Vec::new(),
        lotus_compounds_with_taxa: std::collections::BTreeSet::new(),
        pubchem_cids: Vec::new(),
        np_likeness: 0.0,
        np_label: String::new(),
        np_confidence: 0.0,
        np_score_available: false,
        ring_family: String::new(),
        evidence_notes: Vec::new(),
        motif_context: String::new(),
        chemist_checks: Vec::new(),
        verdict: String::new(),
        error: None,
        #[cfg(target_arch = "wasm32")]
        descriptors: crate::model::RdkitDescriptors::default(),
        #[cfg(target_arch = "wasm32")]
        stereo_tags: Vec::new(),
        num_atoms: 1,
        motif_hits: Vec::new(),
    }
}

#[test]
fn run_checks_evaluates_score_skeleton_and_databases() {
    let mut row = sample_row();
    row.ring_family = "Polyaromatic hydrocarbon".to_string();
    let checks = run_checks(&row);
    assert_eq!(
        checks.iter().map(|c| c.status).collect::<Vec<_>>(),
        vec!["warn", "fail", "fail"]
    );

    row.np_score_available = true;
    row.np_likeness = 1.4;
    row.ring_family = "Steroid tetracycle".to_string();
    row.lotus_taxa = vec!["Taxus".to_string()];
    row.pubchem_cids = vec!["441467".to_string()];
    let pass_checks = run_checks(&row);
    assert_eq!(
        pass_checks.iter().map(|c| c.status).collect::<Vec<_>>(),
        vec!["pass", "pass", "pass"]
    );
    assert_eq!(
        pass_checks.get(2).map(|c| c.detail.as_str()),
        Some("LOTUS + PubChem")
    );
}

#[test]
fn row_verdict_routes_errors_and_database_evidence() {
    let mut err_row = sample_row();
    err_row.error = Some("invalid SMILES".to_string());
    assert_eq!(row_verdict(&err_row), "⚠ invalid SMILES");

    let mut lotus_row = sample_row();
    lotus_row.lotus_taxa = vec!["Plantae".to_string()];
    assert!(
        row_verdict(&lotus_row).contains("LOTUS organism record (Ertl unavailable)"),
        "LOTUS row without Ertl score should report LOTUS organism record"
    );

    lotus_row.np_score_available = true;
    lotus_row.np_likeness = 2.4;
    lotus_row.motifs = vec!["Steroid".to_string()];
    lotus_row.motif_hits = vec![motif_hit("Steroid", "natural", "plants")];
    assert!(
        row_verdict(&lotus_row).contains("LOTUS + strong NP evidence"),
        "LOTUS row with strong Ertl + structural support should report strong evidence"
    );
}
