// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the lipid-selecto-rs project

/// Sphingolipids (SP), from the `LipidMaps` LMSD dataset.
///
/// One family per file: the corpus is 204 entries and the flat version
/// was a single 1001-line constant.
#[cfg(target_arch = "wasm32")]
pub(crate) const LIPIDS: &[(&str, &str, &str)] = &[
    (
        "Cer(m18:1(4E)/16:0)",
        "[C@](C)([H])(NC(CCCCCCCCCCCCCCC)=O)[C@]([H])(O)/C=C/CCCCCCCCCCCCC",
        "Cer(m18:1(4E)/16:0)",
    ),
    (
        "Cer(d16:1/24:0)",
        "[C@](CO)([H])(NC(CCCCCCCCCCCCCCCCCCCCCCC)=O)[C@]([H])(O)/C=C/CCCCCCCCCCC",
        "Cer(d16:1/24:0)",
    ),
    (
        "Cer(d18:1/35:0(35OH))",
        "[C@](CO)([H])(NC(CCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCO)=O)[C@]([H])(O)/C=C/CCCCCCCCCCCCC",
        "Cer(d18:1/35:0(35OH))",
    ),
    (
        "Cer(d18:2/26:0)",
        "[C@](CO)([H])(NC(CCCCCCCCCCCCCCCCCCCCCCCCC)=O)[C@]([H])(O)/C=C/CCCCCCCC/C=C\\CCC",
        "Cer(d18:2/26:0)",
    ),
    (
        "Cer(d20:2(4E,8E)(9Me)/18:1(3E)(2OH[R]))",
        "[C@](CO)([H])(NC([C@H](O)/C=C/CCCCCCCCCCCCCC)=O)[C@]([H])(O)/C=C/CC/C=C(\\C)/CCCCCCCCCCC",
        "Cer(d20:2(4E,8E)(9Me)/18:1(3E)(2OH[R]))",
    ),
    (
        "Cer(d18:2/28:0(2OH))",
        "[C@](CO)([H])(NC(C(O)CCCCCCCCCCCCCCCCCCCCCCCCCC)=O)[C@]([H])(O)/C=C/CCCCCCCC/C=C\\CCC",
        "Cer(d18:2/28:0(2OH))",
    ),
    (
        "Cer(t18:1(6OH)/29:0(29OH))",
        "[C@](CO)([H])(NC(CCCCCCCCCCCCCCCCCCCCCCCCCCCCO)=O)[C@]([H])(O)/C=C/[C@H](O)CCCCCCCCCCCC",
        "Cer(t18:1(6OH)/29:0(29OH))",
    ),
    (
        "1-O-cerotoyl-Cer(d18:1/18:0)",
        "C(OC(=O)CCCCCCCCCCCCCCCCCCCCCCCCC)[C@]([H])(NC(CCCCCCCCCCCCCCCCC)=O)[C@H](O)/C=C/CCCCCCCCCCCCC",
        "1-O-cerotoyl-Cer(d18:1/18:0)",
    ),
    (
        "omega-linoleoyloxy-Cer(t18:1(6OH)/28:0)",
        "[C@](CO)([H])(NC(CCCCCCCCCCCCCCCCCCCCCCCCCCCOC(CCCCCCC/C=C\\C/C=C\\CCCCC)=O)=O)[C@]([H])(O)/C=C/[C@H](O)CCCCCCCCCCCC",
        "omega-linoleoyloxy-Cer(t18:1(6OH)/28:0)",
    ),
    (
        "omega-linoleoyloxy-Cer(d23:0/31:0)",
        "[C@](CO)([H])(NC(CCCCCCCCCCCCCCCCCCCCCCCCCCCCCCOC(CCCCCCC/C=C\\C/C=C\\CCCCC)=O)=O)[C@]([H])(O)CCCCCCCCCCCCCCCCCCCC",
        "omega-linoleoyloxy-Cer(d23:0/31:0)",
    ),
];
