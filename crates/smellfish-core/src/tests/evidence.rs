// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the smellfish-rs project

use crate::evidence::assessment::{EvidenceInputs, assess_np_evidence, np_likeness_label};
use crate::evidence::chemist::{
    is_decoration_motif, is_known_np_motif, is_scaffold_motif, run_checks,
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
    assert_eq!(np_likeness_label(Some(5.0)), "strong natural product");
    assert_eq!(np_likeness_label(Some(2.0)), "strong natural product");
    assert_eq!(np_likeness_label(Some(1.999)), "NP-like");
    assert_eq!(np_likeness_label(Some(1.0)), "NP-like");
    assert_eq!(np_likeness_label(Some(0.999)), "NP-ambiguous");
    assert_eq!(np_likeness_label(Some(0.0)), "NP-ambiguous");
    assert_eq!(np_likeness_label(Some(-0.001)), "synthetic-leaning");
    assert_eq!(np_likeness_label(Some(-1.999)), "synthetic-leaning");
    assert_eq!(np_likeness_label(Some(-2.0)), "highly synthetic");
    assert_eq!(np_likeness_label(None), "unavailable");
    assert_eq!(np_likeness_label(Some(f64::NAN)), "unavailable");
    assert_eq!(np_likeness_label(Some(f64::INFINITY)), "unavailable");
}

#[test]
fn real_ertl_score_notes_stereocenters_and_ring_family() {
    let desc = RdkitDescriptors {
        fraction_csp3: Some(0.68),
        ring_count: Some(4.0),
        aromatic_ring_count: Some(0.0),
        aliphatic_ring_count: Some(2.0),
    };
    let ctx = DatasetMotifContext::default();
    let assessment = assess_np_evidence(&EvidenceInputs {
        descriptors: &desc,
        motif_hits: &[motif_hit(
            "Steroid-like fused ring",
            SourceClass::Unknown,
            "",
        )],
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
        motif_hits: &[motif_hit("Flavonoid core", SourceClass::Unknown, "")],
        stereo_tags: &[],
        np_score: None,
        dataset_context: &ctx,
        lotus_scaffolds: &[],
    });
    assert_eq!(assessment.np_label, "unavailable");
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

    let mut nan_row = MoleculeRow {
        np_score: Some(NpScore {
            score: f64::NAN,
            confidence: 0.5,
        }),
        ..MoleculeRow::default()
    };
    assert_eq!(
        run_checks(&nan_row).first().map(|c| c.status),
        Some(CheckStatus::Warn)
    );
    nan_row.np_score = None;
    assert_eq!(
        run_checks(&nan_row).first().map(|c| c.status),
        Some(CheckStatus::Warn)
    );
}

#[test]
fn motif_context_handles_mixed_kingdom_and_synthetic_majority() {
    let desc = RdkitDescriptors::default();
    let ctx = DatasetMotifContext::default();
    let kingdom_dominant = assess_np_evidence(&EvidenceInputs {
        descriptors: &desc,
        motif_hits: &[
            motif_hit("m1", SourceClass::Natural, "plants"),
            motif_hit("m2", SourceClass::Natural, ""),
            motif_hit("m3", SourceClass::Natural, ""),
            motif_hit("m4", SourceClass::Synthetic, ""),
        ],
        stereo_tags: &[],
        np_score: None,
        dataset_context: &ctx,
        lotus_scaffolds: &[],
    });
    assert_eq!(
        kingdom_dominant.motif_context,
        MotifContext::KingdomEnriched
    );

    let synthetic_dominant = assess_np_evidence(&EvidenceInputs {
        descriptors: &desc,
        motif_hits: &[
            motif_hit("s1", SourceClass::Natural, "plants"),
            motif_hit("s2", SourceClass::Synthetic, ""),
            motif_hit("s3", SourceClass::Synthetic, ""),
        ],
        stereo_tags: &[],
        np_score: None,
        dataset_context: &ctx,
        lotus_scaffolds: &[],
    });
    assert_eq!(
        synthetic_dominant.motif_context,
        MotifContext::SyntheticLeaning
    );
}
