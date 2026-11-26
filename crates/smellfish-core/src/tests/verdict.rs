// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the smellfish-rs project

use crate::evidence::chemist::{EvidenceCounts, count_evidence};
use crate::evidence::verdict::{EvidenceSignals, classify_np_evidence, row_verdict};
use crate::model::{MoleculeRow, NpScore, RdkitMotifHit, SourceClass, VerdictCategory};

fn counts(
    np_core: usize,
    scaffold: usize,
    decoration: usize,
    natural: usize,
    synthetic: usize,
    kingdom_enriched: usize,
) -> EvidenceCounts {
    EvidenceCounts {
        np_core,
        scaffold,
        decoration,
        natural,
        synthetic,
        unknown: 0,
        kingdom_enriched,
    }
}

fn ev(
    score: f64,
    lotus: bool,
    pubchem: bool,
    lotus_scaffolds: usize,
    c: EvidenceCounts,
) -> EvidenceSignals {
    EvidenceSignals {
        np_score: Some(score),
        has_lotus: lotus,
        has_pubchem: pubchem,
        lotus_scaffolds,
        counts: c,
    }
}

fn ev_none(
    lotus: bool,
    pubchem: bool,
    lotus_scaffolds: usize,
    c: EvidenceCounts,
) -> EvidenceSignals {
    EvidenceSignals {
        np_score: None,
        has_lotus: lotus,
        has_pubchem: pubchem,
        lotus_scaffolds,
        counts: c,
    }
}

fn assert_category_agreement(cases: &[(EvidenceSignals, VerdictCategory)]) {
    for (signals, expected_category) in cases {
        let verdict = classify_np_evidence(signals);
        assert_eq!(
            verdict.category, *expected_category,
            "verdict: {}",
            verdict.message
        );
    }
}

#[test]
fn classifies_unloaded_synthetic_and_lotus_verdicts() {
    assert_category_agreement(&[
        (
            ev_none(false, false, 0, counts(0, 0, 0, 0, 0, 0)),
            VerdictCategory::Skeptical,
        ),
        (
            ev_none(false, false, 1, counts(0, 0, 0, 0, 0, 0)),
            VerdictCategory::Skeptical,
        ),
        (
            ev_none(false, false, 1, counts(2, 1, 0, 3, 0, 1)),
            VerdictCategory::Likely,
        ),
        (
            ev_none(true, false, 0, counts(1, 1, 0, 1, 0, 0)),
            VerdictCategory::Likely,
        ),
        (
            ev_none(true, false, 0, counts(0, 0, 0, 0, 0, 0)),
            VerdictCategory::Likely,
        ),
        (
            ev(-2.5, false, false, 0, counts(0, 0, 0, 0, 0, 0)),
            VerdictCategory::Fishy,
        ),
        (
            ev(2.5, false, false, 0, counts(0, 0, 0, 0, 3, 0)),
            VerdictCategory::Caution,
        ),
        (
            ev(2.5, false, false, 0, counts(0, 0, 1, 0, 0, 0)),
            VerdictCategory::Skeptical,
        ),
        (
            ev(-1.0, false, false, 0, counts(0, 0, 0, 0, 0, 0)),
            VerdictCategory::Caution,
        ),
        (
            ev(2.5, true, false, 0, counts(2, 1, 0, 3, 0, 1)),
            VerdictCategory::Likely,
        ),
        (
            ev(1.2, true, false, 0, counts(1, 1, 0, 1, 0, 0)),
            VerdictCategory::Likely,
        ),
        (
            ev(1.2, true, true, 0, counts(0, 0, 0, 1, 0, 0)),
            VerdictCategory::Likely,
        ),
    ]);
}

#[test]
fn classifies_scaffolds_pubchem_and_unbacked_verdicts() {
    assert_category_agreement(&[
        (
            ev(2.5, false, false, 1, counts(0, 0, 0, 0, 0, 0)),
            VerdictCategory::Skeptical,
        ),
        (
            ev(2.5, false, false, 1, counts(2, 1, 0, 3, 0, 1)),
            VerdictCategory::Likely,
        ),
        (
            ev(4.76, false, false, 9, counts(0, 0, 0, 12, 0, 7)),
            VerdictCategory::Likely,
        ),
        (
            ev(1.2, false, false, 1, counts(0, 0, 0, 1, 0, 0)),
            VerdictCategory::Likely,
        ),
        (
            ev(1.2, false, false, 1, counts(0, 0, 0, 0, 0, 0)),
            VerdictCategory::Skeptical,
        ),
        (
            ev(0.8, false, false, 1, counts(0, 0, 0, 1, 0, 0)),
            VerdictCategory::Skeptical,
        ),
        (
            ev(2.6, false, true, 0, counts(2, 1, 0, 3, 0, 1)),
            VerdictCategory::Likely,
        ),
        (
            ev(2.2, false, true, 0, counts(0, 0, 0, 0, 0, 0)),
            VerdictCategory::Skeptical,
        ),
        (
            ev(1.2, false, true, 0, counts(1, 1, 0, 1, 0, 0)),
            VerdictCategory::Neutral,
        ),
        (
            ev(1.2, false, true, 0, counts(0, 0, 0, 0, 0, 0)),
            VerdictCategory::Skeptical,
        ),
        (
            ev(0.6, false, true, 0, counts(0, 0, 0, 0, 0, 0)),
            VerdictCategory::Caution,
        ),
        (
            ev(2.6, false, false, 0, counts(2, 1, 0, 4, 0, 1)),
            VerdictCategory::Likely,
        ),
        (
            ev(2.6, false, false, 0, counts(2, 1, 0, 4, 0, 0)),
            VerdictCategory::Skeptical,
        ),
        (
            ev(1.4, false, false, 0, counts(1, 1, 0, 1, 0, 0)),
            VerdictCategory::Skeptical,
        ),
        (
            ev(0.6, false, false, 0, counts(0, 0, 0, 0, 0, 0)),
            VerdictCategory::Skeptical,
        ),
        (
            ev(0.3, false, false, 0, counts(0, 0, 0, 0, 0, 0)),
            VerdictCategory::Skeptical,
        ),
    ]);
}

#[test]
fn row_verdict_routes_errors_and_database_evidence() {
    let mut err_row = MoleculeRow {
        error: Some("invalid SMILES".to_string()),
        ..MoleculeRow::default()
    };
    let err_v = row_verdict(&err_row);
    assert_eq!(err_v.category, VerdictCategory::Neutral);
    assert_eq!(err_v.message, "Error: invalid SMILES");

    err_row.error = None;
    err_row.lotus_taxa = vec!["Plantae".to_string()];
    assert!(
        row_verdict(&err_row)
            .message
            .contains("LOTUS organism record (Ertl unavailable)")
    );

    err_row.np_score = Some(NpScore {
        score: 2.4,
        confidence: 0.9,
    });
    err_row.motifs = vec!["Steroid".to_string()];
    err_row.motif_hits = vec![RdkitMotifHit {
        label: "Steroid".to_string(),
        source_class: SourceClass::Natural,
        kingdom: "plants".to_string(),
        kingdoms: vec!["plants".to_string()],
    }];
    let strong = row_verdict(&err_row);
    assert_eq!(strong.category, VerdictCategory::Likely);
    assert!(strong.message.contains("LOTUS + strong NP evidence"));
}

#[test]
fn count_evidence_feeds_classifier_for_novel_np() {
    let motifs = [
        "Steroid-like fused ring".to_string(),
        "Flavone ring".to_string(),
    ];
    let hits = [
        RdkitMotifHit {
            label: "Flavone ring".to_string(),
            source_class: SourceClass::Natural,
            kingdom: "plants".to_string(),
            kingdoms: vec!["plants".to_string()],
        },
        RdkitMotifHit {
            label: "Steroid fused ring".to_string(),
            source_class: SourceClass::Natural,
            kingdom: "plants".to_string(),
            kingdoms: vec!["plants".to_string()],
        },
    ];
    let counts = count_evidence(&motifs, &hits);
    assert_eq!(counts.np_core, 2);
    assert_eq!(counts.scaffold, 2);
    assert_eq!(counts.natural, 2);
    assert_eq!(counts.kingdom_enriched, 2);
    let verdict = classify_np_evidence(&ev(2.6, false, false, 0, counts));
    assert!(
        verdict.message.contains("Likely novel NP"),
        "{}",
        verdict.message
    );
}

#[test]
fn non_finite_scores_do_not_classify_as_fishy_and_decorations_do_not_penalize_natural_glycosides() {
    assert_eq!(
        classify_np_evidence(&ev(f64::NAN, false, false, 0, counts(0, 0, 0, 0, 0, 0))).category,
        VerdictCategory::Skeptical
    );
    assert_eq!(
        classify_np_evidence(&ev(
            f64::INFINITY,
            false,
            false,
            0,
            counts(0, 0, 0, 0, 0, 0)
        ))
        .category,
        VerdictCategory::Skeptical
    );
    assert_eq!(
        classify_np_evidence(&ev(2.8, false, false, 0, counts(2, 1, 6, 3, 0, 1))).category,
        VerdictCategory::Likely
    );
}
