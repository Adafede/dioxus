// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the lipid-selecto-rs project

//! LIPID MAPS Structure Database (LMSD) subclass definitions.
//!
//! Contains all 86 LMSD main classes (FA00–FA13, GL00–GL07, GP00–GP25,
//! SP00–SP08, ST01–ST05, PR01–PR04, SL01–SL05, PK00–PK15) with specific SMARTS
//! patterns. This is the primary classification system used by [`super::lmsd_classes`].

use super::{
    ChemicalClass, FA_PALETTE, GL_PALETTE, GP_PALETTE, PK_PALETTE, PR_PALETTE, SL_PALETTE,
    SP_PALETTE, ST_PALETTE,
};

/// Combined LMSD entry: family, code, human-readable description,
/// SMARTS pattern, and LMSD database count.
///
/// This is the single source of truth for [`lmsd_classes`] and the LMSD SMARTS
/// counts — no duplication.
struct LmsdEntry {
    family: &'static str,
    code: &'static str,
    description: &'static str,
    smarts: &'static str,
    count: usize,
}

/// Static LMSD subclass entries in LIPID MAPS family order.
/// Family order follows the LIPID MAPS hierarchy: FA -> GL -> GP -> SP -> ST -> PR -> SL -> PK.
const LMSD_ENTRIES: &[LmsdEntry] = &[
    LmsdEntry {
        family: "Fatty Acyls",
        code: "FA01",
        description: "Fatty Acids and Conjugates",
        smarts: r"[#6;A;!R]~[#6;A;!R]~[#6;A;!R]~[#6;A;!R]~[#6;A;!R]~[#6;A;!R][CX3](=[OX1])[OX2H1,OX1-]",
        count: 3102,
    },
    LmsdEntry {
        family: "Fatty Acyls",
        code: "FA07",
        description: "Fatty esters",
        smarts: r"[#6;A;!R]~[#6;A;!R]~[#6;A;!R]~[#6;A;!R]~[#6;A;!R][CX3](=[OX1])[OX2][#6]",
        count: 2454,
    },
    LmsdEntry {
        family: "Fatty Acyls",
        code: "FA03",
        description: "Eicosanoids",
        smarts: r"[CX3](=[OX1])[OX2H1,OX1-,OX2,NX3].*=C.*(-[H2])-[H2].[#6&r5]",
        count: 1381,
    },
    LmsdEntry {
        family: "Fatty Acyls",
        code: "FA04",
        description: "Docosanoids",
        smarts: r"[CX3](=[OX1])[OX2H1,OX1-,OX2,NX3].*=[!224*]-[#6].C=[!234*]!@[!#7]-[!#6].[$([#6]~[#8])&r5]",
        count: 1191,
    },
    LmsdEntry {
        family: "Fatty Acyls",
        code: "FA02",
        description: "Octadecanoids",
        smarts: r"[#6;A]~[#6;A]~[#6;A]~[#6;A]~[#6;A]~[#6;A]~[#6;A]~[#6;A][CX3](=[OX1])[OX2H1,OX1-]",
        count: 763,
    },
    LmsdEntry {
        family: "Fatty Acyls",
        code: "FA11",
        description: "Hydrocarbons",
        smarts: r"[CH2X4;!R][CH2X4;!R][CH2X4;!R][CH2X4;!R][CH2X4;!R][CH2X4;!R][CH3X4;!R]",
        count: 701,
    },
    LmsdEntry {
        family: "Fatty Acyls",
        code: "FA08",
        description: "Fatty amides",
        smarts: r"[#6;A;!R]~[#6;A;!R]~[#6;A;!R]~[#6;A;!R]~[#6;A;!R][CX3](=[OX1])[NX3]",
        count: 599,
    },
    LmsdEntry {
        family: "Fatty Acyls",
        code: "FA05",
        description: "Fatty alcohols",
        smarts: r"[#6;A;!R]~[#6;A;!R]~[#6;A;!R]~[#6;A;!R]~[#6;A;!R]~[#6;A;!R][CX4;H2,H1][OX2H1]",
        count: 512,
    },
    LmsdEntry {
        family: "Fatty Acyls",
        code: "FA12",
        description: "Oxygenated hydrocarbons",
        smarts: r"[#6;A;!R]~[#6;A;!R]~[#6;A;!R]~[#6;A;!R]~[#6;A;!R]~[#6;A;!R][#6](=[OX1])[#6]",
        count: 363,
    },
    LmsdEntry {
        family: "Fatty Acyls",
        code: "FA06",
        description: "Fatty aldehydes",
        smarts: r"[#6;A;!R]~[#6;A;!R]~[#6;A;!R]~[#6;A;!R]~[#6;A;!R][CX3H1](=[OX1])",
        count: 270,
    },
    LmsdEntry {
        family: "Fatty Acyls",
        code: "FA13",
        description: "Fatty acyl glycosides",
        smarts: r"*([#6&$([!X5&C&x]~[#6])&R&r6](/[!$(*/[#8]~*-[#6]),R])-&/[#6&R&r6])-[#8]/[R&x]=,@*-,@[#8].[!#17].[!#6,!R1;$([#6]~[#8])]-[#6&H2]",
        count: 257,
    },
    LmsdEntry {
        family: "Fatty Acyls",
        code: "FA00",
        description: "Other Fatty Acyls",
        smarts: r"*#,-,=[As]",
        count: 50,
    },
    LmsdEntry {
        family: "Fatty Acyls",
        code: "FA09",
        description: "Fatty nitriles",
        smarts: r"*-&!=*!-,=,-,/[!#27&@OH23,v{11-};#6,#7]~*!=*.*#[!122*&@OH23]",
        count: 28,
    },
    LmsdEntry {
        family: "Fatty Acyls",
        code: "FA10",
        description: "Fatty ethers",
        smarts: r"*[!+2]~[!+2].*(-[!@SP3,C,D{-11},H5,X{16-},a,h5,r])=[#6&$([#6]~[#8])]",
        count: 18,
    },
    LmsdEntry {
        family: "Glycerolipids",
        code: "GL03",
        description: "Triradylglycerols",
        smarts: r"[CX4]([OX2][CX3](=[OX1])[#6])([CH2X4][OX2][CX3](=[OX1])[#6])[CH2X4][OX2][CX3](=[OX1])[#6]",
        count: 6936,
    },
    LmsdEntry {
        family: "Glycerolipids",
        code: "GL02",
        description: "Diradylglycerols",
        smarts: r"[CHX4;$([CHX4]([OX2H1])([CH2X4][OX2][CX3](=[OX1])[#6])[CH2X4][OX2][CX3](=[OX1])[#6]),$([CHX4]([OX2][CX3](=[OX1])[#6])([CH2X4][OX2H1])[CH2X4][OX2][CX3](=[OX1])[#6])]",
        count: 604,
    },
    LmsdEntry {
        family: "Glycerolipids",
        code: "GL05",
        description: "Glycosyldiradylglycerols",
        smarts: r"*(!:[#6]([#6&R])~[#8])-[#8&R]-&@[#6&r6](-[!#8])!=[+0]-,@[!#6,!r6]~[!#17,!R{9-},+,-,-5,H14&x;A].*=[#8]",
        count: 104,
    },
    LmsdEntry {
        family: "Glycerolipids",
        code: "GL01",
        description: "Monoradylglycerols",
        smarts: r"*[#6](-,=,@[#6&H2])-,:,@[H2][#8&H]",
        count: 93,
    },
    LmsdEntry {
        family: "Glycerolipids",
        code: "GL04",
        description: "Glycosylmonoradylglycerols",
        smarts: r"*~*[#1].*!:[!h5&!s,A,H{15-};#6;r6]~[#8]~[!#8]-[#6]",
        count: 25,
    },
    LmsdEntry {
        family: "Glycerolipids",
        code: "GL07",
        description: "Betaine diradylglycerols",
        smarts: r"[NX4+]([CH3])([CH3])([CH3])[CX4][CX4][CX4][OX2][CH2X4][CHX4]([OX2][CX3](=[OX1])[#6])[CH2X4][OX2][CX3](=[OX1])[#6]",
        count: 16,
    },
    LmsdEntry {
        family: "Glycerolipids",
        code: "GL00",
        description: "Other Glycerolipids",
        smarts: r"*~[#6]*[C]~*~[C]*[C]~***~*[#6]-*~*[!#27&@OH23]*-*~[!H14].*(-*!-,=,/*~[!H13])[#6]~*(~[#6])-[#6]",
        count: 10,
    },
    LmsdEntry {
        family: "Glycerolipids",
        code: "GL06",
        description: "Betaine monoradylglycerols",
        smarts: r"[NX4+]([CH3])([CH3])([CH3])[CX4][CX4][CX4][OX2][CH2X4][CHX4]([OX2H1])[CH2X4][OX2][CX3](=[OX1])[#6]",
        count: 7,
    },
    LmsdEntry {
        family: "Glycerophospholipids",
        code: "GP01",
        description: "Glycerophosphocholines",
        smarts: r"[PX4](=[OX1])([OX2,OX1-])([OX2])[OX2][CH2X4][CH2X4][NX4+]([CH3])([CH3])[CH3]",
        count: 1905,
    },
    LmsdEntry {
        family: "Glycerophospholipids",
        code: "GP02",
        description: "Glycerophosphoethanolamines",
        smarts: r"[#6][CX3](=[OX1])[OX2][#6][#6][OX2][PX4](=[OX1])([OX2,OX1-])[OX2][CH2X4][CH2X4][NX3,NX4+]",
        count: 1565,
    },
    LmsdEntry {
        family: "Glycerophospholipids",
        code: "GP04",
        description: "Glycerophosphoglycerols",
        smarts: r"*(!=[!F]~[!#7]-[#15](-,:[!#7]-[!F]/*-[#8])~[#8])~[#8&z{-10}].*!=[v{-13}]",
        count: 1351,
    },
    LmsdEntry {
        family: "Glycerophospholipids",
        code: "GP12",
        description: "Glycerophosphoglycerophosphoglycerols",
        smarts: r"[PX4](=[OX1])([OX2,OX1-])([OX2])[OX2][CH2X4][CHX4]([OX2H1,OX2])[CH2X4][OX2][PX4](=[OX1])([OX2,OX1-])[OX2]",
        count: 1306,
    },
    LmsdEntry {
        family: "Glycerophospholipids",
        code: "GP03",
        description: "Glycerophosphoserines",
        smarts: r"[#6][CX3](=[OX1])[OX2][#6][#6][OX2][PX4](=[OX1])([OX2,OX1-])[OX2][CH2X4][CHX4]([CX3](=[OX1])[OX2H1,OX1-])[NX3,NX4+]",
        count: 1231,
    },
    LmsdEntry {
        family: "Glycerophospholipids",
        code: "GP10",
        description: "Glycerophosphates",
        smarts: r"[#6;A;!R]~[#6;A;!R]~[#6;A;!R][CX3](=[OX1])[OX2][#6][#6][OX2][PX4](=[OX1])([OX2H1,OX1-])[OX2H1,OX1-]",
        count: 1205,
    },
    LmsdEntry {
        family: "Glycerophospholipids",
        code: "GP06",
        description: "Glycerophosphoinositols",
        smarts: r"[PX4](=[OX1])([OX2,OX1-])([OX2])[OX2][#6;r6]1[#6;r6]([OX2H1])[#6;r6]([OX2H1])[#6;r6]([OX2H1])[#6;r6]([OX2H1])[#6;r6]1[OX2H1]",
        count: 1199,
    },
    LmsdEntry {
        family: "Glycerophospholipids",
        code: "GP15",
        description: "Glycerophosphoinositolglycans",
        smarts: r"[!H16,$(I#[#6]~[#8]),$([#6]~[#8]),-,C,S,h12].[!Mn][#6&r6]@[#6]-[#8]~[#15]-[#8]",
        count: 338,
    },
    LmsdEntry {
        family: "Glycerophospholipids",
        code: "GP20",
        description: "Oxidized glycerophospholipids",
        smarts: r"*!-,@[#15]!#*-*~[#6](~[#6&H2])~[#8]~[#6]=[#8].*-[!#8,!H]",
        count: 273,
    },
    LmsdEntry {
        family: "Glycerophospholipids",
        code: "GP21",
        description: "Glycerophosphoethanolamine glycans",
        smarts: r"[PX4](=[OX1])([OX2,OX1-])([OX2][#6])[OX2][CH2X4][CH2X4][NX3][#6][#6;R][OX2;R]",
        count: 44,
    },
    LmsdEntry {
        family: "Glycerophospholipids",
        code: "GP13",
        description: "CDP-Glycerols",
        smarts: r"[#6][CX3](=[OX1])[OX2][#6][#6][OX2][PX4](=[OX1])([OX2,OX1-])[OX2][PX4](=[OX1])([OX2,OX1-])[OX2][CH2X4][#6;r5]1[OX2;r5][#6;r5]([#7;r6])[#6;r5]([OX2H1])[#6;r5]1[OX2H1]",
        count: 37,
    },
    LmsdEntry {
        family: "Glycerophospholipids",
        code: "GP14",
        description: "Glycosylglycerophospholipids",
        smarts: r"*(*~[O])!#&@O!=[!$([#6]-[#8].[#7]);#14,X{-4},a].[!#6]~[!#6]",
        count: 24,
    },
    LmsdEntry {
        family: "Glycerophospholipids",
        code: "GP22",
        description: "Dihydroxyacetonephosphates",
        smarts: r"[PX4](=[OX1])([OX2,OX1-])([OX2,OX1-])[OX2][CH2X4][CX3](=[OX1])[CH2X4][OX2][#6]",
        count: 16,
    },
    LmsdEntry {
        family: "Glycerophospholipids",
        code: "GP00",
        description: "Other Glycerophospholipids",
        smarts: r"*-,@*~*.*=[!H{-14},!Si](~[#6,#7])~[#8].[P]",
        count: 15,
    },
    LmsdEntry {
        family: "Glycerophospholipids",
        code: "GP23",
        description: "Glycerophosphoethanols",
        smarts: r"[CH3X4][CH2X4][OX2][PX4](=[OX1])([OX2,OX1-])[OX2][CH2X4][CHX4]([OX2])[CH2X4][OX2][CX3](=[OX1])[#6]",
        count: 8,
    },
    LmsdEntry {
        family: "Glycerophospholipids",
        code: "GP25",
        description: "Cyclic glycerophosphatidic acids",
        smarts: r"[#15;r5](=[OX1])([OX2,OX1-])([OX2;r5][#6;r5])[OX2;r5][#6;r5][#6][OX2][CX3](=[OX1])[#6]",
        count: 6,
    },
    LmsdEntry {
        family: "Glycerophospholipids",
        code: "GP07",
        description: "Glycerophosphoinositol monophosphates",
        smarts: r"[#6;r6]([OX2][PX4](=[OX1])([OX2,OX1-])[OX2][#6])~[#6;r6]~[#6;r6]~[#6;r6][OX2][PX4](=[OX1])([OX2H1,OX1-])[OX2H1,OX1-]",
        count: 4,
    },
    LmsdEntry {
        family: "Glycerophospholipids",
        code: "GP08",
        description: "Glycerophosphoinositol bisphosphates",
        smarts: r"*(-O-[#15])-&/[C&r6].*!:,@[#15]",
        count: 4,
    },
    LmsdEntry {
        family: "Glycerophospholipids",
        code: "GP24",
        description: "Glycerophosphothreonines",
        smarts: r"[CX3](=[OX1])([OX2H1,OX1-])[CHX4]([NX3,NX4+])[CHX4]([CH3])[OX2][PX4](=[OX1])([OX2,OX1-])[OX2][#6]",
        count: 3,
    },
    LmsdEntry {
        family: "Glycerophospholipids",
        code: "GP05",
        description: "Glycerophosphoglycerophosphates",
        smarts: r"[PX4](=[OX1])([OX2,OX1-])([OX2])[OX2][CH2X4][CHX4]([OX2H1])[CH2X4][OX2][PX4](=[OX1])([OX2,OX1-])[OX2,OX1-]",
        count: 3,
    },
    LmsdEntry {
        family: "Glycerophospholipids",
        code: "GP11",
        description: "Glyceropyrophosphates",
        smarts: r"[#6;A;!R]~[#6;A;!R]~[#6;A;!R][CX3](=[OX1])[OX2][#6][#6][OX2][PX4](=[OX1])([OX2,OX1-])[OX2][PX4](=[OX1])([OX2,OX1-])[OX2,OX1-]",
        count: 2,
    },
    LmsdEntry {
        family: "Glycerophospholipids",
        code: "GP17",
        description: "Glycerophosphonoethanolamines",
        smarts: r"[PX4](=[OX1])([OX2,OX1-])([OX2][#6][#6][OX2][CX3](=[OX1])[#6])[CH2X4][CH2X4][NX3,NX4+]",
        count: 1,
    },
    LmsdEntry {
        family: "Glycerophospholipids",
        code: "GP16",
        description: "Glycerophosphonocholines",
        smarts: r"[PX4](=[OX1])([OX2,OX1-])([OX2][#6])[CH2X4][CH2X4][NX4+]([CH3])([CH3])[CH3]",
        count: 1,
    },
    LmsdEntry {
        family: "Glycerophospholipids",
        code: "GP18",
        description: "Di-glycerol tetraether phospholipids (caldarchaeols)",
        smarts: r"[OX2H1][CH2X4][CHX4;R]([OX2;R][CH2X4;R][CH2X4;R])[CH2X4;R][OX2;R][CH2X4;R][CH2X4;R]",
        count: 1,
    },
    LmsdEntry {
        family: "Glycerophospholipids",
        code: "GP19",
        description: "Glycerol-nonitol tetraether phospholipids",
        smarts: r"[OX2H1][CH2X4][CX4]([OX2H1])([CHX4]([OX2H1])[CHX4;R][OX2;R][CH2X4;R])[CHX4]([OX2H1])[CHX4]([OX2H1])[CHX4]([OX2H1])[CH2X4][OX2H1]",
        count: 1,
    },
    LmsdEntry {
        family: "Glycerophospholipids",
        code: "GP09",
        description: "Glycerophosphoinositol trisphosphates",
        smarts: r"[PX4](=[OX1])([OX2,OX1-])([OX2][#6])[OX2][#6;r6]1[#6;r6][#6;r6]([OX2][PX4](=[OX1])([OX2H1,OX1-])[OX2H1,OX1-])[#6;r6]([OX2][PX4](=[OX1])([OX2H1,OX1-])[OX2H1,OX1-])[#6;r6]([OX2][PX4](=[OX1])([OX2H1,OX1-])[OX2H1,OX1-])[#6;r6]1",
        count: 1,
    },
    LmsdEntry {
        family: "Sphingolipids",
        code: "SP05",
        description: "Neutral glycosphingolipids",
        smarts: r"*-[#6&R1](-&@*!#[#6&R]-*)~[R]-[R]@[#6&$([#6]-[#8]~[#6])]-&!@[$([#6]-[#8].[#7])].[v{-11}]",
        count: 2117,
    },
    LmsdEntry {
        family: "Sphingolipids",
        code: "SP06",
        description: "Acidic glycosphingolipids",
        smarts: r"*.*(=[!15N,!H12,!x7])([#8])~[$([#6]-[#8].[#7])&+0&r]",
        count: 1393,
    },
    LmsdEntry {
        family: "Sphingolipids",
        code: "SP02",
        description: "Ceramides",
        smarts: r"*(-*(-*(-[!#1,!X,@TH1&v2,R{9-16};!X,#15,#34,$([!#7]~[!*]),R{9-16},a])~[#7&H]-*(-,=[!X12,R2,x{-9}])~[A])~[!#6,!X5])~[!X12,R2,x{-9}].[!183*]",
        count: 612,
    },
    LmsdEntry {
        family: "Sphingolipids",
        code: "SP03",
        description: "Phosphosphingolipids",
        smarts: r"*-*-*(-*)-,@[#7]!@*=*.*~P",
        count: 353,
    },
    LmsdEntry {
        family: "Sphingolipids",
        code: "SP01",
        description: "Sphingoid bases",
        smarts: r"*.[#6&+0]~[#6]~[#6][$([#6]~[#7])](-[#6])~[+0]",
        count: 129,
    },
    LmsdEntry {
        family: "Sphingolipids",
        code: "SP00",
        description: "Other Sphingolipids",
        smarts: r"*.*!=[#16]~*[!#27&@OH23]-[+0]-*-,:,=*",
        count: 11,
    },
    LmsdEntry {
        family: "Sphingolipids",
        code: "SP04",
        description: "Phosphonosphingolipids",
        smarts: r"*(/[#15]~[!#6&!#7])-[#6]-[#7]",
        count: 9,
    },
    LmsdEntry {
        family: "Sphingolipids",
        code: "SP07",
        description: "Basic glycosphingolipids",
        smarts: r"[OX2;r6]1[#6;r6][#6;r6]([OX2H1])[#6;r6]([OX2H1])[#6;r6]([OX2H1])[#6;r6]1[OX2][CH2X4][CHX4]([NX3H2])[CHX4]([OX2H1])[CX3]=[CX3]",
        count: 1,
    },
    LmsdEntry {
        family: "Sphingolipids",
        code: "SP08",
        description: "Amphoteric glycosphingolipids",
        smarts: r"[SX4](=[OX1])(=[OX1])([OX2H1,OX1-])[OX2][CH2X4][#6;r6]1[OX2;r6][#6;r6]([OX2][CH2X4][CHX4]([NX3H2])[CHX4]([OX2H1]))[#6;r6][#6;r6][#6;r6]1",
        count: 1,
    },
    LmsdEntry {
        family: "Sterol Lipids",
        code: "ST01",
        description: "Sterols",
        smarts: r"*[A](!#&!=[R&X{3-}&r6])-[r6&x3](/*):,@*",
        count: 1923,
    },
    LmsdEntry {
        family: "Sterol Lipids",
        code: "ST04",
        description: "Bile acids and derivatives",
        smarts: r"[!#6,!H2](-[#6&H3])-[#6]-&/[#6&r5]-&/[#6&H3]",
        count: 795,
    },
    LmsdEntry {
        family: "Sterol Lipids",
        code: "ST03",
        description: "Secosteroids",
        smarts: r"*@*(-*)(/**@[#6,+]~[R])~[#6&x]",
        count: 761,
    },
    LmsdEntry {
        family: "Sterol Lipids",
        code: "ST02",
        description: "Steroids",
        smarts: r"*!#[A]~[#6&R&h0](-&@[#6&R&h0]!:[!+2]@,-,:[#6&R&h0]~[#6&r6]-1)-[R]-1",
        count: 402,
    },
    LmsdEntry {
        family: "Sterol Lipids",
        code: "ST05",
        description: "Steroid conjugates",
        smarts: r"*!=&@[!#6&!#7,!R2,48*;!Si]~[D4](!=*(-*)[#6&X4])~[#6,#7;R{-3}]@[!D4]~[#6,#7;R{-3}]@*/,@[!D4]@[R&x{-7}].*~[#8]",
        count: 232,
    },
    LmsdEntry {
        family: "Prenol Lipids",
        code: "PR01",
        description: "Isoprenoids",
        smarts: r"[#6](=[#6]-[#6])(-[CH3])-[#6]-[#6]",
        count: 2475,
    },
    LmsdEntry {
        family: "Prenol Lipids",
        code: "PR02",
        description: "Quinones and hydroquinones",
        smarts: r"*-[#6&D3&r6][#6&r6]!-*(~[#6&D3&r6])-[C].C",
        count: 82,
    },
    LmsdEntry {
        family: "Prenol Lipids",
        code: "PR04",
        description: "Hopanoids",
        smarts: r"*-&/*.*-,=*/*-&@*(-,=[#6&H3])-&/[#6&H3]",
        count: 50,
    },
    LmsdEntry {
        family: "Prenol Lipids",
        code: "PR03",
        description: "Polyprenols",
        smarts: r"[#6](=[#6]-[#6]-[#8])(-[#6])-[#6]-[#6]-[#6]=[#6](-[#6])-[#6]",
        count: 37,
    },
    LmsdEntry {
        family: "Saccharolipids",
        code: "SL03",
        description: "Acyltrehaloses",
        smarts: r"*.*(!-,!=,@*(!:*(~[!r,!x{7-},#8&R&h,+,@@,A&r27,A&x{8-},C,R{-12},Si;!r,!x{7-},#8&R&h,+,@@,D,R{-12};!r{1-7}&!r{9-},!x{7-},#34,#8&R&h,+,+2,@@,R{-12};!x{7-},#34,#8&R&h,+,+2,15N,@@,F,R0,R{-12}&r,r8,v0;!x{7-},#8&R&h,+,15N,@@,A&r27,A&x{8-},C,F,R0,R{-12}&r,Si,v0;!x{7-},#8&R&h,+,15N,@@,D,F,R0,R{-12}&r,v0])-[C]-,=[#8])~[#8&R])-C-[#8&R]",
        count: 1305,
    },
    LmsdEntry {
        family: "Saccharolipids",
        code: "SL05",
        description: "Other acyl sugars",
        smarts: r"*.*-&@*-[#6&r{-23}]-[#8][#6]=[#8]",
        count: 25,
    },
    LmsdEntry {
        family: "Saccharolipids",
        code: "SL01",
        description: "Acylaminosugars",
        smarts: r"[#6;r6]([NX3][CX3](=[OX1])[CH2X4][CHX4]([OX2])[#6;A;!R]~[#6;A;!R]~[#6;A;!R])[#6;r6][OX2;r6]",
        count: 18,
    },
    LmsdEntry {
        family: "Saccharolipids",
        code: "SL02",
        description: "Acylaminosugar glycans",
        smarts: r"[#6;r6]1([CX3](=[OX1])[OX2H1,OX1-])[OX2;r6][#6;r6]([#6][OX2])[#6;r6]([OX2H1])[#6;r6]([OX2])[CH2X4;r6]1",
        count: 3,
    },
    LmsdEntry {
        family: "Polyketides",
        code: "PK12",
        description: "Flavonoids",
        smarts: r"[#6;r6]1~[#6;r6]~[#6;r6]~[#6;r6]2~[#6;r6](~[#6;r6]1)~[#6,#8]~[#6]~[#6](~[#6;r6]3~[#6;r6]~[#6;r6]~[#6;r6]~[#6;r6]~[#6;r6]3)~[#8,#6]2",
        count: 6602,
    },
    LmsdEntry {
        family: "Polyketides",
        code: "PK13",
        description: "Aromatic polyketides",
        smarts: r"[#6;r6]1~[#6;r6]~[#6;r6]~[#6;r6]([OX2,OX1])~[#6;r6]([#6;R,#6;!R](=[OX1,#6]))~[#6;r6]1",
        count: 199,
    },
    LmsdEntry {
        family: "Polyketides",
        code: "PK15",
        description: "Phenolic lipids",
        smarts: r"*(~[#6&+0&r6]!-*~[#8])~[R].[!#7]",
        count: 127,
    },
    LmsdEntry {
        family: "Polyketides",
        code: "PK03",
        description: "Annonaceae acetogenins",
        smarts: r"*(-&/*(=[#8])-[#8]@[#6])-[!+2]-&/*~[!+2]",
        count: 99,
    },
    LmsdEntry {
        family: "Polyketides",
        code: "PK04",
        description: "Macrolides and lactone polyketides",
        smarts: r"*-*.*!:[!N](!-,/C)-[#8]-[$([#6]=[#8])&r]",
        count: 46,
    },
    LmsdEntry {
        family: "Polyketides",
        code: "PK09",
        description: "Polyether antibiotics",
        smarts: r"*(-*-,:[A])-&@[#53,C,N](-&@[!#1&!#6&!R{14-}])-,:[#6&R&v{-16}]",
        count: 41,
    },
    LmsdEntry {
        family: "Polyketides",
        code: "PK11",
        description: "Cytochalasins",
        smarts: r"[#6]/[x4]",
        count: 38,
    },
    LmsdEntry {
        family: "Polyketides",
        code: "PK02",
        description: "Halogenated acetogenins",
        smarts: r"[#35][#6]~[#6]~[#6;R][OX2;R]",
        count: 15,
    },
    LmsdEntry {
        family: "Polyketides",
        code: "PK01",
        description: "Linear polyketides",
        smarts: r"*[#6]-[#6,R;v{3-}](~[#6])-[#6]*[0*]-[#6,R14](~[#6])-[#6]*.*[#6]-[#6,R14](-,@[!#17,!+0,!H3&!H9,!h]#,-,=[#6])~[!-2]",
        count: 10,
    },
    LmsdEntry {
        family: "Polyketides",
        code: "PK07",
        description: "Linear tetracyclines",
        smarts: r"*~*(~*~[#6](-*)~[#6]-[#6]~1)~[#6]~1~[#6].*~[#6]~*~*",
        count: 8,
    },
    LmsdEntry {
        family: "Polyketides",
        code: "PK05",
        description: "Ansamycins and related polyketides",
        smarts: r"*~*.*!=[#6](-[#6](!=*)~[$([#6]-[#8].[#7])]*~[#6&x]~*[#6&x]!=[#8])-[#8]",
        count: 8,
    },
    LmsdEntry {
        family: "Polyketides",
        code: "PK10",
        description: "Aflatoxins and related substances",
        smarts: r"[OX2;r5][#6;r5]1[OX2;r5][#6;r5]~[#6;r5][#6;r5]1",
        count: 6,
    },
    LmsdEntry {
        family: "Polyketides",
        code: "PK06",
        description: "Polyenes",
        smarts: r"[#6;!r6]=[#6;!r6]-[#6;!r6]=[#6;!r6]-[#6;!r6]=[#6;!r6]",
        count: 5,
    },
    LmsdEntry {
        family: "Polyketides",
        code: "PK14",
        description: "Non-ribosomal peptide/polyketide hybrids",
        smarts: r"[#6;R]~[#6][CX3](=[OX1])[NX3][CX4][CX3](=[OX1])",
        count: 5,
    },
    LmsdEntry {
        family: "Polyketides",
        code: "PK08",
        description: "Angucyclines",
        smarts: r"[#6;r6]1=[#6;r6][#6;r6]2[#6;r6](=[OX1])[#6;r6]3[#6;r6]4[#6;r6](=[OX1])[#6;r6][#6;r6][#6;r6][#6;r6]=4[#6;r6]=[#6;r6][#6;r6]=3[#6;r6](=[OX1])[#6;r6]=2[#6;r6]=[#6;r6]1",
        count: 4,
    },
    LmsdEntry {
        family: "Polyketides",
        code: "PK00",
        description: "Other Polyketides",
        smarts: r"[OX2;R]1[#6;R]2[#6;R][#6;R][#6;R]1([CX3](=[OX1])[OX2H1,OX1-])[#6;R]([OX2H1])([CX3](=[OX1])[OX2H1,OX1-])[#6;R]([CX3](=[OX1])[OX2H1,OX1-])[OX2;R]2",
        count: 1,
    },
];

/// Family names in LIPID MAPS hierarchy order, each with its microshades palette.
const LMSD_FAMILY_PALETTES: &[(&str, &[&str; 5])] = &[
    ("Fatty Acyls", &FA_PALETTE),
    ("Glycerolipids", &GL_PALETTE),
    ("Glycerophospholipids", &GP_PALETTE),
    ("Sphingolipids", &SP_PALETTE),
    ("Sterol Lipids", &ST_PALETTE),
    ("Prenol Lipids", &PR_PALETTE),
    ("Saccharolipids", &SL_PALETTE),
    ("Polyketides", &PK_PALETTE),
];

/// Return a static slice of all pre-compiled LMSD subclass classes.
#[must_use]
pub(crate) fn lmsd_classes() -> &'static [ChemicalClass] {
    static CLASSES: std::sync::LazyLock<Vec<ChemicalClass>> = std::sync::LazyLock::new(|| {
        let mut result = Vec::with_capacity(LMSD_ENTRIES.len());
        for (family_name, palette) in LMSD_FAMILY_PALETTES {
            let mut matching: Vec<&LmsdEntry> = LMSD_ENTRIES
                .iter()
                .filter(|e| e.family == *family_name)
                .collect();
            matching.sort_by_key(|e| std::cmp::Reverse(e.count));
            for (idx, entry) in matching.iter().enumerate() {
                let shade_idx = if idx < 4 { idx } else { 4 };
                let full_name = format!("{} [{}]", entry.description, entry.code);
                // `LMSD_FAMILY_PALETTES` types every palette as `&[&str; 5]`, so
                // the length is a type-level invariant, and `shade_idx` above is
                // `0..=4` by construction (`idx` capped at 4, else the literal 4).
                // `shade_idx < 5` is therefore provable, and the lookup cannot
                // miss. Rewriting it as `get` would have to invent a fallback for
                // an unreachable out-of-range index, and turning the loud panic
                // into a silently wrong (or empty) colour is worse than the
                // out-of-bounds check this lint is warning about.
                #[allow(clippy::indexing_slicing)]
                let shade = palette[shade_idx];
                result.push(ChemicalClass::new(
                    full_name,
                    entry.smarts,
                    shade,
                    (*family_name).to_string(),
                ));
            }
        }
        result
    });

    &CLASSES
}

/// Generate all LMSD subclass classes, each with a **specific** SMARTS pattern.
///
/// Each LMSD subclass (FA01-FA13, GL01-GL07, GP01-GP20, SP01-SP08,
/// ST01-ST05, PR01-PR04, SL01-SL05, PK11-PK15) gets its own distinct
/// SMARTS pattern so that `ChemicalClass::matches` identifies the
/// correct subclass directly — no post-hoc family filtering needed.
///
/// Colors are assigned using microshades palettes:
/// - First 4 classes per family get shades 0-3
/// - All remaining classes get shade 4 (the lightest)
///
/// Names use the full LMSD format: "Description \[CODE]"
/// (e.g., "Fatty Acyls and Conjugates \[FA01]").
/// Within each family, classes are sorted by LMSD database count (descending).
#[must_use]
#[cfg(any(test, target_arch = "wasm32"))]
pub(crate) fn lmsd_all() -> Vec<ChemicalClass> {
    lmsd_classes().to_vec()
}
