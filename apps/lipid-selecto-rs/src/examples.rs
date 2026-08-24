//! Collection of 204 example SMILES covering all LIPID MAPS classes from real data.
//! Generated from `LipidMaps` LMSD dataset.
//! Covers all 8 categories: FA, GL, GP, SP, ST, PR, SL, PK

/// Real lipid examples from `LipidMaps` LMSD dataset.
pub const EXAMPLE_LIPIDS: &[(&str, &str, &str)] = &[
    // === Fatty Acyls (FA) ===
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
    // === Glycerolipids (GL) ===
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
    // === Glycerophospholipids (GP) ===
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
    // === Sphingolipids (SP) ===
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
    // === Sterol Lipids (ST) ===
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
    // === Prenol Lipids (PR) ===
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
    // === Saccharolipids (SL) ===
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
    // === Polyketides (PK) ===
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

/// Convert example list to query format (just SMILES + description lines separated by newlines).
#[must_use]
pub fn example_smiles() -> Vec<String> {
    EXAMPLE_LIPIDS
        .iter()
        .map(|(id, smiles, _)| format!("{id}\t{smiles}"))
        .collect()
}
