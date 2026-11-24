// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the cxsmiles-yoga project

//! Example SMILES-list fixtures for the demo.
//!
//! Six curated example sets covering every construct family supported by
//! `cxsmiles-core`: positional isomers (`m:` blocks, including a double-`m`
//! acetyl case) and variable-length repeats (`Sg:n:` blocks), plus a
//! best-effort constitutional-isomer set.

/// An example input set: `(id, label, one-SMILES-per-line text)`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ExampleSet {
    /// Short stable identifier used as the segmented control value.
    pub(crate) id: &'static str,
    /// Human-readable label shown in the example picker.
    pub(crate) label: &'static str,
    /// Multiline SMILES list (one structure per line).
    pub(crate) smiles: &'static str,
}

/// The six demo example sets.
pub(crate) const EXAMPLE_SETS: [ExampleSet; 6] = [
    ExampleSet {
        id: "biphenyl_cl",
        label: "Cl on a biphenyl (m: block)",
        smiles: "Clc1ccccc1-c2ccccc2
Clc1cccc(-c2ccccc2)c1
Clc1ccc(-c2ccccc2)cc1",
    },
    ExampleSet {
        id: "omf_moving",
        label: "Methoxy on a triol (m: block)",
        smiles: "COc1c(O)cc(O)cc1
Oc1c(OC)cc(O)cc1
Oc1c(O)cc(OC)cc1",
    },
    ExampleSet {
        id: "acetyl",
        label: "Acetate on a biphenyl (double m: block)",
        smiles: "CC(=O)Oc1ccccc1-c2ccccc2
CC(=O)Oc1cccc(-c2ccccc2)c1
CC(=O)Oc1ccc(-c2ccccc2)cc1",
    },
    ExampleSet {
        id: "pfas",
        label: "PFAS -CF2- repeats (Sg:n:)",
        smiles: "OC(=O)C(F)(F)C(F)F
OC(=O)C(F)(F)C(F)(F)C(F)F
OC(=O)C(F)(F)C(F)(F)C(F)(F)C(F)F",
    },
    ExampleSet {
        id: "alkyl",
        label: "Alkyl -CH2- repeats (Sg:n:)",
        smiles: "CCCCCCC
CCCCCCCC
CCCCCCCCC",
    },
    ExampleSet {
        id: "constitutional",
        label: "Constitutional isomers (best-effort)",
        smiles: "CC(C)(C)O
CC(C)CO
CC(O)CC
OCCCC
CCOCC
COCCC",
    },
];

/// Look up an example set by `id`.
#[must_use]
pub(crate) fn example_smiles(id: &str) -> Option<&'static str> {
    EXAMPLE_SETS.iter().find(|e| e.id == id).map(|e| e.smiles)
}

#[cfg(test)]
mod tests {
    use cxsmiles_core::{ConstructKind, generate};

    use super::{EXAMPLE_SETS, example_smiles};

    #[test]
    fn every_example_set_generates_expected_construct() -> Result<(), cxsmiles_core::CxError> {
        let expected = [
            ("biphenyl_cl", ConstructKind::Positional, true),
            ("omf_moving", ConstructKind::Positional, true),
            ("acetyl", ConstructKind::Positional, true),
            ("pfas", ConstructKind::Repeating, true),
            ("alkyl", ConstructKind::Repeating, true),
            ("constitutional", ConstructKind::BestEffort, false),
        ];
        assert_eq!(EXAMPLE_SETS.len(), expected.len());
        for (set, (exp_id, exp_kind, exp_clean)) in EXAMPLE_SETS.iter().zip(expected) {
            assert_eq!(set.id, exp_id);
            assert_eq!(example_smiles(set.id), Some(set.smiles));
            let lines: Vec<String> = set.smiles.lines().map(str::to_owned).collect();
            let res = generate(&lines)?;
            assert_eq!(res.construct, exp_kind, "example {}", set.id);
            assert_eq!(res.confidence.clean, exp_clean, "example {}", set.id);
        }
        assert_eq!(example_smiles("missing"), None);
        Ok(())
    }
}
