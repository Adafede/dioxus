// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the cxsmiles-yoga project

//! Example SMILES-list fixtures for the demo.
//!
//! Six curated example sets covering every construct family the core
//! supports: positional isomers (`m:` blocks, including a double-`m` acetyl
//! case) and variable-length repeats (`Sg:n:` blocks), plus a best-effort
//! constitutional-isomer set. Each entry is `(id, label, multiline SMILES)`.

/// An example input set: `(id, label, one-SMILES-per-line text)`.
#[derive(Debug)]
pub struct ExampleSet {
    pub id: &'static str,
    pub label: &'static str,
    pub smiles: &'static str,
}

macro_rules! example_sets {
    ($( ($id:literal, $label:literal, $smiles:literal) ),* $(,)?) => {
        /// The six demo example sets.
        pub const EXAMPLE_SETS: &[ExampleSet] = &[
            $( ExampleSet { id: $id, label: $label, smiles: $smiles }, )*
        ];

        const EXAMPLE_ITEMS: &[ui::prelude::SegmentedControlItem] = &[
            $( ui::prelude::SegmentedControlItem { label: $label, value: $id }, )*
        ];
    };
}

example_sets! {
    (
        "biphenyl_cl",
        "Cl on a biphenyl (m: block)",
        "Clc1ccccc1-c2ccccc2
Clc1cccc(-c2ccccc2)c1
Clc1ccc(-c2ccccc2)cc1"
    ),
    (
        "omf_moving",
        "Methoxy on a triol (m: block)",
        "COc1c(O)cc(O)cc1
Oc1c(OC)cc(O)cc1
Oc1c(O)cc(OC)cc1"
    ),
    (
        "acetyl",
        "Acetate on a biphenyl (double m: block)",
        "CC(=O)Oc1ccccc1-c2ccccc2
CC(=O)Oc1cccc(-c2ccccc2)c1
CC(=O)Oc1ccc(-c2ccccc2)cc1"
    ),
    (
        "pfas",
        "PFAS -CF2- repeats (Sg:n:)",
        "OC(=O)C(F)(F)C(F)F
OC(=O)C(F)(F)C(F)(F)C(F)F
OC(=O)C(F)(F)C(F)(F)C(F)(F)C(F)F"
    ),
    (
        "alkyl",
        "Alkyl -CH2- repeats (Sg:n:)",
        "CCCCCCC
CCCCCCCC
CCCCCCCCC"
    ),
    (
        "constitutional",
        "Constitutional isomers (best-effort)",
        "CC(C)(C)O
CC(C)CO
CC(O)CC
OCCCC
CCOCC
COCCC"
    ),
}

/// Look up an example set by `id`.
#[must_use]
pub fn example_smiles(id: &str) -> Option<&'static str> {
    EXAMPLE_SETS.iter().find(|e| e.id == id).map(|e| e.smiles)
}

/// The `SegmentedControl` items for the example picker.
#[must_use]
pub const fn example_items() -> &'static [ui::prelude::SegmentedControlItem] {
    EXAMPLE_ITEMS
}
