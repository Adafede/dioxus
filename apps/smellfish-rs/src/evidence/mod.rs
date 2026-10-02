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

#[cfg(test)]
pub(crate) use assessment::np_likeness_label;
#[allow(clippy::module_name_repetitions)] // EvidenceXxx re-exports preserve domain naming
pub(crate) use assessment::{EvidenceInputs, assess_np_evidence};
#[cfg(target_arch = "wasm32")]
pub(crate) use chemist::run_checks;
#[cfg(test)]
pub(crate) use chemist::{is_known_np_motif, is_scaffold_motif};
#[cfg(target_arch = "wasm32")]
pub(crate) use verdict::row_verdict;

#[cfg(test)]
mod tests;
