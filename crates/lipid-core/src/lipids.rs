// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the lipid-selecto-rs project

//! Broad lipid classification from SMILES structure or molecular formula.

mod classify;
mod types;

pub use classify::{classify_acyclic_molecule, classify_formula, is_acyclic};
#[cfg(test)]
pub(crate) use classify::{
    classify_smiles, classify_spectrum, counts_from_molecule, formula_counts,
};
pub use types::{ElementCounts, LipidClassification};
