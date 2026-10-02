// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the lipid-selecto-rs project

/// Glycerophospholipids (GP), from the `LipidMaps` LMSD dataset.
///
/// One family per file: the corpus is 204 entries and the flat version
/// was a single 1001-line constant.
#[cfg(target_arch = "wasm32")]
pub(crate) const LIPIDS: &[(&str, &str, &str)] = &[
    (
        "PE 17:0/20:4(5Z,8Z,11Z,14Z)",
        "[C@](COP(O)(=O)OCCN)([H])(OC(CCC/C=C\\C/C=C\\C/C=C\\C/C=C\\CCCCC)=O)COC(CCCCCCCCCCCCCCCC)=O",
        "PE 17:0/20:4(5Z,8Z,11Z,14Z)",
    ),
    (
        "PE 13:0/20:3(8Z,11Z,14Z)",
        "[C@](COP(=O)(O)OCCN)([H])(OC(CCCCCC/C=C\\C/C=C\\C/C=C\\CCCCC)=O)COC(CCCCCCCCCCCC)=O",
        "PE 13:0/20:3(8Z,11Z,14Z)",
    ),
    (
        "PE 17:0/20:3(8Z,11Z,14Z)",
        "[C@](COP(=O)(O)OCCN)([H])(OC(CCCCCC/C=C\\C/C=C\\C/C=C\\CCCCC)=O)COC(CCCCCCCCCCCCCCCC)=O",
        "PE 17:0/20:3(8Z,11Z,14Z)",
    ),
    (
        "PE 18:3(9Z,12Z,15Z)/12:0",
        "[C@](COP(=O)(O)OCCN)([H])(OC(CCCCCCCCCCC)=O)COC(CCCCCCC/C=C\\C/C=C\\C/C=C\\CC)=O",
        "PE 18:3(9Z,12Z,15Z)/12:0",
    ),
    (
        "PE 20:1(11Z)/22:0",
        "[C@](COP(=O)(O)OCCN)([H])(OC(CCCCCCCCCCCCCCCCCCCCC)=O)COC(CCCCCCCCC/C=C\\CCCCCCCC)=O",
        "PE 20:1(11Z)/22:0",
    ),
    (
        "PE 22:0/17:1(9Z)",
        "[C@](COP(=O)(O)OCCN)([H])(OC(CCCCCCC/C=C\\CCCCCCC)=O)COC(CCCCCCCCCCCCCCCCCCCCC)=O",
        "PE 22:0/17:1(9Z)",
    ),
    (
        "PE 20:0/20:0",
        "[C@](COP(=O)(O)OCCN)([H])(OC(CCCCCCCCCCCCCCCCCCC)=O)COC(CCCCCCCCCCCCCCCCCCC)=O",
        "PE 20:0/20:0",
    ),
    (
        "PE O-18:0/22:1(11Z)",
        "[C@](COP(=O)(O)OCCN)([H])(OC(CCCCCCCCC/C=C\\CCCCCCCCCC)=O)COCCCCCCCCCCCCCCCCCC",
        "PE O-18:0/22:1(11Z)",
    ),
    (
        "PE P-16:0/18:1(11Z)",
        "[C@](COP(=O)(O)OCCN)([H])(OC(CCCCCCCCC/C=C\\CCCCCC)=O)CO/C=C\\CCCCCCCCCCCCCC",
        "PE P-16:0/18:1(11Z)",
    ),
    (
        "PE P-20:3(11Z,14Z,17Z)/22:5(7Z,10Z,13Z,16Z,19Z)",
        "[C@](COP(=O)(O)OCCN)([H])(OC(CCCCC/C=C\\C/C=C\\C/C=C\\C/C=C\\C/C=C\\CC)=O)CO/C=C\\CCCCCCCC/C=C\\C/C=C\\C/C=C\\CC",
        "PE P-20:3(11Z,14Z,17Z)/22:5(7Z,10Z,13Z,16Z,19Z)",
    ),
    (
        "CL(1'-[18:2(9Z,12Z)/18:2(9Z,12Z)],3'-[18:2(9Z,12Z)/18:2(9Z,12Z)])",
        "P(OC[C@]([H])(OC(CCCCCCC/C=C\\C/C=C\\CCCCC)=O)COC(CCCCCCC/C=C\\C/C=C\\CCCCC)=O)(O)(OC[C@](O)([H])COP(OC[C@]([H])(OC(CCCCCCC/C=C\\C/C=C\\CCCCC)=O)COC(CCCCCCC/C=C\\C/C=C\\CCCCC)=O)(O)=O)=O",
        "CL(1'-[18:2(9Z,12Z)/18:2(9Z,12Z)],3'-[18:2(9Z,12Z)/18:2(9Z,12Z)])",
    ),
    (
        "CL(1'-[16:0/18:2(9Z,12Z)],3'-[18:1(9Z)/20:4(5Z,8Z,11Z,14Z)])",
        "P(OC[C@]([H])(OC(CCCCCCC/C=C\\C/C=C\\CCCCC)=O)COC(CCCCCCCCCCCCCCC)=O)(O)(OC[C@](O)([H])COP(OC[C@]([H])(OC(=O)CCC/C=C\\C/C=C\\C/C=C\\C/C=C\\CCCCC)COC(=O)CCCCCCC/C=C\\CCCCCCCC)(O)=O)=O",
        "CL(1'-[16:0/18:2(9Z,12Z)],3'-[18:1(9Z)/20:4(5Z,8Z,11Z,14Z)])",
    ),
    (
        "CL(1'-[18:0/18:0],3'-[16:0/20:0])",
        "P(OC[C@]([H])(OC(CCCCCCCCCCCCCCCCC)=O)COC(CCCCCCCCCCCCCCCCC)=O)(O)(OC[C@](O)([H])COP(OC[C@]([H])(OC(=O)CCCCCCCCCCCCCCCCCCC)COC(=O)CCCCCCCCCCCCCCC)(O)=O)=O",
        "CL(1'-[18:0/18:0],3'-[16:0/20:0])",
    ),
    (
        "CL(1'-[18:0/20:0],3'-[20:0/18:2(9Z,12Z)])",
        "P(OC[C@]([H])(OC(CCCCCCCCCCCCCCCCCCC)=O)COC(CCCCCCCCCCCCCCCCC)=O)(O)(OC[C@](O)([H])COP(OC[C@]([H])(OC(=O)CCCCCCC/C=C\\C/C=C\\CCCCC)COC(=O)CCCCCCCCCCCCCCCCCCC)(O)=O)=O",
        "CL(1'-[18:0/20:0],3'-[20:0/18:2(9Z,12Z)])",
    ),
    (
        "CL(1'-[18:1(9Z)/18:1(9Z)],3'-[18:1(9Z)/18:1(9Z)])",
        "P(OC[C@]([H])(OC(CCCCCCC/C=C\\CCCCCCCC)=O)COC(CCCCCCC/C=C\\CCCCCCCC)=O)(O)(OC[C@](O)([H])COP(OC[C@]([H])(OC(=O)CCCCCCC/C=C\\CCCCCCCC)COC(=O)CCCCCCC/C=C\\CCCCCCCC)(O)=O)=O",
        "CL(1'-[18:1(9Z)/18:1(9Z)],3'-[18:1(9Z)/18:1(9Z)])",
    ),
    (
        "CL(1'-[18:2(9Z,12Z)/16:0],3'-[16:0/18:0])",
        "P(OC[C@]([H])(OC(CCCCCCCCCCCCCCC)=O)COC(CCCCCCC/C=C\\C/C=C\\CCCCC)=O)(O)(OC[C@](O)([H])COP(OC[C@]([H])(OC(=O)CCCCCCCCCCCCCCCCC)COC(=O)CCCCCCCCCCCCCCC)(O)=O)=O",
        "CL(1'-[18:2(9Z,12Z)/16:0],3'-[16:0/18:0])",
    ),
    (
        "CL(1'-[18:2(9Z,12Z)/18:2(9Z,12Z)],3'-[20:0/18:0])",
        "P(OC[C@]([H])(OC(CCCCCCC/C=C\\C/C=C\\CCCCC)=O)COC(CCCCCCC/C=C\\C/C=C\\CCCCC)=O)(O)(OC[C@](O)([H])COP(OC[C@]([H])(OC(=O)CCCCCCCCCCCCCCCCC)COC(=O)CCCCCCCCCCCCCCCCCCC)(O)=O)=O",
        "CL(1'-[18:2(9Z,12Z)/18:2(9Z,12Z)],3'-[20:0/18:0])",
    ),
    (
        "CL(1'-[20:0/18:0],3'-[18:1(9Z)/16:0])",
        "P(OC[C@]([H])(OC(CCCCCCCCCCCCCCCCC)=O)COC(CCCCCCCCCCCCCCCCCCC)=O)(O)(OC[C@](O)([H])COP(OC[C@]([H])(OC(=O)CCCCCCCCCCCCCCC)COC(=O)CCCCCCC/C=C\\CCCCCCCC)(O)=O)=O",
        "CL(1'-[20:0/18:0],3'-[18:1(9Z)/16:0])",
    ),
    (
        "CL(1'-[20:0/20:0],3'-[20:4(5Z,8Z,11Z,14Z)/20:4(5Z,8Z,11Z,14Z)])",
        "P(OC[C@]([H])(OC(CCCCCCCCCCCCCCCCCCC)=O)COC(CCCCCCCCCCCCCCCCCCC)=O)(O)(OC[C@](O)([H])COP(OC[C@]([H])(OC(=O)CCC/C=C\\C/C=C\\C/C=C\\C/C=C\\CCCCC)COC(=O)CCC/C=C\\C/C=C\\C/C=C\\C/C=C\\CCCCC)(O)=O)=O",
        "CL(1'-[20:0/20:0],3'-[20:4(5Z,8Z,11Z,14Z)/20:4(5Z,8Z,11Z,14Z)])",
    ),
    (
        "CL(1'-[20:4(5Z,8Z,11Z,14Z)/18:1(9Z)],3'-[18:2(9Z,12Z)/20:0])",
        "P(OC[C@]([H])(OC(CCCCCCC/C=C\\CCCCCCCC)=O)COC(CCC/C=C\\C/C=C\\C/C=C\\C/C=C\\CCCCC)=O)(O)(OC[C@](O)([H])COP(OC[C@]([H])(OC(=O)CCCCCCCCCCCCCCCCCCC)COC(=O)CCCCCCC/C=C\\C/C=C\\CCCCC)(O)=O)=O",
        "CL(1'-[20:4(5Z,8Z,11Z,14Z)/18:1(9Z)],3'-[18:2(9Z,12Z)/20:0])",
    ),
    (
        "PS 12:0/13:0",
        "C(O)(=O)[C@@]([H])(N)COP(OC[C@]([H])(OC(CCCCCCCCCCCC)=O)COC(CCCCCCCCCCC)=O)(=O)O",
        "PS 12:0/13:0",
    ),
    (
        "PS 15:0/17:1(9Z)",
        "C(O)(=O)[C@@]([H])(N)COP(OC[C@]([H])(OC(CCCCCCC/C=C\\CCCCCCC)=O)COC(CCCCCCCCCCCCCC)=O)(=O)O",
        "PS 15:0/17:1(9Z)",
    ),
    (
        "PS 17:1(9Z)/20:1(11Z)",
        "C(O)(=O)[C@@]([H])(N)COP(OC[C@]([H])(OC(CCCCCCCCC/C=C\\CCCCCCCC)=O)COC(CCCCCCC/C=C\\CCCCCCC)=O)(=O)O",
        "PS 17:1(9Z)/20:1(11Z)",
    ),
    (
        "PS 18:3(6Z,9Z,12Z)/21:0",
        "C(O)(=O)[C@@]([H])(N)COP(OC[C@]([H])(OC(CCCCCCCCCCCCCCCCCCCC)=O)COC(CCCC/C=C\\C/C=C\\C/C=C\\CCCCC)=O)(=O)O",
        "PS 18:3(6Z,9Z,12Z)/21:0",
    ),
    (
        "PS 20:0/17:0",
        "C(O)(=O)[C@@]([H])(N)COP(OC[C@]([H])(OC(CCCCCCCCCCCCCCCC)=O)COC(CCCCCCCCCCCCCCCCCCC)=O)(=O)O",
        "PS 20:0/17:0",
    ),
    (
        "PS 20:4(5Z,8Z,11Z,14Z)/20:1(11Z)",
        "C(O)(=O)[C@@]([H])(N)COP(OC[C@]([H])(OC(CCCCCCCCC/C=C\\CCCCCCCC)=O)COC(CCC/C=C\\C/C=C\\C/C=C\\C/C=C\\CCCCC)=O)(=O)O",
        "PS 20:4(5Z,8Z,11Z,14Z)/20:1(11Z)",
    ),
    (
        "PS 22:2(13Z,16Z)/16:1(9Z)",
        "C(O)(=O)[C@@]([H])(N)COP(OC[C@]([H])(OC(CCCCCCC/C=C\\CCCCCC)=O)COC(CCCCCCCCCCC/C=C\\C/C=C\\CCCCC)=O)(=O)O",
        "PS 22:2(13Z,16Z)/16:1(9Z)",
    ),
    (
        "PS 18:0/16:1(9Z)",
        "C(O)(=O)[C@@]([H])(N)COP(OC[C@]([H])(OC(CCCCCCC/C=C\\CCCCCC)=O)COC(CCCCCCCCCCCCCCCCC)=O)(=O)O",
        "PS 18:0/16:1(9Z)",
    ),
    (
        "PS O-18:0/13:0",
        "C(O)(=O)[C@@]([H])(N)COP(OC[C@]([H])(OC(CCCCCCCCCCCC)=O)COCCCCCCCCCCCCCCCCCC)(=O)O",
        "PS O-18:0/13:0",
    ),
    (
        "PS P-18:0/20:5(5Z,8Z,11Z,14Z,17Z)",
        "C(O)(=O)[C@@]([H])(N)COP(OC[C@]([H])(OC(CCC/C=C\\C/C=C\\C/C=C\\C/C=C\\C/C=C\\CC)=O)CO/C=C\\CCCCCCCCCCCCCCCC)(=O)O",
        "PS P-18:0/20:5(5Z,8Z,11Z,14Z,17Z)",
    ),
    (
        "PC 12:0/13:0",
        "[C@](COP(=O)([O-])OCC[N+](C)(C)C)([H])(OC(CCCCCCCCCCCC)=O)COC(CCCCCCCCCCC)=O",
        "PC 12:0/13:0",
    ),
    (
        "PC 18:0/20:1(14Z)",
        "[C@](COP(=O)([O-])OCC[N+](C)(C)C)([H])(OC(CCCCCCCCCCCC/C=C\\CCCCC)=O)COC(CCCCCCCCCCCCCCCCC)=O",
        "PC 18:0/20:1(14Z)",
    ),
    (
        "PC 24:0/24:0",
        "[C@](COP(=O)([O-])OCC[N+](C)(C)C)([H])(OC(CCCCCCCCCCCCCCCCCCCCCCC)=O)COC(CCCCCCCCCCCCCCCCCCCCCCC)=O",
        "PC 24:0/24:0",
    ),
    (
        "PC 16:0/13:0",
        "[C@](COP(=O)([O-])OCC[N+](C)(C)C)([H])(OC(CCCCCCCCCCCC)=O)COC(CCCCCCCCCCCCCCC)=O",
        "PC 16:0/13:0",
    ),
    (
        "PC 18:3(6Z,9Z,12Z)/18:0",
        "[C@](COP(=O)([O-])OCC[N+](C)(C)C)([H])(OC(CCCCCCCCCCCCCCCCC)=O)COC(CCCC/C=C\\C/C=C\\C/C=C\\CCCCC)=O",
        "PC 18:3(6Z,9Z,12Z)/18:0",
    ),
    (
        "PC 20:2(11Z,14Z)/15:1(9Z)",
        "[C@](COP(=O)([O-])OCC[N+](C)(C)C)([H])(OC(CCCCCCC/C=C\\CCCCC)=O)COC(CCCCCCCCC/C=C\\C/C=C\\CCCCC)=O",
        "PC 20:2(11Z,14Z)/15:1(9Z)",
    ),
    (
        "PC 22:1(11Z)/21:0",
        "[C@](COP(=O)([O-])OCC[N+](C)(C)C)([H])(OC(CCCCCCCCCCCCCCCCCCCC)=O)COC(CCCCCCCCC/C=C\\CCCCCCCCCC)=O",
        "PC 22:1(11Z)/21:0",
    ),
    (
        "PC 22:1(13Z)/18:1(9Z)",
        "[C@](COP(=O)([O-])OCC[N+](C)(C)C)([H])(OC(CCCCCCC/C=C\\CCCCCCCC)=O)COC(CCCCCCCCCCC/C=C\\CCCCCCCC)=O",
        "PC 22:1(13Z)/18:1(9Z)",
    ),
    (
        "PC O-18:0/20:4(8Z,11Z,14Z,17Z)",
        "[C@](COP(=O)([O-])OCC[N+](C)(C)C)([H])(OC(CCCCCC/C=C\\C/C=C\\C/C=C\\C/C=C\\CC)=O)COCCCCCCCCCCCCCCCCCC",
        "PC O-18:0/20:4(8Z,11Z,14Z,17Z)",
    ),
    (
        "PC P-18:0/22:4(7Z,10Z,13Z,16Z)",
        "[C@](COP(=O)([O-])OCC[N+](C)(C)C)([H])(OC(CCCCC/C=C\\C/C=C\\C/C=C\\C/C=C\\CCCCC)=O)CO/C=C\\CCCCCCCCCCCCCCCC",
        "PC P-18:0/22:4(7Z,10Z,13Z,16Z)",
    ),
    (
        "PG 12:0/13:0",
        "[C@](COP(=O)(O)OCC(O)CO)([H])(OC(CCCCCCCCCCCC)=O)COC(CCCCCCCCCCC)=O",
        "PG 12:0/13:0",
    ),
    (
        "PG 15:0/22:0",
        "[H][C@](O)(CO)COP(OC[C@]([H])(OC(CCCCCCCCCCCCCCCCCCCCC)=O)COC(CCCCCCCCCCCCCC)=O)(=O)O",
        "PG 15:0/22:0",
    ),
    (
        "PG 17:2(9Z,12Z)/18:3(6Z,9Z,12Z)",
        "[H][C@](O)(CO)COP(OC[C@]([H])(OC(CCCC/C=C\\C/C=C\\C/C=C\\CCCCC)=O)COC(CCCCCCC/C=C\\C/C=C\\CCCC)=O)(=O)O",
        "PG 17:2(9Z,12Z)/18:3(6Z,9Z,12Z)",
    ),
    (
        "PG 18:4(6Z,9Z,12Z,15Z)/13:0",
        "[H][C@](O)(CO)COP(OC[C@]([H])(OC(CCCCCCCCCCCC)=O)COC(CCCC/C=C\\C/C=C\\C/C=C\\C/C=C\\CC)=O)(=O)O",
        "PG 18:4(6Z,9Z,12Z,15Z)/13:0",
    ),
    (
        "PG 20:2(11Z,14Z)/14:1(9Z)",
        "[H][C@](O)(CO)COP(OC[C@]([H])(OC(CCCCCCC/C=C\\CCCC)=O)COC(CCCCCCCCC/C=C\\C/C=C\\CCCCC)=O)(=O)O",
        "PG 20:2(11Z,14Z)/14:1(9Z)",
    ),
    (
        "PG 21:0/20:4(5Z,8Z,11Z,14Z)",
        "[H][C@](O)(CO)COP(OC[C@]([H])(OC(CCC/C=C\\C/C=C\\C/C=C\\C/C=C\\CCCCC)=O)COC(CCCCCCCCCCCCCCCCCCCC)=O)(=O)O",
        "PG 21:0/20:4(5Z,8Z,11Z,14Z)",
    ),
    (
        "PG 22:6(4Z,7Z,10Z,13Z,16Z,19Z)/18:3(9Z,12Z,15Z)",
        "[H][C@](O)(CO)COP(OC[C@]([H])(OC(CCCCCCC/C=C\\C/C=C\\C/C=C\\CC)=O)COC(CC/C=C\\C/C=C\\C/C=C\\C/C=C\\C/C=C\\C/C=C\\CC)=O)(=O)O",
        "PG 22:6(4Z,7Z,10Z,13Z,16Z,19Z)/18:3(9Z,12Z,15Z)",
    ),
    (
        "PG 17:0/20:0",
        "[H][C@](O)(CO)COP(OC[C@]([H])(OC(CCCCCCCCCCCCCCCCCCC)=O)COC(CCCCCCCCCCCCCCCC)=O)(=O)O",
        "PG 17:0/20:0",
    ),
    (
        "PG P-16:0/15:1(9Z)",
        "[C@]([H])(OC(CCCCCCC/C=C\\CCCCC)=O)(COP(=O)(O)OC[C@@]([H])(O)CO)CO/C=C\\CCCCCCCCCCCCCC",
        "PG P-16:0/15:1(9Z)",
    ),
    (
        "LBPA 16:1(9Z)/18:1(9Z)",
        "O(P(OC[C@](OC(CCCCCCC/C=C\\CCCCCCCC)=O)([H])CO)(O)=O)C[C@@]([H])(OC(CCCCCCC/C=C\\CCCCCC)=O)CO",
        "LBPA 16:1(9Z)/18:1(9Z)",
    ),
    (
        "PI 16:0/18:1(9Z)",
        "[C@]([H])(OC(CCCCCCC/C=C\\CCCCCCCC)=O)(COP(=O)(O)O[C@H]1[C@H](O)[C@@H](O)[C@H](O)[C@@H](O)[C@H]1O)COC(CCCCCCCCCCCCCCC)=O",
        "PI 16:0/18:1(9Z)",
    ),
    (
        "PI 15:0/20:5(5Z,8Z,11Z,14Z,17Z)",
        "[C@]([H])(OC(CCC/C=C\\C/C=C\\C/C=C\\C/C=C\\C/C=C\\CC)=O)(COP(=O)(O)O[C@H]1[C@H](O)[C@@H](O)[C@H](O)[C@@H](O)[C@H]1O)COC(CCCCCCCCCCCCCC)=O",
        "PI 15:0/20:5(5Z,8Z,11Z,14Z,17Z)",
    ),
    (
        "PI 17:1(9Z)/22:2(13Z,16Z)",
        "[C@]([H])(OC(CCCCCCCCCCC/C=C\\C/C=C\\CCCCC)=O)(COP(=O)(O)O[C@H]1[C@H](O)[C@@H](O)[C@H](O)[C@@H](O)[C@H]1O)COC(CCCCCCC/C=C\\CCCCCCC)=O",
        "PI 17:1(9Z)/22:2(13Z,16Z)",
    ),
    (
        "PI 18:3(6Z,9Z,12Z)/22:4(7Z,10Z,13Z,16Z)",
        "[C@]([H])(OC(CCCCC/C=C\\C/C=C\\C/C=C\\C/C=C\\CCCCC)=O)(COP(=O)(O)O[C@H]1[C@H](O)[C@@H](O)[C@H](O)[C@@H](O)[C@H]1O)COC(CCCC/C=C\\C/C=C\\C/C=C\\CCCCC)=O",
        "PI 18:3(6Z,9Z,12Z)/22:4(7Z,10Z,13Z,16Z)",
    ),
    (
        "PI 20:0/17:0",
        "[C@]([H])(OC(CCCCCCCCCCCCCCCC)=O)(COP(=O)(O)O[C@H]1[C@H](O)[C@@H](O)[C@H](O)[C@@H](O)[C@H]1O)COC(CCCCCCCCCCCCCCCCCCC)=O",
        "PI 20:0/17:0",
    ),
    (
        "PI 20:4(5Z,8Z,11Z,14Z)/18:4(6Z,9Z,12Z,15Z)",
        "[C@]([H])(OC(CCCC/C=C\\C/C=C\\C/C=C\\C/C=C\\CC)=O)(COP(=O)(O)O[C@H]1[C@H](O)[C@@H](O)[C@H](O)[C@@H](O)[C@H]1O)COC(CCC/C=C\\C/C=C\\C/C=C\\C/C=C\\CCCCC)=O",
        "PI 20:4(5Z,8Z,11Z,14Z)/18:4(6Z,9Z,12Z,15Z)",
    ),
    (
        "PI 22:1(11Z)/22:4(7Z,10Z,13Z,16Z)",
        "[C@]([H])(OC(CCCCC/C=C\\C/C=C\\C/C=C\\C/C=C\\CCCCC)=O)(COP(=O)(O)O[C@H]1[C@H](O)[C@@H](O)[C@H](O)[C@@H](O)[C@H]1O)COC(CCCCCCCCC/C=C\\CCCCCCCCCC)=O",
        "PI 22:1(11Z)/22:4(7Z,10Z,13Z,16Z)",
    ),
    (
        "PI 18:3(9Z,12Z,15Z)/18:1(9Z)",
        "[C@]([H])(OC(CCCCCCC/C=C\\CCCCCCCC)=O)(COP(=O)(O)O[C@H]1[C@H](O)[C@@H](O)[C@H](O)[C@@H](O)[C@H]1O)COC(CCCCCCC/C=C\\C/C=C\\C/C=C\\CC)=O",
        "PI 18:3(9Z,12Z,15Z)/18:1(9Z)",
    ),
    (
        "PI 10:0/16:0",
        "[C@]([H])(OC(CCCCCCCCCCCCCCC)=O)(COP(=O)(O)O[C@H]1[C@H](O)[C@@H](O)[C@H](O)[C@@H](O)[C@H]1O)COC(CCCCCCCCC)=O",
        "PI 10:0/16:0",
    ),
    (
        "PI P-16:0/19:0",
        "[C@]([H])(OC(CCCCCCCCCCCCCCCCCC)=O)(COP(=O)(O)O[C@H]1[C@H](O)[C@@H](O)[C@H](O)[C@@H](O)[C@H]1O)CO/C=C\\CCCCCCCCCCCCCC",
        "PI P-16:0/19:0",
    ),
    (
        "PA 12:0/13:0",
        "[C@](COP(=O)(O)O)([H])(OC(CCCCCCCCCCCC)=O)COC(CCCCCCCCCCC)=O",
        "PA 12:0/13:0",
    ),
    (
        "PA 15:0/12:0",
        "[C@](COP(=O)(O)O)([H])(OC(CCCCCCCCCCC)=O)COC(CCCCCCCCCCCCCC)=O",
        "PA 15:0/12:0",
    ),
    (
        "PA 17:1(9Z)/18:2(9Z,12Z)",
        "[C@](COP(=O)(O)O)([H])(OC(CCCCCCC/C=C\\C/C=C\\CCCCC)=O)COC(CCCCCCC/C=C\\CCCCCCC)=O",
        "PA 17:1(9Z)/18:2(9Z,12Z)",
    ),
    (
        "PA 18:3(6Z,9Z,12Z)/18:3(9Z,12Z,15Z)",
        "[C@](COP(=O)(O)O)([H])(OC(CCCCCCC/C=C\\C/C=C\\C/C=C\\CC)=O)COC(CCCC/C=C\\C/C=C\\C/C=C\\CCCCC)=O",
        "PA 18:3(6Z,9Z,12Z)/18:3(9Z,12Z,15Z)",
    ),
    (
        "PA 19:1(9Z)/20:3(8Z,11Z,14Z)",
        "[C@](COP(=O)(O)O)([H])(OC(CCCCCC/C=C\\C/C=C\\C/C=C\\CCCCC)=O)COC(CCCCCCC/C=C\\CCCCCCCCC)=O",
        "PA 19:1(9Z)/20:3(8Z,11Z,14Z)",
    ),
    (
        "PA 20:4(5Z,8Z,11Z,14Z)/14:1(9Z)",
        "[C@](COP(=O)(O)O)([H])(OC(CCCCCCC/C=C\\CCCC)=O)COC(CCC/C=C\\C/C=C\\C/C=C\\C/C=C\\CCCCC)=O",
        "PA 20:4(5Z,8Z,11Z,14Z)/14:1(9Z)",
    ),
    (
        "PA 22:1(11Z)/19:1(9Z)",
        "[C@](COP(=O)(O)O)([H])(OC(CCCCCCC/C=C\\CCCCCCCCC)=O)COC(CCCCCCCCC/C=C\\CCCCCCCCCC)=O",
        "PA 22:1(11Z)/19:1(9Z)",
    ),
    (
        "PA 20:0/16:0",
        "[C@](COP(=O)(O)O)([H])(OC(CCCCCCCCCCCCCCC)=O)COC(CCCCCCCCCCCCCCCCCCC)=O",
        "PA 20:0/16:0",
    ),
    (
        "PA 15:0/18:1(9Z)-d7",
        "[2H]C(C(CCCCC/C=C\\CCCCCCCC(O[C@@](COC(=O)CCCCCCCCCCCCCC)([H])COP(O)(O)=O)=O)([2H])[2H])(C([2H])([2H])[2H])[2H]",
        "PA 15:0/18:1(9Z)-d7",
    ),
    (
        "PA P-18:0/12:0",
        "[C@](COP(=O)(O)O)([H])(OC(CCCCCCCCCCC)=O)CO/C=C\\CCCCCCCCCCCCCCCC",
        "PA P-18:0/12:0",
    ),
];
