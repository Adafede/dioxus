// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the lipid-selecto-rs project

/// Saccharolipids (SL), from the `LipidMaps` LMSD dataset.
///
/// One family per file: the corpus is 204 entries and the flat version
/// was a single 1001-line constant.
#[cfg(target_arch = "wasm32")]
pub(crate) const LIPIDS: &[(&str, &str, &str)] = &[
    (
        "DAT(16:0/21:0(2Me[R],3OH[R],4Me[S],6Me[S]))",
        "CCCCCCCCCCCCCCC[C@H](C)C[C@H](C)[C@H]([C@@H](C)C(=O)O[C@H]1[C@@H]([C@@H](CO)O[C@@H]([C@@H]1OC(=O)CCCCCCCCCCCCCCC)O[C@@H]1[C@@H]([C@H]([C@@H]([C@@H](CO)O1)O)O)O)O)O",
        "DAT(16:0/21:0(2Me[R],3OH[R],4Me[S],6Me[S]))",
    ),
    (
        "PAT16(22:1(2E)(2Me,4Me[S],6Me[S])/24:0(2Me[R],3OH[R],4Me[S],6Me[S])/25:1(2E)(2Me,4Me[S],6Me[S])/22:1(2E)(2Me,4Me[S],6Me[S]))",
        "CCCCCCCCCCCCCCCCCCC[C@H](C)C[C@H](C)/C=C(\\C)/C(=O)O[C@@H]1[C@H]([C@@H]([C@@H](CO)O[C@@H]1O[C@@H]1[C@@H]([C@H]([C@@H]([C@@H](COC(=O)[C@H](C)[C@@H]([C@@H](C)C[C@@H](C)CCCCCCCCCCCCCCCCCC)O)O1)O)OC(=O)/C(=C/[C@@H](C)C[C@@H](C)CCCCCCCCCCCCCCCC)/C)OC(=O)CCCCCCCCCCCCCCC)OC(=O)/C(=C/[C@@H](C)C[C@@H](C)CCCCCCCCCCCCCCCC)/C)O",
        "PAT16(22:1(2E)(2Me,4Me[S],6Me[S])/24:0(2Me[R],3OH[R],4Me[S],6Me[S])/25:1(2E)(2Me,4Me[S],6Me[S])/22:1(2E)(2Me,4Me[S],6Me[S]))",
    ),
    (
        "PAT16(24:1(2E)(2Me,4Me[S],6Me[S])/24:0(2Me[R],3OH[R],4Me[S],6Me[S])/26:1(2E)(2Me,4Me[S],6Me[S])/24:1(2E)(2Me,4Me[S],6Me[S]))",
        "CCCCCCCCCCCCCCCCCCCC[C@H](C)C[C@H](C)/C=C(\\C)/C(=O)O[C@@H]1[C@H]([C@@H]([C@@H](CO)O[C@@H]1O[C@@H]1[C@@H]([C@H]([C@@H]([C@@H](COC(=O)[C@H](C)[C@@H]([C@@H](C)C[C@@H](C)CCCCCCCCCCCCCCCCCC)O)O1)O)OC(=O)/C(=C/[C@@H](C)C[C@@H](C)CCCCCCCCCCCCCCCCCC)/C)OC(=O)CCCCCCCCCCCCCCC)OC(=O)/C(=C/[C@@H](C)C[C@@H](C)CCCCCCCCCCCCCCCCCC)/C)O",
        "PAT16(24:1(2E)(2Me,4Me[S],6Me[S])/24:0(2Me[R],3OH[R],4Me[S],6Me[S])/26:1(2E)(2Me,4Me[S],6Me[S])/24:1(2E)(2Me,4Me[S],6Me[S]))",
    ),
    (
        "PAT16(25:1(2E)(2Me,4Me[S],6Me[S])/24:0(2Me[R],3OH[R],4Me[S],6Me[S])/24:0(2Me[R],3OH[R],4Me[S],6Me[S])/25:1(2E)(2Me,4Me[S],6Me[S]))",
        "CCCCCCCCCCCCCCCCCCC[C@H](C)C[C@H](C)/C=C(\\C)/C(=O)O[C@@H]1[C@@H](CO)O[C@@H]([C@@H]([C@H]1O)OC(=O)[C@H](C)[C@@H]([C@@H](C)C[C@@H](C)CCCCCCCCCCCCCCCCCC)O)O[C@@H]1[C@@H]([C@H]([C@@H]([C@@H](COC(=O)[C@H](C)[C@@H]([C@@H](C)C[C@@H](C)CCCCCCCCCCCCCCCCCC)O)O1)O)OC(=O)/C(=C/[C@@H](C)C[C@@H](C)CCCCCCCCCCCCCCCCCCC)/C)OC(=O)CCCCCCCCCCCCCCC",
        "PAT16(25:1(2E)(2Me,4Me[S],6Me[S])/24:0(2Me[R],3OH[R],4Me[S],6Me[S])/24:0(2Me[R],3OH[R],4Me[S],6Me[S])/25:1(2E)(2Me,4Me[S],6Me[S]))",
    ),
    (
        "PAT16(24:0(2Me[R],3OH[R],4Me[S],6Me[S])/22:1(2E)(2Me,4Me[S],6Me[S])/22:1(2E)(2Me,4Me[S],6Me[S])/26:1(2E)(2Me,4Me[S],6Me[S]))",
        "CCCCCCCCCCCCCCCCCCCC[C@H](C)C[C@H](C)/C=C(\\C)/C(=O)O[C@@H]1[C@@H](CO)O[C@@H]([C@@H]([C@H]1O)OC(=O)/C(=C/[C@@H](C)C[C@@H](C)CCCCCCCCCCCCCCCC)/C)O[C@@H]1[C@@H]([C@H]([C@@H]([C@@H](COC(=O)/C(=C/[C@@H](C)C[C@@H](C)CCCCCCCCCCCCCCCC)/C)O1)O)OC(=O)[C@H](C)[C@@H]([C@@H](C)C[C@@H](C)CCCCCCCCCCCCCCCCCC)O)OC(=O)CCCCCCCCCCCCCCC",
        "PAT16(24:0(2Me[R],3OH[R],4Me[S],6Me[S])/22:1(2E)(2Me,4Me[S],6Me[S])/22:1(2E)(2Me,4Me[S],6Me[S])/26:1(2E)(2Me,4Me[S],6Me[S]))",
    ),
    (
        "PAT18(22:1(2E)(2Me,4Me[S],6Me[S])/22:1(2E)(2Me,4Me[S],6Me[S])/24:1(2E)(2Me,4Me[S],6Me[S])/24:0(2Me[R],3OH[R],4Me[S],6Me[S]))",
        "CCCCCCCCCCCCCCCCCC[C@H](C)C[C@H](C)/C=C(\\C)/C(=O)O[C@@H]1[C@H]([C@@H]([C@@H](CO)O[C@@H]1O[C@@H]1[C@@H]([C@H]([C@@H]([C@@H](COC(=O)/C(=C/[C@@H](C)C[C@@H](C)CCCCCCCCCCCCCCCC)/C)O1)O)OC(=O)/C(=C/[C@@H](C)C[C@@H](C)CCCCCCCCCCCCCCCC)/C)OC(=O)CCCCCCCCCCCCCCCCC)OC(=O)[C@H](C)[C@@H]([C@@H](C)C[C@@H](C)CCCCCCCCCCCCCCCCCC)O)O",
        "PAT18(22:1(2E)(2Me,4Me[S],6Me[S])/22:1(2E)(2Me,4Me[S],6Me[S])/24:1(2E)(2Me,4Me[S],6Me[S])/24:0(2Me[R],3OH[R],4Me[S],6Me[S]))",
    ),
    (
        "PAT18(24:1(2E)(2Me,4Me[S],6Me[S])/22:1(2E)(2Me,4Me[S],6Me[S])/26:1(2E)(2Me,4Me[S],6Me[S])/22:1(2E)(2Me,4Me[S],6Me[S]))",
        "CCCCCCCCCCCCCCCCCC[C@H](C)C[C@H](C)/C=C(\\C)/C(=O)O[C@@H]1[C@@H](CO)O[C@@H]([C@@H]([C@H]1O)OC(=O)[C@H](C)[C@@H]([C@@H](C)C[C@@H](C)CCCCCCCCCCCCCCCCCC)O)O[C@@H]1[C@@H]([C@H]([C@@H]([C@@H](COC(=O)/C(=C/[C@@H](C)C[C@@H](C)CCCCCCCCCCCCCCCC)/C)O1)O)OC(=O)/C(=C/[C@@H](C)C[C@@H](C)CCCCCCCCCCCCCCCCCC)/C)OC(=O)CCCCCCCCCCCCCCCCC",
        "PAT18(24:1(2E)(2Me,4Me[S],6Me[S])/22:1(2E)(2Me,4Me[S],6Me[S])/26:1(2E)(2Me,4Me[S],6Me[S])/22:1(2E)(2Me,4Me[S],6Me[S]))",
    ),
    (
        "PAT18(25:1(2E)(2Me,4Me[S],6Me[S])/22:1(2E)(2Me,4Me[S],6Me[S])/24:0(2Me[R],3OH[R],4Me[S],6Me[S])/24:1(2E)(2Me,4Me[S],6Me[S]))",
        "CCCCCCCCCCCCCCCCCCC[C@H](C)C[C@H](C)/C=C(\\C)/C(=O)O[C@H]1[C@@H]([C@@H](COC(=O)/C(=C/[C@@H](C)C[C@@H](C)CCCCCCCCCCCCCCCC)/C)O[C@@H]([C@@H]1OC(=O)CCCCCCCCCCCCCCCCC)O[C@@H]1[C@@H]([C@H]([C@@H]([C@@H](CO)O1)OC(=O)/C(=C/[C@@H](C)C[C@@H](C)CCCCCCCCCCCCCCCCCC)/C)O)OC(=O)[C@H](C)[C@@H]([C@@H](C)C[C@@H](C)CCCCCCCCCCCCCCCCCC)O)O",
        "PAT18(25:1(2E)(2Me,4Me[S],6Me[S])/22:1(2E)(2Me,4Me[S],6Me[S])/24:0(2Me[R],3OH[R],4Me[S],6Me[S])/24:1(2E)(2Me,4Me[S],6Me[S]))",
    ),
    (
        "PAT18(26:1(2E)(2Me,4Me[S],6Me[S])/24:1(2E)(2Me,4Me[S],6Me[S])/22:1(2E)(2Me,4Me[S],6Me[S])/25:1(2E)(2Me,4Me[S],6Me[S]))",
        "CCCCCCCCCCCCCCCCCCCC[C@H](C)C[C@H](C)/C=C(\\C)/C(=O)O[C@@H]1[C@@H](CO)O[C@@H]([C@@H]([C@H]1O)OC(=O)/C(=C/[C@@H](C)C[C@@H](C)CCCCCCCCCCCCCCCCCC)/C)O[C@@H]1[C@@H]([C@H]([C@@H]([C@@H](COC(=O)/C(=C/[C@@H](C)C[C@@H](C)CCCCCCCCCCCCCCCCCC)/C)O1)O)OC(=O)/C(=C/[C@@H](C)C[C@@H](C)CCCCCCCCCCCCCCCCCCCC)/C)OC(=O)CCCCCCCCCCCCCCCCC",
        "PAT18(26:1(2E)(2Me,4Me[S],6Me[S])/24:1(2E)(2Me,4Me[S],6Me[S])/22:1(2E)(2Me,4Me[S],6Me[S])/25:1(2E)(2Me,4Me[S],6Me[S]))",
    ),
    (
        "PAT18(24:0(2Me[R],3OH[R],4Me[S],6Me[S])/24:1(2E)(2Me,4Me[S],6Me[S])/24:1(2E)(2Me,4Me[S],6Me[S])/26:1(2E)(2Me,4Me[S],6Me[S]))",
        "CCCCCCCCCCCCCCCCCCCC[C@H](C)C[C@H](C)/C=C(\\C)/C(=O)O[C@@H]1[C@@H](CO)O[C@@H]([C@@H]([C@H]1O)OC(=O)/C(=C/[C@@H](C)C[C@@H](C)CCCCCCCCCCCCCCCCCC)/C)O[C@@H]1[C@@H]([C@H]([C@@H]([C@@H](COC(=O)/C(=C/[C@@H](C)C[C@@H](C)CCCCCCCCCCCCCCCCCC)/C)O1)O)OC(=O)[C@H](C)[C@@H]([C@@H](C)C[C@@H](C)CCCCCCCCCCCCCCCCCC)O)OC(=O)CCCCCCCCCCCCCCCCC",
        "PAT18(24:0(2Me[R],3OH[R],4Me[S],6Me[S])/24:1(2E)(2Me,4Me[S],6Me[S])/24:1(2E)(2Me,4Me[S],6Me[S])/26:1(2E)(2Me,4Me[S],6Me[S]))",
    ),
];
