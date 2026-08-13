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

/// The six demo example sets.
pub const EXAMPLE_SETS: &[ExampleSet] = &[
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
pub fn example_smiles(id: &str) -> Option<&'static str> {
    EXAMPLE_SETS.iter().find(|e| e.id == id).map(|e| e.smiles)
}

/// The `SegmentedControl` items for the example picker.
pub fn example_items() -> Vec<ui::components::SegmentedControlItem> {
    EXAMPLE_SETS
        .iter()
        .map(|e| ui::components::SegmentedControlItem {
            label: e.label.to_string(),
            value: e.id.to_string(),
        })
        .collect()
}
