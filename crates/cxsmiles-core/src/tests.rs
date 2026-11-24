// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the cxsmiles-yoga project

use chematic::core::Molecule;
use chematic::smiles::{canonical_smiles, parse};

use crate::types::CxResult;

mod e2e;
mod graph;
mod limits;
mod parse_tests;
mod positional_builders;
mod positional_helpers;
mod repeating_builders;
mod repeating_helpers;
mod roundtrip;

pub(super) fn mol(smiles: &str) -> Molecule {
    parse(smiles).expect("the fixtures in this module are valid SMILES")
}

pub(super) fn chain_mol(atoms: usize) -> Molecule {
    mol(&"C".repeat(atoms))
}

pub(super) fn lines(s: &str) -> Vec<String> {
    s.lines()
        .map(|l| l.trim().to_string())
        .filter(|l| !l.is_empty())
        .collect()
}

pub(super) fn assert_roundtrip(result: &CxResult, group: &[Molecule]) {
    let canon: Vec<String> = group.iter().map(canonical_smiles).collect();
    for c in &canon {
        assert!(
            result.enumerated.iter().any(|e| e == c),
            "input canonical SMILES {c} is NOT covered by round-trip; enumerated={:?}",
            result.enumerated
        );
    }
}
