// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the smellfish-rs project

//! Natural-product evidence framework: motif counting, ring-family classification,
//! note synthesis, and typed verdict classification.

mod assessment;
mod chemist;
mod ring_family;
mod verdict;

pub use crate::model::{Verdict, VerdictCategory};
pub use assessment::{
    EvidenceAssessment, EvidenceInputs, assess_np_evidence, assess_np_evidence_with_counts,
    np_likeness_label,
};
pub use chemist::{
    EvidenceCounts, count_evidence, is_decoration_motif, is_known_np_motif, is_scaffold_motif,
    run_checks,
};
pub use ring_family::classify_ring_family;
pub use verdict::{EvidenceSignals, classify_np_evidence, row_verdict, row_verdict_with_counts};
