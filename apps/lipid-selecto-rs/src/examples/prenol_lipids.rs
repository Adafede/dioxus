// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the lipid-selecto-rs project

/// Prenol Lipids (PR), from the `LipidMaps` LMSD dataset.
///
/// One family per file: the corpus is 204 entries and the flat version
/// was a single 1001-line constant.
#[cfg(target_arch = "wasm32")]
pub(crate) const LIPIDS: &[(&str, &str, &str)] = &[
    (
        "Juvenile Hormone II",
        "C(=O)(OC)/C=C(\\C)/CC/C=C(\\C)/CC[C@H]1O[C@]1(C)CC",
        "Juvenile Hormone II",
    ),
    (
        "(+)-alpha-thujene",
        "C1C=C([C@]2([H])C[C@]12C(C)C)C",
        "(+)-alpha-thujene",
    ),
    (
        "alpha-Cubebene",
        "[C@]12([H])C3(CC=C([C@@]31[H])C)[C@H](C)CC[C@H]2C(C)C",
        "alpha-Cubebene",
    ),
    (
        "Axerophthene",
        "C1C(C)(C)C(/C=C/C(/C)=C/C=C/C(/C)=C/C)=C(C)CC1",
        "Axerophthene",
    ),
    (
        "Lagaspholone B",
        "C12[C@@]([H])(O)[C@](C)(O)[C@@]3([H])[C@@]4([H])C(C)(C)[C@@]4([H])CCC(=C)[C@]3([H])C=1C(=O)[C@@](O)(C)C2",
        "Lagaspholone B",
    ),
    (
        "3-Epikatonic acid",
        "C1[C@@]2(C)[C@@]([H])(CC[C@]3(C)[C@]2([H])CC=C2[C@@]3(C)CC[C@]3(C)[C@@]2([H])C[C@](C)(C(=O)O)CC3)C(C)(C)[C@@H](O)C1",
        "3-Epikatonic acid",
    ),
    (
        "Caloxanthin sulfate",
        "C1(=C(C)C[C@@H](O)[C@H](O)C1(C)C)/C=C/C(/C)=C/C=C/C(/C)=C/C=C/C=C(\\C)/C=C/C=C(\\C)/C=C/C1=C(C)C[C@@H](OS(O)(=O)=O)CC1(C)C",
        "Caloxanthin sulfate",
    ),
    (
        "11',12'-Dihydrospheroidene",
        "C(=C(/C)\\C=C\\CC(C)(OC)C)/C=C/C(/C)=C/C=C/C(/C)=C/C=C/C=C(\\C)/CC/C=C(\\C)/CC/C=C(\\C)/CC/C=C(\\C)/C",
        "11',12'-Dihydrospheroidene",
    ),
    (
        "Cryptoxanthin glucoside",
        "C1C(C)(C)C(/C=C/C(/C)=C/C=C/C(/C)=C/C=C/C=C(\\C)/C=C/C=C(\\C)/C=C/C2=C(C)CCCC2(C)C)=C(C)C[C@H]1O[C@H]1[C@H](O)[C@@H](O)[C@H](O)[C@@H](CO)O1",
        "Cryptoxanthin glucoside",
    ),
    (
        "Anhydrolutein I",
        "C1C(C)(C)C(/C=C/C(/C)=C/C=C/C(/C)=C/C=C/C=C(\\C)/C=C/C=C(\\C)/C=C/[C@H]2C(=C)C=CCC2(C)C)=C(C)C[C@H]1O",
        "Anhydrolutein I",
    ),
];
