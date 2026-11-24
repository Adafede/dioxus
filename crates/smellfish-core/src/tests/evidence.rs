// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the smellfish-rs project

use crate::evidence::{
    EvidenceInputs, assess_np_evidence, is_decoration_motif, is_known_np_motif, is_scaffold_motif,
    np_likeness_label, run_checks,
};
use crate::model::{
    CheckStatus, DatasetMotifContext, MoleculeRow, MotifContext, NpScore, RdkitDescriptors,
    RdkitMotifHit, RingFamily, SourceClass,
};
use std::collections::HashMap;

fn motif_hit(label: &str, source_class: SourceClass, kingdom: &str) -> RdkitMotifHit {
    RdkitMotifHit {
        label: label.to_string(),
        source_class,
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
    assert_eq!(np_likeness_label(1.999), "NP-ambiguous");
    assert_eq!(np_likeness_label(0.5), "NP-ambiguous");
    assert_eq!(np_likeness_label(0.499), "weak NP signals");
    assert_eq!(np_likeness_label(-1.0), "weak NP signals");
    assert_eq!(np_likeness_label(-1.001), "highly synthetic");
}

#[test]
fn real_ertl_score_stereo_and_no_hallucinated_rules() {
    let desc = RdkitDescriptors {
        fraction_csp3: Some(0.68),
        ring_count: Some(4.0),
        aromatic_ring_count: Some(0.0),
        aliphatic_ring_count: Some(2.0),
    };
    let ctx = DatasetMotifContext::default();
    let assessment = assess_np_evidence(&EvidenceInputs {
        descriptors: &desc,
        motifs: &["Steroid-like fused ring".to_string()],
        motif_hits: &[],
        stereo_tags: &["R/S".to_string(), "R/S".to_string()],
        np_score: Some(NpScore {
            score: 3.42,
            confidence: 0.75,
        }),
        dataset_context: &ctx,
        lotus_scaffolds: &[],
    });
    assert_eq!(assessment.np_label, "strong natural product");
    assert_eq!(assessment.ring_family, RingFamily::Steroid);
    assert!(
        assessment
            .evidence_notes
            .iter()
            .any(|n| n.contains("stereochemical"))
    );
    assert!(
        assessment
            .evidence_notes
            .iter()
            .all(|n| !n.contains("TPSA") && !n.contains("clogp") && !n.contains("lipophilic"))
    );
}

#[test]
fn model_unavailable_and_dataset_common_motif_notes() {
    let mut ctx = DatasetMotifContext {
        motif_counts: HashMap::new(),
        common_threshold: 2,
    };
    ctx.motif_counts.insert("Flavonoid core".to_string(), 3);
    let desc = RdkitDescriptors::default();
    let assessment = assess_np_evidence(&EvidenceInputs {
        descriptors: &desc,
        motifs: &["Flavonoid core".to_string()],
        motif_hits: &[],
        stereo_tags: &[],
        np_score: None,
        dataset_context: &ctx,
        lotus_scaffolds: &[],
    });
    assert_eq!(assessment.np_label, "weak NP signals");
    assert!(
        assessment
            .evidence_notes
            .iter()
            .any(|n| n.contains("model not loaded"))
    );
    assert!(
        assessment
            .evidence_notes
            .iter()
            .any(|n| n.contains("shared with dataset"))
    );
}

#[test]
fn motif_balance_and_context_detection() {
    assert!(!is_known_np_motif("Methoxy"));
    assert!(is_decoration_motif("Methoxy"));
    assert!(!is_scaffold_motif("Methoxy"));

    let desc = RdkitDescriptors::default();
    let ctx = DatasetMotifContext::default();
    let empty = assess_np_evidence(&EvidenceInputs {
        descriptors: &desc,
        motifs: &[],
        motif_hits: &[],
        stereo_tags: &[],
        np_score: Some(NpScore {
            score: 0.2,
            confidence: 0.9,
        }),
        dataset_context: &ctx,
        lotus_scaffolds: &[],
    });
    assert_eq!(empty.motif_context, MotifContext::NoneDetected);
    assert!(!empty.motif_context.is_detected());

    let assessment = assess_np_evidence(&EvidenceInputs {
        descriptors: &desc,
        motifs: &[
            "Steroid".to_string(),
            "Lactone".to_string(),
            "Sugar".to_string(),
            "Sulfonamide".to_string(),
        ],
        motif_hits: &[
            motif_hit("Steroid", SourceClass::Natural, "plants"),
            motif_hit("Lactone", SourceClass::Natural, ""),
            motif_hit("Sugar", SourceClass::Natural, "fungi"),
            motif_hit("Sulfonamide", SourceClass::Synthetic, ""),
        ],
        stereo_tags: &[],
        np_score: Some(NpScore {
            score: 1.5,
            confidence: 0.9,
        }),
        dataset_context: &ctx,
        lotus_scaffolds: &[],
    });
    assert!(assessment.motif_context.is_detected());
    assert!(
        assessment
            .evidence_notes
            .iter()
            .any(|n| n.contains("(75% natural, 25% synthetic)"))
    );
}

#[test]
fn run_checks_evaluates_score_skeleton_and_databases() {
    let mut row = MoleculeRow {
        ring_family: RingFamily::Polyaromatic,
        ..MoleculeRow::default()
    };
    let checks = run_checks(&row);
    assert_eq!(
        checks.iter().map(|c| c.status).collect::<Vec<_>>(),
        vec![CheckStatus::Warn, CheckStatus::Fail, CheckStatus::Fail]
    );

    row.np_score = Some(NpScore {
        score: 1.4,
        confidence: 0.9,
    });
    row.ring_family = RingFamily::Steroid;
    row.lotus_taxa = vec!["Taxus".to_string()];
    row.pubchem_cids = vec!["441467".to_string()];
    let pass = run_checks(&row);
    assert_eq!(
        pass.iter().map(|c| c.status).collect::<Vec<_>>(),
        vec![CheckStatus::Pass, CheckStatus::Pass, CheckStatus::Pass]
    );
    assert_eq!(
        pass.get(2).map(|c| c.detail.as_str()),
        Some("LOTUS + PubChem")
    );
}
