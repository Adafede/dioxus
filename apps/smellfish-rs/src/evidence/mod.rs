// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the smellfish-rs project

//! Evidence assessment for natural-product (NP) originality.
//!
//! **Primary evidence** — the Ertl NP-likeness score — is computed in the
//! rdkit.js bridge using the open-data fragment-contribution model from:
//!
//! > Ertl, P., Roggo, S., & Schuffenhauer, A. (2008). "Natural Product-likeness
//! > Score and Its Application for Prioritization of Compound Libraries."
//! > *J. Chem. Inf. Model.*, 48, 68–74. DOI: 10.1021/ci700286x
//!
//! The open-source, open-data implementation and model file (`np_model.bin`)
//! are from:
//!
//! > Jayaseelan, K. V., Moreno, P., Truszkowski, A., Ertl, P., & Steinbeck, C.
//! > (2012). "Natural product-likeness score revisited: an open-source, open-data
//! > implementation." *BMC Bioinformatics*, 13, 106. DOI: 10.1186/1471-2105-13-106
//!
//! The model was trained on ~50 000 natural products (open databases) vs.
//! ~1 M drug-like molecules from ZINC.  Each Morgan-fingerprint (radius 2)
//! bit carries a log-probability-ratio contribution; the score is the sum of
//! contributions divided by the heavy-atom count, with log-compression beyond
//! ±4 to prevent score explosion.
//!
//! **Secondary evidence** — structural observations — uses only values that a
//! practising natural-product chemist would recognise as NP-typical, drawn from
//! the same Ertl papers and from the "escaping the *flatland*" literature
//! (Ertl 2003, *J. Am. Chem. Soc.* 125, 10353; Ertl & Schuppenhauer 2011).

pub(crate) mod assessment;
pub(crate) mod chemist;
mod ring_family;
pub(crate) mod verdict;

#[allow(clippy::module_name_repetitions)]
pub use assessment::{EvidenceAssessment, EvidenceInputs, assess_np_evidence, np_likeness_label};
#[cfg(target_arch = "wasm32")]
pub use chemist::run_checks;
pub use chemist::{is_known_np_motif, is_scaffold_motif};
pub use ring_family::classify_ring_family;
#[allow(clippy::module_name_repetitions)]
pub use verdict::EvidenceSignals;
#[cfg(target_arch = "wasm32")]
pub use verdict::row_verdict;

#[cfg(test)]
mod tests {
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
        assert!(!assessment.ring_family.is_empty());
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
        assert!(!assessment.ring_family.is_empty());
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
}
