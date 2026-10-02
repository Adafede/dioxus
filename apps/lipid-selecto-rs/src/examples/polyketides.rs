// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the lipid-selecto-rs project

/// Polyketides (PK), from the `LipidMaps` LMSD dataset.
///
/// One family per file: the corpus is 204 entries and the flat version
/// was a single 1001-line constant.
#[cfg(target_arch = "wasm32")]
pub(crate) const LIPIDS: &[(&str, &str, &str)] = &[
    (
        "Delphinidin",
        "C1(O)C=C2[O+]=C(C3=CC(O)=C(O)C(O)=C3)C(O)=CC2=C(O)C=1",
        "Delphinidin",
    ),
    (
        "Chamaeflavone A",
        "C1(OC)C=C2O[C@@H](C3=CC=C(OC)C=C3)[C@]([H])([C@@]3([H])[C@H](C4=CC=C(O)C=C4)OC4=CC(O)=CC(O)=C4C3=O)C(=O)C2=C(O)C=1",
        "Chamaeflavone A",
    ),
    (
        "Sophorapterocarpan A",
        "C1(O)C=CC2[C@]3([H])OC4=CC(O)=C(C/C=C(/C)\\C)C=C4[C@]3([H])COC=2C=1",
        "Sophorapterocarpan A",
    ),
    (
        "Isovitexin 7-O-galactoside-2''-O-rhamnoside",
        "C[C@@H]1O[C@H]([C@@H]([C@@H]([C@H]1O)O)O)O[C@H]1[C@@H](O[C@@H]([C@H]([C@@H]1O)O)CO)C1C(=CC2OC(=CC(C=2C=1O)=O)C1=CC=C(C=C1)O)O[C@@H]1O[C@@H]([C@@H]([C@@H]([C@H]1O)O)O)CO",
        "Isovitexin 7-O-galactoside-2''-O-rhamnoside",
    ),
    (
        "Tricetin 3',4',5'-trimethyl ether",
        "C1(O)=CC2OC(C3C=C(OC)C(OC)=C(OC)C=3)=CC(=O)C=2C(O)=C1",
        "Tricetin 3',4',5'-trimethyl ether",
    ),
    (
        "8-C-Glucosyl-5-deoxykaempferol",
        "C1(O)=C([C@H]2[C@H](O)[C@@H](O)[C@H](O)[C@@H](CO)O2)C2OC(C3C=CC(O)=CC=3)=C(O)C(=O)C=2C=C1",
        "8-C-Glucosyl-5-deoxykaempferol",
    ),
    (
        "Quercetin 3-(6''-acetylglucoside)",
        "C1(O)=CC2OC(C3C=C(O)C(O)=CC=3)=C(O[C@H]3[C@H](O)[C@@H](O)[C@H](O)[C@@H](COC(=O)C)O3)C(=O)C=2C(O)=C1",
        "Quercetin 3-(6''-acetylglucoside)",
    ),
    (
        "Quercetin 3-methyl ether 7-glucoside",
        "C1(O[C@H]2[C@H](O)[C@@H](O)[C@H](O)[C@@H](CO)O2)=CC2OC(C3C=C(O)C(O)=CC=3)=C(OC)C(=O)C=2C(O)=C1",
        "Quercetin 3-methyl ether 7-glucoside",
    ),
    (
        "Glyasperin D",
        "C1(OC)=CC2OC[C@@H](C3C(O)=CC(O)=CC=3)CC=2C(OC)=C1C/C=C(\\C)/C",
        "Glyasperin D",
    ),
    (
        "Ovaliflavanone D",
        "C1(O)=C(C/C=C(\\C)/C)C2OC(C3C=C4OCOC4=CC=3)CC(=O)C=2C=C1C/C=C(\\C)/C",
        "Ovaliflavanone D",
    ),
];
