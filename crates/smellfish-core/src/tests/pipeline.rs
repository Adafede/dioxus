// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the smellfish-rs project

use crate::model::{
    EndpointStatus, Enrichment, EnrichmentOutcome, MotifContext, RawRow, SourceSummary,
    VerdictCategory, parse_rdkit_inspect_json,
};
use crate::pipeline::{RowInspection, finalize_pass2, prepare_pass1};
use std::collections::{BTreeSet, HashMap};

fn fixture_inspections() -> Vec<RowInspection> {
    let valid_json = r#"{
        "inchikey": "RYYVLZVUVIJVGH-UHFFFAOYSA-N",
        "svg": "<svg xmlns=\"http://www.w3.org/2000/svg\"></svg>",
        "motifs": [
            {"label": "Purine ring", "source_class": "natural", "kingdom": "plants", "kingdoms": ["plants"]},
            {"label": "Shared motif", "source_class": "unknown", "kingdom": "", "kingdoms": []}
        ],
        "descriptors": {"ring_count": 2.0, "aromatic_ring_count": 2.0, "aliphatic_ring_count": 0.0, "fraction_csp3": 0.25},
        "stereo_tags": [],
        "np_score": 2.4,
        "np_confidence": 0.9,
        "num_atoms": 14,
        "substituents": ["methyl", "methyl"],
        "lotus_scaffolds": []
    }"#;
    let Ok(parsed_ok) = parse_rdkit_inspect_json(valid_json) else {
        unreachable!("valid JSON fixture must parse");
    };

    let unsafe_svg_json = r#"{
        "inchikey": "AAAAAAAAAAAAAA-BBBBBBBBBB-N",
        "svg": "<svg onload=\"alert(1)\"></svg>",
        "motifs": [{"label": "Shared motif", "source_class": "unknown", "kingdom": "", "kingdoms": []}],
        "np_score": 0.3,
        "np_confidence": 0.5,
        "num_atoms": 6
    }"#;
    let Ok(parsed_second) = parse_rdkit_inspect_json(unsafe_svg_json) else {
        unreachable!("valid JSON fixture must parse");
    };

    vec![
        RowInspection {
            raw: RawRow {
                index: 1,
                label: "Caffeine".to_string(),
                smiles: "CN1C=NC2=C1C(=O)N(C(=O)N2C)C".to_string(),
            },
            outcome: Ok(parsed_ok),
        },
        RowInspection {
            raw: RawRow {
                index: 2,
                label: "UnsafeSvg".to_string(),
                smiles: "c1ccccc1".to_string(),
            },
            outcome: Ok(parsed_second),
        },
        RowInspection {
            raw: RawRow {
                index: 3,
                label: "BadSmiles".to_string(),
                smiles: "not_smiles".to_string(),
            },
            outcome: Err("parse error".to_string()),
        },
    ]
}

fn fixture_enrichment() -> EnrichmentOutcome {
    let mut lotus_map = HashMap::new();
    lotus_map.insert(
        "RYYVLZVUVIJVGH".to_string(),
        SourceSummary {
            taxa: BTreeSet::from(["Coffea arabica".to_string()]),
            compounds: BTreeSet::from(["Q60235".to_string(), "Q999999".to_string()]),
            cids: BTreeSet::new(),
            compounds_with_taxa: BTreeSet::from(["Q60235".to_string()]),
        },
    );
    EnrichmentOutcome {
        enrichment: Enrichment {
            lotus: lotus_map,
            pubchem: HashMap::new(),
        },
        endpoints: vec![EndpointStatus::default()],
        warnings: Vec::new(),
    }
}

#[test]
fn pass1_and_pass2_aggregate_motifs_sanitize_svgs_and_merge_enrichment() {
    let batch = prepare_pass1(fixture_inspections());
    assert_eq!(batch.unique_inchikeys.len(), 2);
    assert_eq!(batch.dataset_context.common_threshold, 2);

    let outcome = finalize_pass2(batch, fixture_enrichment());
    assert_eq!(outcome.unique_inchikeys, 2);
    assert_eq!(outcome.unique_substituents, 1);
    assert_eq!(outcome.rows.len(), 3);

    if let Some(first) = outcome.rows.first() {
        assert!(first.svg.is_some(), "safe SVG should survive ingest");
        assert_eq!(first.verdict.category, VerdictCategory::Likely);
        assert!(
            first
                .lotus_compounds
                .iter()
                .any(|c| c.qid == "Q60235" && c.has_taxa)
        );
        assert!(
            first
                .lotus_compounds
                .iter()
                .any(|c| c.qid == "Q999999" && !c.has_taxa)
        );
        assert!(
            first
                .evidence_notes
                .iter()
                .any(|n| n.contains("shared with dataset")),
            "notes: {:?}",
            first.evidence_notes
        );
    }
    if let Some(second) = outcome.rows.get(1) {
        assert!(
            second.svg.is_none(),
            "unsafe SVG must be stripped during Pass 1"
        );
        assert_eq!(second.motif_context, MotifContext::NoneDetected);
        assert!(!second.motif_context.is_detected());
    }
    if let Some(third) = outcome.rows.get(2) {
        assert_eq!(third.error.as_deref(), Some("parse error"));
        assert_eq!(third.verdict.category, VerdictCategory::Neutral);
        assert_eq!(third.verdict.message, "⚠ parse error");
    }
}
