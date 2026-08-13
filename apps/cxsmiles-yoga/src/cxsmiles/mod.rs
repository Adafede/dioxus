// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! CX-SMILES generation core (UI-free, unit-testable).
//!
//! Pipeline (`generate_cxsmiles`): parse → cluster → maximum-common
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
//! - [`types`] — public result types.
//! - [`parse`] — SMILES input parsing and ECFP4/Tanimoto clustering.
//! - [`graph`] — Molecule↔Query conversion and graph matching primitives.
//! - [`positional`] — `m:` (positional equivalence) construction.
//! - [`repeating`] — `Sg:n:` (repeating) construction.
//! - [`roundtrip`] — enumeration & round-trip coverage used for confidence.

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::cast_possible_wrap,
    clippy::cast_lossless
)]

use chematic::core::Molecule;

// Public re-exports (unchanged from the single-file layout): the writer cannot
// emit `*`, so callers reach for `canonical_smiles`/`parse`/`write` via this
// module too.
pub use chematic::smiles::{canonical_smiles, parse, write};

pub(crate) mod graph;
pub(crate) mod parse;
pub(crate) mod positional;
pub(crate) mod repeating;
pub(crate) mod roundtrip;
pub(crate) mod types;

pub use types::{
    Confidence, Construct, Coverage, CxError, CxResult, CxResult_, FloatingPart, RepeatUnit,
};

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
/// # Panics
///
/// Panics if clustering discards every group (unreachable: at least one
/// cluster is always retained).
///
/// # Errors
///
/// Returns a [`CxError`] if any input fails to parse as SMILES.
#[allow(clippy::module_name_repetitions)]
pub fn generate_cxsmiles(smiles: &[String]) -> CxResult_ {
    let mols = parse_list(smiles)?;
    if mols.is_empty() {
        return Err(CxError("no parseable SMILES in input".into()));
    }
    if mols.len() == 1 {
        let mol = &mols[0];
        let smi = canonical_smiles(mol);
        return Ok(CxResult {
            cx_smiles: smi.clone(),
            base_smiles: smi.clone(),
            construct: Construct::BestEffort,
            scaffold_smiles: smi.clone(),
            floating: Vec::new(),
            repeating: None,
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
    let group: Vec<Molecule> = if clusters.iter().map(Vec::len).max().unwrap_or(0) >= 2 {
        clusters.into_iter().max_by_key(Vec::len).unwrap()
    } else {
        mols
    };

    let counts: Vec<usize> = group.iter().map(Molecule::atom_count).collect();
    if counts.iter().all(|c| *c == counts[0]) {
        build_positional(&group)
    } else {
        build_repeating(&group)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn lines(s: &str) -> Vec<String> {
        s.lines()
            .map(|l| l.trim().to_string())
            .filter(|l| !l.is_empty())
            .collect()
    }

    fn assert_roundtrip(result: &CxResult, group: &[Molecule]) {
        let canon: Vec<String> = group.iter().map(canonical_smiles).collect();
        for c in &canon {
            assert!(
                result.enumerated.iter().any(|e| e == c),
                "input canonical SMILES {c} is NOT covered by round-trip; enumerated={:?}",
                result.enumerated
            );
        }
    }

    #[test]
    fn parse_canonical_write_roundtrip() {
        let m = parse("Clc1ccccc1-c2ccccc2").unwrap();
        assert_eq!(canonical_smiles(&m), "c1ccc(-c2ccccc2)c(c1)Cl");
        assert_eq!(write(&m), "Clc1ccccc1-c2ccccc2");
    }

    #[test]
    fn positional_biphenyl_cl() {
        // Reference (RDKit-indexed): C1=CC=CC=C1C2=...Cl* |m:13:0.2.3|
        let input = "Clc1ccccc1-c2ccccc2\nClc1cccc(-c2ccccc2)c1\nClc1ccc(-c2ccccc2)cc1";
        let mols = parse_list(&lines(input)).unwrap();
        let r = generate_cxsmiles(&lines(input)).unwrap();
        assert_eq!(r.construct, Construct::Positional);
        assert_roundtrip(&r, &mols);
        assert!(r.confidence.clean);
        assert_eq!(r.scaffold_smiles, "c1ccccc1-c2ccccc2");
        assert_eq!(r.floating.len(), 1);
        assert_eq!(r.floating[0].equiv.len(), 3);
        assert!(r.cx_smiles.contains("m:"));
        assert_eq!(r.cx_smiles.matches("m:").count(), 1);
    }

    #[test]
    fn positional_omf_moving() {
        // Reference: OC1=C(O)C=C(O)C=C1.C* |m:10:0.3.6|
        let input = "COc1c(O)cc(O)cc1\nOc1c(OC)cc(O)cc1\nOc1c(O)cc(OC)cc1";
        let mols = parse_list(&lines(input)).unwrap();
        let r = generate_cxsmiles(&lines(input)).unwrap();
        assert_eq!(r.construct, Construct::Positional);
        assert_roundtrip(&r, &mols);
        assert!(r.confidence.clean);
        assert_eq!(r.floating.len(), 1);
        assert_eq!(r.floating[0].equiv.len(), 3);
    }

    #[test]
    fn positional_acetyl_double_m() {
        // Acetate (-OCOCH3) moves on a triol scaffold; emitted as two m: blocks
        // (O* + C(=O)(C)*) — the "double m: block variant".
        let input =
            "CC(=O)Oc1ccccc1-c2ccccc2\nCC(=O)Oc1cccc(-c2ccccc2)c1\nCC(=O)Oc1ccc(-c2ccccc2)cc1";
        let mols = parse_list(&lines(input)).unwrap();
        let r = generate_cxsmiles(&lines(input)).unwrap();
        assert_eq!(r.construct, Construct::Positional);
        assert_roundtrip(&r, &mols);
        assert!(r.confidence.clean);
        assert_eq!(r.floating.len(), 2, "floating={:?}", r.floating);
        let o_star = r.floating.iter().find(|f| !f.split).expect("O* group");
        let acetyl = r
            .floating
            .iter()
            .find(|f| f.split)
            .expect("C(=O)(C)* group");
        assert!(o_star.fragment_smiles.contains('O') && o_star.fragment_smiles.contains('*'));
        assert!(
            acetyl.fragment_smiles.contains('C')
                && acetyl.fragment_smiles.contains('O')
                && acetyl.fragment_smiles.contains('*')
        );
        assert_eq!(o_star.equiv.len(), 3);
        assert_eq!(r.cx_smiles.matches("m:").count(), 2);
    }

    #[test]
    fn repeating_pfas() {
        // Reference: OC(=O)C(F)(F)C(F)F |Sg:n:3,4,5:n:ht|
        let input =
            "OC(=O)C(F)(F)C(F)F\nOC(=O)C(F)(F)C(F)(F)C(F)F\nOC(=O)C(F)(F)C(F)(F)C(F)(F)C(F)F";
        let mols = parse_list(&lines(input)).unwrap();
        let r = generate_cxsmiles(&lines(input)).unwrap();
        assert_eq!(r.construct, Construct::Repeating);
        assert_roundtrip(&r, &mols);
        assert!(r.confidence.clean);
        assert_eq!(r.scaffold_smiles, "OC(=O)C(F)(F)C(F)F");
        assert_eq!(r.cx_smiles, "OC(=O)C(F)(F)C(F)F |Sg:n:3,4,5:n:ht|");
    }

    #[test]
    fn repeating_alkyl() {
        // Reference: CCCCCCC |Sg:n:3:n:ht|
        let input = "CCCCCCC\nCCCCCCCC\nCCCCCCCCC";
        let mols = parse_list(&lines(input)).unwrap();
        let r = generate_cxsmiles(&lines(input)).unwrap();
        assert_eq!(r.construct, Construct::Repeating);
        assert_roundtrip(&r, &mols);
        assert_eq!(r.scaffold_smiles, "CCCCCCC");
        assert!(r.cx_smiles.contains("Sg:n:3:n:ht"));
    }

    #[test]
    fn best_effort_constitutional_isomers() {
        // Six constitutional isomers of C4H10O with no clean shared moving
        // group -> best-effort, sub-100% round-trip coverage.
        let input = "\
CC(C)(C)O
CC(C)CO
CC(O)CC
OCCCC
CCOCC
COCCC";
        let r = generate_cxsmiles(&lines(input)).unwrap();
        assert!(
            r.confidence.coverage.fraction() < 1.0,
            "expected sub-100% coverage, got {:?}; cx={}",
            r.confidence,
            r.cx_smiles
        );
    }
}
