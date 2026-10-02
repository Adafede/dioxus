// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the lipid-selecto-rs project

/// Sterol Lipids (ST), from the `LipidMaps` LMSD dataset.
///
/// One family per file: the corpus is 204 entries and the flat version
/// was a single 1001-line constant.
#[cfg(target_arch = "wasm32")]
pub(crate) const LIPIDS: &[(&str, &str, &str)] = &[
    (
        "Cholestane skeleton",
        "[C@]12(CCC3CCCC[C@]3(C)[C@@]1([H])CC[C@]1(C)[C@@]([H])([C@@](C)([H])CCCC(C)C)CC[C@@]21[H])[H]",
        "Cholestane skeleton",
    ),
    (
        "Penasterol",
        "C12CC[C@@]3([H])C(C)(C)[C@@H](O)CC[C@]3(C)C=1CC[C@]1(C)[C@@]([H])([C@@](C)([H])CC/C=C(\\C)/C)CC[C@@]21C(O)=O",
        "Penasterol",
    ),
    (
        "Menellsteroid E",
        "[C@H]1(O)[C@]2(C)[C@@]3([H])[C@@H](O)C[C@]4(C)[C@@]([H])([C@H](C)CCCC(C)C)CC[C@@]4([H])[C@]3([H])CC(=O)[C@@]2(O)C[C@@H](O)C1",
        "Menellsteroid E",
    ),
    (
        "Cholesteryl 11-hydroperoxy-eicosatetraenoate",
        "C(=C/C/C=C\\CC(OO)/C=C/C=C\\CCCCC)/CCCC(O[C@@H]1CC2=CC[C@@]3([H])[C@]4([H])CC[C@]([H])([C@]([H])(C)CCCC(C)C)[C@@]4(C)CC[C@]3([H])[C@@]2(C)CC1)=O",
        "Cholesteryl 11-hydroperoxy-eicosatetraenoate",
    ),
    (
        "4alpha,14alpha-Dimethyl-5alpha-Ergesta-7,9(11),24(28)-trien-3beta-ol",
        "C1[C@]2(C)C3=CC[C@]4(C)[C@@]([H])([C@]([H])(C)CCC(=C)C(C)C)CC[C@@]4(C)C3=CC[C@@]2([H])[C@H](C)[C@@H](O)C1",
        "4alpha,14alpha-Dimethyl-5alpha-Ergesta-7,9(11),24(28)-trien-3beta-ol",
    ),
    (
        "Certonardosterol K",
        "C1[C@]2(C)[C@@]3([H])CC[C@]4(C)[C@@]([H])([C@]([H])(C)/C=C/C(C)CCO)C[C@@H](O)[C@@]4([H])[C@]3(O)C[C@H](O)[C@@]2([H])C(O)[C@@H](O)C1",
        "Certonardosterol K",
    ),
    (
        "Strongylosterol",
        "C1[C@]2(C)[C@@]3([H])CC[C@]4(C)[C@@]([H])([C@]([H])(C)CC[C@@H](CC)C(CC)=C)CC[C@@]4([H])[C@]3([H])CC=C2C[C@@H](O)C1",
        "Strongylosterol",
    ),
    (
        "Klyflaccisteroid D",
        "C1C[C@H](O)CC2=CC(=O)[C@@]3([H])[C@]4([H])CC[C@]([H])([C@H](C)[C@H]5C[C@]5(C)[C@H](C)C(C)C)[C@@]4(C)CC(=O)[C@]3([H])[C@@]12C",
        "Klyflaccisteroid D",
    ),
    (
        "Cimimanol F",
        "C1C=C2[C@@]3(C[C@]43CC[C@H](O[C@@H]3OC[C@@H](O)[C@H](O[C@@H]5OC[C@@H](O)[C@H](O)[C@H]5O)[C@H]3OC(=O)CC(=O)O)C(C)(C)[C@]14[H])[C@@H](O)C[C@@]1(C)[C@@]2(C)CC(=O)[C@]1([H])[C@@H](CC(=O)[C@@H]1OC1(C)C)C",
        "Cimimanol F",
    ),
    (
        "2beta-acetoxy-3,5-di-O-acetylhellebrigenin",
        "C1[C@@]2(C=O)[C@](OC(C)=O)(CC[C@]3([H])[C@]2([H])CC[C@]2(C)[C@@]([H])(C4C=CC(=O)OC=4)CC[C@]32O)C[C@@H](OC(C)=O)[C@H]1OC(C)=O",
        "2beta-acetoxy-3,5-di-O-acetylhellebrigenin",
    ),
];
