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
    let counts = count_evidence(&hits);
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

#[test]
fn classification_enums_and_verdict_monotonicity_hold() {
    use crate::model::{CheckStatus, InChIKey14, MotifContext, RingFamily};

    assert_eq!(
        RingFamily::default(),
        RingFamily::Acyclic,
        "default ring family is acyclic"
    );
    for fam in [
        RingFamily::Steroid,
        RingFamily::Sugar,
        RingFamily::Macrolide,
        RingFamily::Flavonoid,
        RingFamily::Heteroaromatic,
        RingFamily::Polyaromatic,
        RingFamily::MixedAromaticAliphatic,
        RingFamily::Polycyclic,
        RingFamily::CompactRing,
        RingFamily::Acyclic,
    ] {
        assert_eq!(fam.to_string(), fam.as_str(), "Display matches as_str");
    }

    for ctx in [
        MotifContext::ErtlCoreWithSupport,
        MotifContext::ErtlCore,
        MotifContext::KingdomEnrichedStructural,
        MotifContext::NaturalStructural,
        MotifContext::StructuralSupport,
        MotifContext::KingdomEnriched,
        MotifContext::NaturalLeaning,
        MotifContext::SyntheticLeaning,
        MotifContext::DecorationOnly,
        MotifContext::NoneDetected,
    ] {
        assert_eq!(ctx.to_string(), ctx.as_str(), "Display matches as_str");
        assert_eq!(
            ctx.is_detected(),
            ctx != MotifContext::NoneDetected,
            "is_detected matches NoneDetected"
        );
    }

    for cat in [
        VerdictCategory::Likely,
        VerdictCategory::Neutral,
        VerdictCategory::Caution,
        VerdictCategory::Skeptical,
        VerdictCategory::Fishy,
    ] {
        assert!(
            cat.css_class().starts_with("verdict-"),
            "CSS class prefix for {cat:?}"
        );
    }

    for st in [CheckStatus::Pass, CheckStatus::Warn, CheckStatus::Fail] {
        assert_eq!(st.to_string(), st.as_str(), "CheckStatus Display");
    }
    for sc in [
        SourceClass::Natural,
        SourceClass::Synthetic,
        SourceClass::Unknown,
    ] {
        assert_eq!(sc.to_string(), sc.as_str(), "SourceClass Display");
        assert_eq!(
            SourceClass::from(sc.as_str()),
            sc,
            "SourceClass str roundtrip"
        );
    }
    let Some(key) = InChIKey14::from_inchikey("RYYVLZVUVIJVGH-UHFFFAOYSA-N") else {
        unreachable!("valid 27-char InChIKey must parse");
    };
    assert_eq!(key.to_string(), "RYYVLZVUVIJVGH", "InChIKey14 Display");

    let rank = |cat: VerdictCategory| -> u8 {
        match cat {
            VerdictCategory::Fishy => 0,
            VerdictCategory::Caution => 1,
            VerdictCategory::Skeptical => 2,
            VerdictCategory::Neutral => 3,
            VerdictCategory::Likely => 4,
        }
    };
    let mut prev = 0u8;
    for score in [-3.0, -2.0, -0.5, 0.5, 1.5, 2.5] {
        let v = classify_np_evidence(&ev(score, false, false, 0, counts(2, 1, 0, 4, 0, 1)));
        let r = rank(v.category);
        assert!(
            r >= prev,
            "verdict rank regressed at score={score}: {prev} -> {r}"
        );
        prev = r;
    }
}
