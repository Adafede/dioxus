// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the lipid-selecto-rs project

/// Glycerolipids (GL), from the `LipidMaps` LMSD dataset.
///
/// One family per file: the corpus is 204 entries and the flat version
/// was a single 1001-line constant.
#[cfg(target_arch = "wasm32")]
pub(crate) const LIPIDS: &[(&str, &str, &str)] = &[
    (
        "TG 16:0/16:0/16:0",
        "C(OC(=O)CCCCCCCCCCCCCCC)[C@]([H])(OC(CCCCCCCCCCCCCCC)=O)COC(CCCCCCCCCCCCCCC)=O",
        "TG 16:0/16:0/16:0",
    ),
    (
        "TG 16:0/18:3(9Z,12Z,15Z)/22:1(13Z) [iso6]",
        "C(OC(=O)CCCCCCCCCCC/C=C\\CCCCCCCC)[C@]([H])(OC(CCCCCCC/C=C\\C/C=C\\C/C=C\\CC)=O)COC(CCCCCCCCCCCCCCC)=O",
        "TG 16:0/18:3(9Z,12Z,15Z)/22:1(13Z) [iso6]",
    ),
    (
        "TG 18:1(9Z)/18:1(9Z)/22:6(4Z,7Z,10Z,13Z,16Z,19Z) [iso3]",
        "C(OC(=O)CC/C=C\\C/C=C\\C/C=C\\C/C=C\\C/C=C\\C/C=C\\CC)[C@]([H])(OC(CCCCCCC/C=C\\CCCCCCCC)=O)COC(CCCCCCC/C=C\\CCCCCCCC)=O",
        "TG 18:1(9Z)/18:1(9Z)/22:6(4Z,7Z,10Z,13Z,16Z,19Z) [iso3]",
    ),
    (
        "TG 19:0/20:4(5Z,8Z,11Z,14Z)/22:5(7Z,10Z,13Z,16Z,19Z) [iso6]",
        "C(OC(=O)CCCCC/C=C\\C/C=C\\C/C=C\\C/C=C\\C/C=C\\CC)[C@]([H])(OC(CCC/C=C\\C/C=C\\C/C=C\\C/C=C\\CCCCC)=O)COC(CCCCCCCCCCCCCCCCCC)=O",
        "TG 19:0/20:4(5Z,8Z,11Z,14Z)/22:5(7Z,10Z,13Z,16Z,19Z) [iso6]",
    ),
    (
        "TG 14:0/14:0/22:1(11Z) [iso3]",
        "C(OC(=O)CCCCCCCCC/C=C\\CCCCCCCCCC)[C@]([H])(OC(CCCCCCCCCCCCC)=O)COC(CCCCCCCCCCCCC)=O",
        "TG 14:0/14:0/22:1(11Z) [iso3]",
    ),
    (
        "TG 12:0/18:1(9Z)/20:3(8Z,11Z,14Z) [iso6]",
        "C(OC(=O)CCCCCC/C=C\\C/C=C\\C/C=C\\CCCCC)[C@]([H])(OC(CCCCCCC/C=C\\CCCCCCCC)=O)COC(CCCCCCCCCCC)=O",
        "TG 12:0/18:1(9Z)/20:3(8Z,11Z,14Z) [iso6]",
    ),
    (
        "TG 14:0/14:1(9Z)/22:6(4Z,7Z,10Z,13Z,16Z,19Z) [iso6]",
        "C(OC(=O)CC/C=C\\C/C=C\\C/C=C\\C/C=C\\C/C=C\\C/C=C\\CC)[C@]([H])(OC(CCCCCCC/C=C\\CCCC)=O)COC(CCCCCCCCCCCCC)=O",
        "TG 14:0/14:1(9Z)/22:6(4Z,7Z,10Z,13Z,16Z,19Z) [iso6]",
    ),
    (
        "TG 14:1(9Z)/19:0/20:0 [iso6]",
        "C(OC(=O)CCCCCCCCCCCCCCCCCCC)[C@]([H])(OC(CCCCCCCCCCCCCCCCCC)=O)COC(CCCCCCC/C=C\\CCCC)=O",
        "TG 14:1(9Z)/19:0/20:0 [iso6]",
    ),
    (
        "TG 15:1(9Z)/18:3(6Z,9Z,12Z)/22:6(4Z,7Z,10Z,13Z,16Z,19Z) [iso6]",
        "C(OC(=O)CC/C=C\\C/C=C\\C/C=C\\C/C=C\\C/C=C\\C/C=C\\CC)[C@]([H])(OC(CCCC/C=C\\C/C=C\\C/C=C\\CCCCC)=O)COC(CCCCCCC/C=C\\CCCCC)=O",
        "TG 15:1(9Z)/18:3(6Z,9Z,12Z)/22:6(4Z,7Z,10Z,13Z,16Z,19Z) [iso6]",
    ),
    (
        "TG 18:1(9Z)/18:4(6Z,9Z,12Z,15Z)/20:3(8Z,11Z,14Z) [iso6]",
        "C(OC(=O)CCCCCC/C=C\\C/C=C\\C/C=C\\CCCCC)[C@]([H])(OC(CCCC/C=C\\C/C=C\\C/C=C\\C/C=C\\CC)=O)COC(CCCCCCC/C=C\\CCCCCCCC)=O",
        "TG 18:1(9Z)/18:4(6Z,9Z,12Z,15Z)/20:3(8Z,11Z,14Z) [iso6]",
    ),
];
