// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the cxsmiles-yoga project

//! CX-SMILES generation core (UI-free, unit-testable).
//!
//! Pipeline (`generate`): parse → cluster → maximum-common
//! substructure → diff & classify → serialise → round-trip confidence.
//!
//! `chematic::cx` only handles atom-level CX fields (labels/props/radicals/
//! zero-bonds/wavy bonds) — **not** `m:` (positional equivalence) or
//! `Sg:n:` (repeating units). Both are hand-rolled here, along with their
//! expansion logic used for round-tripping.
//!
//! Note: chematic's SMILES *writer* does not emit `*` for wildcard atoms (it
//! writes the placeholder element `C`), so the CX base string is assembled by
//! hand: `scaffold_smiles` followed by `.[*]<frag>` for each floating group.
//! This keeps atom indices fully under our control.
//!
//! This module is split by responsibility:
//! - `types` — public result types.
//! - `parse` — SMILES input parsing and ECFP4/Tanimoto clustering.
//! - `graph` — Molecule↔Query conversion and graph matching primitives.
//! - `positional` — `m:` (positional equivalence) construction.
//! - `repeating` — `Sg:n:` (repeating) construction.
//! - `roundtrip` — enumeration & round-trip coverage used for confidence.

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::cast_possible_wrap,
    clippy::cast_lossless
)]

use chematic::core::Molecule;

// Crate-internal re-exports: the writer cannot emit `*`, so in-crate callers
// reach for `canonical_smiles`/`parse`/`write` via this module.
pub use chematic::smiles::canonical_smiles;

pub mod graph;
pub mod parse;
pub mod positional;
pub mod repeating;
pub mod roundtrip;
pub mod types;

pub(crate) use types::{Confidence, Construct, Coverage, CxError, CxResult, CxResult_};

// Internal helpers consumed by the orchestrator below. `parse` is also a
// re-exported *value* (the chematic parser); there is no namespace clash because
// the submodule lives in the type namespace.
use parse::{cluster, parse_list};
use positional::build_positional;
use repeating::build_repeating;

/// Minimum ECFP4/Tanimoto similarity for two structures to share a coherent
/// candidate group (single-linkage). Biphenyl variants pairwise reach ≥0.37.
const CLUSTER_TANIMOTO: f64 = 0.3;

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Generate a CX-SMILES from a list of related SMILES (one per entry).
///
/// # Errors
///
/// Returns a [`CxError`] if any input fails to parse as SMILES.
pub(crate) fn generate(smiles: &[String]) -> CxResult_ {
    let mols = parse_list(smiles)?;
    if mols.is_empty() {
        return Err(CxError("no parseable SMILES in input".into()));
    }
    if let [mol] = mols.as_slice() {
        let smi = canonical_smiles(mol);
        return Ok(CxResult {
            cx_smiles: smi.clone(),
            construct: Construct::BestEffort,
            scaffold_smiles: smi.clone(),
            floating: Vec::new(),
            confidence: Confidence {
                coverage: Coverage {
                    covered: 1,
                    total: 1,
                },
                clean: true,
            },
            enumerated: vec![smi],
        });
    }

    let clusters = cluster(&mols, CLUSTER_TANIMOTO);
    // Largest cluster, but only if it actually contains ≥2 molecules (a
    // singleton "group" means no shared scaffold to collapse); otherwise fall
    // back to treating every input as its own group. Non-panicking: `unwrap_or`
    // handles both the empty-cluster and singleton cases.
    let group: Vec<Molecule> = clusters
        .into_iter()
        .max_by_key(Vec::len)
        .filter(|c| c.len() >= 2)
        .unwrap_or(mols);

    // Positional construction needs every input in the group to have the same
    // atom count; anything else is a repeating unit instead.
    let uniform = group
        .first()
        .is_some_and(|first| group.iter().all(|m| m.atom_count() == first.atom_count()));
    if uniform {
        build_positional(&group)
    } else {
        build_repeating(&group)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
#[expect(clippy::unwrap_used)] // tests unwrap fixtures to fail-fast on parse/generate errors
#[expect(clippy::expect_used)]
// tests expect known fixtures (O*, C(=O)(C)*) to be present
// `r.floating[0]` below reads the group a `m:` block was just emitted for, and
// `generate` fails outright when it emits none, so an out-of-range read is a
// regression in `generate`, not something this test can usefully report itself.
#[allow(clippy::indexing_slicing)]
mod tests;
