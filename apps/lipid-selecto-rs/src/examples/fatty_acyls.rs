// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the lipid-selecto-rs project

/// Fatty Acyls (FA), from the `LipidMaps` LMSD dataset.
///
/// One family per file: the corpus is 204 entries and the flat version
/// was a single 1001-line constant.
#[cfg(target_arch = "wasm32")]
pub(crate) const LIPIDS: &[(&str, &str, &str)] = &[
    (
        "10Z,13Z,16Z-nonadecatrienenitrile",
        "C(#N)CCCCCCCC/C=C\\C/C=C\\C/C=C\\CC",
        "10Z,13Z,16Z-nonadecatrienenitrile",
    ),
    ("Lauronitrile", "C(CCCCCCCCCCC)#N", "Lauronitrile"),
    ("Palmitonitrile", "C(CCCCCCCCCCCCCCC)#N", "Palmitonitrile"),
    (
        "Albanitrile C",
        "N#CCCCCC#CC#CC#CCCCCCC(O)C#N",
        "Albanitrile C",
    ),
    ("Albanitrile F", "N#CCCCCC#CC#CC#CCCCCC#N", "Albanitrile F"),
    (
        "Albanitrile G",
        "N#CC(O)CCCC#CC#CC#CCCCCCC(O)C#N",
        "Albanitrile G",
    ),
    (
        "Colneleic acid",
        "OC(=O)CCCCCC/C=C/O/C=C/C=C\\CCCCC",
        "Colneleic acid",
    ),
    (
        "Etherolenic acid",
        "C(CCCCCCC/C=C\\C=C\\O/C=C/C=C\\CC)(=O)O",
        "Etherolenic acid",
    ),
    (
        "omega5(Z)-etherolenic acid",
        "C(CCCCCCC/C=C\\C=C\\O/C=C\\C=C/CC)(=O)O",
        "omega5(Z)-etherolenic acid",
    ),
    (
        "11Z-etherolenic acid",
        "C(CCCCCCC/C=C\\C=C/O/C=C/C=C\\CC)(=O)O",
        "11Z-etherolenic acid",
    ),
    (
        "Maracin A",
        "C(CC/C=C/OC#CCC/C=C/C/C=C/C/C=C\\C=C)(=O)O",
        "Maracin A",
    ),
    (
        "Montiporic acid A",
        "C(COCC#CC#CCCCCCCC)(=O)O",
        "Montiporic acid A",
    ),
    (
        "Montiporic acid D",
        "C(COCC#CC#CCCCCCCC/C=C\\C=C)(=O)O",
        "Montiporic acid D",
    ),
    (
        "(1'Z)Colnelenic acid",
        "OC(=O)CCCCCC/C=C/O/C=C\\C=C/C/C=C\\CC",
        "(1'Z)Colnelenic acid",
    ),
    ("Palmitic acid", "OC(CCCCCCCCCCCCCCC)=O", "Palmitic acid"),
    (
        "3-hydroxy-3-methyl-2-oxo-pentanoic acid",
        "C(O)(C)(CC)C(=O)C(=O)O",
        "3-hydroxy-3-methyl-2-oxo-pentanoic acid",
    ),
    (
        "13,16-docosadienoic acid",
        "C(CC/C=C/C/C=C/CCCCC)CCCCCCCCC(=O)O",
        "13,16-docosadienoic acid",
    ),
    (
        "6E-nonenoic acid",
        "C(CCCC/C=C/CC)(=O)O",
        "6E-nonenoic acid",
    ),
    (
        "4Z,7Z,10Z,13Z,16Z,19Z,22Z,25Z-octacosaoctaenoic acid",
        "C(CC/C=C\\C/C=C\\C/C=C\\C/C=C\\C/C=C\\C/C=C\\C/C=C\\C/C=C\\CC)(=O)O",
        "4Z,7Z,10Z,13Z,16Z,19Z,22Z,25Z-octacosaoctaenoic acid",
    ),
    (
        "2Z-pentadecenoic acid",
        "C(=C/CCCCCCCCCCCC)/C(=O)O",
        "2Z-pentadecenoic acid",
    ),
    (
        "4-hydroxy-undecanoic acid",
        "C(CCC(O)CCCCCCC)(=O)O",
        "4-hydroxy-undecanoic acid",
    ),
    (
        "3-caproyl propionic acid",
        "C(CCCC)C(=O)CCC(=O)O",
        "3-caproyl propionic acid",
    ),
    (
        "7,8-dichloro-hexadecanoic acid",
        "C(CCCCCC(Cl)C(Cl)CCCCCCCC)(=O)O",
        "7,8-dichloro-hexadecanoic acid",
    ),
    (
        "3,4-dimethyl-5-carboxyethyl-2-furanacrylic acid",
        "C(/C(O)=O)=C\\C1=C(C)C(C)=C(CCC(=O)O)O1",
        "3,4-dimethyl-5-carboxyethyl-2-furanacrylic acid",
    ),
    ("Enanthaldehyde", "C([H])(CCCCCC)=O", "Enanthaldehyde"),
    ("2-octenal", "CCCCC/C=C/C([H])=O", "2-octenal"),
    ("6-decenal", "CCC/C=C/CCCCC([H])=O", "6-decenal"),
    ("pentadecanal", "C(CCCCC)CCCCCCCCC([H])=O", "pentadecanal"),
    ("3Z-hexenal", "C(C/C=C\\CC)=O", "3Z-hexenal"),
    (
        "4R,8S-Dimethyldecanal",
        "C(CC[C@H](C)CCC[C@@H](C)CC)(=O)[H]",
        "4R,8S-Dimethyldecanal",
    ),
    (
        "4E,9Z-Tetradecadienal",
        "C(CC/C=C/CCC/C=C\\CCCC)(=O)[H]",
        "4E,9Z-Tetradecadienal",
    ),
    (
        "4E,6Z-Hexadecadienal",
        "C(CC/C=C/C=C\\CCCCCCCCC)(=O)[H]",
        "4E,6Z-Hexadecadienal",
    ),
    (
        "13E-Octadecenal",
        "C(CCCCCCCCCCC/C=C/CCCC)(=O)[H]",
        "13E-Octadecenal",
    ),
    (
        "cis-11-Hexadecenal",
        "O=CCCCCCCCCCC=CCCCC",
        "cis-11-Hexadecenal",
    ),
    (
        "N-linolenoyl-glutamine",
        "C(CCCCCCC/C=C\\C/C=C\\C/C=C\\CC)(=O)N[C@@]([H])(CCC(N)=O)C(O)=O",
        "N-linolenoyl-glutamine",
    ),
    (
        "(+)N-(2S-hydroxy-propyl) alpha,alpha-dimethylarachidonoyl amine",
        "C(/C/C=C\\C/C=C\\CCCCC)=C/C/C=C\\CCC(C)(C)C(=O)NC[C@@H](O)C",
        "(+)N-(2S-hydroxy-propyl) alpha,alpha-dimethylarachidonoyl amine",
    ),
    (
        "N-docosahexaenoyl GABA",
        "C(CC/C=C\\C/C=C\\C/C=C\\C/C=C\\C/C=C\\C/C=C\\CC)(=O)NCCCC(=O)O",
        "N-docosahexaenoyl GABA",
    ),
    (
        "1-(4-carboxybutanamido)-1'-(dimethylcarbamoyl)ferrocene",
        "C12[Fe]3456789(C%10C3C4C5(C(N(C)C)=O)C6%10)C(C7C18)C29NC(=O)CCCC(=O)O",
        "1-(4-carboxybutanamido)-1'-(dimethylcarbamoyl)ferrocene",
    ),
    (
        "N-(3R-(15-methyl-3-(13-methyl-tetradecenoyloxy)-hexadecanoyl)-glycyl)-L-serine",
        "C(CNC(=O)C[C@]([H])(OC(=O)CC/C=C\\CCCCCCCC(C)C)CCCCCCCCCCCC(C)C)(=O)N[C@H](CO)C(O)=O",
        "N-(3R-(15-methyl-3-(13-methyl-tetradecenoyloxy)-hexadecanoyl)-glycyl)-L-serine",
    ),
    (
        "Semiplenamide G",
        "C([C@]1(O[C@@H]1CCCCCCCCCCCCCCC)C)(=O)NC(C)COC(=O)C",
        "Semiplenamide G",
    ),
    (
        "Thalassotalic acid A",
        "N(C(CCCCCCCCC)=O)/C(/C(=O)O)=C\\C1=CC=C(C=C1)O",
        "Thalassotalic acid A",
    ),
    (
        "N-(2E,14Z-eicosanoyl)-isobutylamine",
        "C(/C=C/CCCCCCCCCC/C=C\\CCCCC)(=O)NCC(C)C",
        "N-(2E,14Z-eicosanoyl)-isobutylamine",
    ),
    (
        "N-(dodecanoyl)-homoserine lactone",
        "[C@@H]1(CCOC1=O)NC(=O)CCCCCCCCCCC",
        "N-(dodecanoyl)-homoserine lactone",
    ),
    (
        "N-(5Z,8Z,11Z,14Z-docosatetraenoyl)-EA",
        "C(/C/C=C\\C/C=C\\CCCCCCC)=C/C/C=C\\CCCC(=O)NCCO",
        "N-(5Z,8Z,11Z,14Z-docosatetraenoyl)-EA",
    ),
    (
        "Palmityl palmitate",
        "CCCCCCCCCCCCCCCC(OCCCCCCCCCCCCCCCC)=O",
        "Palmityl palmitate",
    ),
    (
        "SFE 12:1(7Z)/2:0",
        "O(C(=O)C)CCCCCC/C=C\\CCCC",
        "SFE 12:1(7Z)/2:0",
    ),
    (
        "SFE 1:0/13:0(2Me,6Me,10Me)",
        "O=C(C(C)CCCC(C)CCCC(C)CCC)OC",
        "SFE 1:0/13:0(2Me,6Me,10Me)",
    ),
    ("Allyl butyrate", "O(C(CCC)=O)CC=C", "Allyl butyrate"),
    (
        "WE 24:1(17Z)/18:1(6Z)",
        "O=C(CCCC/C=C\\CCCCCCCCCCC)OCCCCCCCCCCCCCCCC/C=C\\CCCCCC",
        "WE 24:1(17Z)/18:1(6Z)",
    ),
    (
        "WE 18:0/18:1(10Z)",
        "O=C(CCCCCCCC/C=C\\CCCCCCC)OCCCCCCCCCCCCCCCCCC",
        "WE 18:0/18:1(10Z)",
    ),
    (
        "Type III cyanolipid 22:0 ester",
        "C(OC/C(/C)=C/C#N)(=O)CCCCCCCCCCCCCCCCCCCCC",
        "Type III cyanolipid 22:0 ester",
    ),
    (
        "Heptacosan-21-olide",
        "C1(OC(CCCCCC)CCCCCCCCCCCCCCCCCCC1)=O",
        "Heptacosan-21-olide",
    ),
    (
        "Malonyl-CoA",
        "[C@@H]1([C@H](O)[C@H](OP(=O)(O)O)[C@@H](COP(O)(=O)OP(O)(=O)OCC(C)([C@@H](O)C(=O)NCCC(=O)NCCSC(=O)CC(O)=O)C)O1)N1C=NC2C(N)=NC=NC1=2",
        "Malonyl-CoA",
    ),
    (
        "pivaloylcarnitine",
        "CC(C(OC(C[N+](C)(C)C)CC([O-])=O)=O)(C)C",
        "pivaloylcarnitine",
    ),
    (
        "Lanceolitol A1",
        "C(=O)(CCCCCCCCCCC)O[C@@H]1[C@H](O)[C@@H](O)[C@H](O)[C@@H](O)[C@@H]1O[C@H]1[C@H](O)[C@@H](O)[C@H](O)CO1",
        "Lanceolitol A1",
    ),
    (
        "1-(O-alpha-D-galactopyranosyl)-(1,3R,27S,29R)-triacontanetetrol",
        "O([C@@H]1[C@H](O)[C@@H](O)[C@@H](O)[C@@H](CO)O1)CC[C@H](O)CCCCCCCCCCCCCCCCCCCCCCC[C@H](O)C[C@H](O)C",
        "1-(O-alpha-D-galactopyranosyl)-(1,3R,27S,29R)-triacontanetetrol",
    ),
    (
        "Ethyl 3-O-beta-D-glucopyranosyl-butanoate",
        "O([C@H]1[C@H](O)[C@@H](O)[C@H](O)[C@@H](CO)O1)C(CC(=O)OCC)C",
        "Ethyl 3-O-beta-D-glucopyranosyl-butanoate",
    ),
    (
        "13-sophorosyloxydocosanoic acid",
        "C(C(CCCCCCCCC)O[C@@H]1O[C@H](CO)[C@@H](O)[C@H](O)[C@H]1O[C@@H]1O[C@H](CO)[C@@H](O)[C@H](O)[C@H]1O)CCCCCCCCCCC(=O)O",
        "13-sophorosyloxydocosanoic acid",
    ),
    (
        "Daumone-3",
        "O([C@@H](CCCC/C=C/C(=O)O)C)[C@H]1[C@H](O)C[C@@H](O)[C@H](C)O1",
        "Daumone-3",
    ),
    (
        "ascr#23",
        "O([C@@H](CCCCCCCCC/C=C/C(=O)O)C)[C@H]1[C@H](O)C[C@@H](O)[C@H](C)O1",
        "ascr#23",
    ),
    (
        "bhos#38",
        "O(CCCCCCCCCCCCCCCCCC[C@@H](O)CC(=O)O)[C@H]1[C@H](O)C[C@@H](O)[C@H](C)O1",
        "bhos#38",
    ),
    (
        "bhos#22",
        "O[C@@H]1C[C@@H](O)[C@H](C)O[C@H]1OCCCCCCCCCC[C@@H](O)CC(=O)O",
        "bhos#22",
    ),
    (
        "ibha#28",
        "O([C@@H](CCCCCCCCCCC[C@H](O)CC(=O)O)C)[C@H]1[C@H](O)C[C@@H](OC(=O)C2=CNC3C=CC=CC=32)[C@H](C)O1",
        "ibha#28",
    ),
    (
        "icos#17",
        "O(CCCCCCCC/C=C/C(=O)O)[C@H]1[C@H](O)C[C@@H](OC(=O)C2=CNC3C=CC=CC=32)[C@H](C)O1",
        "icos#17",
    ),
    ("oct-1-en-3S-ol", "C([C@H](CCCCC)O)=C", "oct-1-en-3S-ol"),
    (
        "11Z-eicosen-1-ol",
        "C(/C=C\\CCCCCCCC)CCCCCCCCCO",
        "11Z-eicosen-1-ol",
    ),
    (
        "2,4-Dimethyl-2E,4E-hexadien-1-ol",
        "OC/C(/C)=C/C(/C)=C/C",
        "2,4-Dimethyl-2E,4E-hexadien-1-ol",
    ),
    (
        "3Z,6E,8E-Dodecatrien-1-ol",
        "OCC/C=C\\C/C=C/C=C/CCC",
        "3Z,6E,8E-Dodecatrien-1-ol",
    ),
    (
        "3,7,11,15-Tetramethyl-6,10,14-hexadecatrien-1-ol",
        "OCCC(C)CC/C=C(\\C)/CC/C=C(\\C)/CC/C=C(\\C)/C",
        "3,7,11,15-Tetramethyl-6,10,14-hexadecatrien-1-ol",
    ),
    (
        "2-Methyloctan-4S-ol",
        "CC(C)C[C@@H](O)CCCC",
        "2-Methyloctan-4S-ol",
    ),
    ("4-Methyl-1-pentanol", "CC(C)CCCO", "4-Methyl-1-pentanol"),
    (
        "1-Deoxy-D-glucitol",
        "CC(C(C(C(CO)O)O)O)O",
        "1-Deoxy-D-glucitol",
    ),
    (
        "Gigantetrocinone",
        "CCCCCCCCCCCCCCC(C(O)CCC(O)C1OC(CC1)CCCCCC1OC(=O)C(C1)CC(C)=O)O",
        "Gigantetrocinone",
    ),
    (
        "Persin",
        "CC(=O)OC[C@@H](O)CC(=O)CCCCCCC/C=C/CCCCCCCC",
        "Persin",
    ),
];
