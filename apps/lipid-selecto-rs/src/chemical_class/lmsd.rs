//! LIPID MAPS Structure Database (LMSD) subclass definitions.
//!
//! Contains the 58 standard LMSD subclasses (FA01–FA13, GL01–GL07, GP01–GP20,
//! SP01–SP08, ST01–ST05, PR01–PR04, SL01–SL05, PK11–PK15) with specific SMARTS
//! patterns. This is the primary classification system used by [`super::lmsd_all`].

use super::{
    ChemicalClass, FA_PALETTE, GL_PALETTE, GP_PALETTE, PK_PALETTE, PR_PALETTE, SL_PALETTE,
    SP_PALETTE, ST_PALETTE,
};

/// Combined LMSD entry: family, code, human-readable description,
/// SMARTS pattern, and LMSD database count.
///
/// This is the single source of truth for [`lmsd_all`] and the LMSD SMARTS
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
        smarts: r"[CX3](=[OX1])[OH]",
        count: 3102,
    },
    LmsdEntry {
        family: "Fatty Acyls",
        code: "FA07",
        description: "Fatty esters",
        smarts: r"[CX3](=[OX1])[O;$(OC)]",
        count: 2454,
    },
    LmsdEntry {
        family: "Fatty Acyls",
        code: "FA03",
        description: "Eicosanoids",
        smarts: r"[#6]~[#6]~[#6]~[#6]~[#6]~[#6]~[#6]~[#6]~[#6]~[#6]~[#6]~[#6]~[#6]~[#6]~[#6]~[#6]~[#6]~[#6][CX3](=[OX1])[OH]",
        count: 1381,
    },
    LmsdEntry {
        family: "Fatty Acyls",
        code: "FA04",
        description: "Docosanoids",
        smarts: r"[#6]~[#6]~[#6]~[#6]~[#6]~[#6]~[#6]~[#6]~[#6]~[#6]~[#6]~[#6]~[#6]~[#6]~[#6]~[#6]~[#6]~[#6]~[#6]~[#6][CX3](=[OX1])[OH]",
        count: 1191,
    },
    LmsdEntry {
        family: "Fatty Acyls",
        code: "FA02",
        description: "Octadecanoids",
        smarts: r"[#6]~[#6]~[#6]~[#6]~[#6]~[#6]~[#6]~[#6]~[#6]~[#6]~[#6]~[#6]~[#6]~[#6]~[#6]~[#6]~[#6][CX3](=[OX1])[OH]",
        count: 763,
    },
    LmsdEntry {
        family: "Fatty Acyls",
        code: "FA11",
        description: "Hydrocarbons",
        smarts: r"[#6]~[#6]~[#6]~[#6]~[#6]~[#6]~[#6]~[#6]~[#6]~[#6]",
        count: 701,
    },
    LmsdEntry {
        family: "Fatty Acyls",
        code: "FA08",
        description: "Fatty amides",
        smarts: r"[CX3](=[OX1])[NX3]",
        count: 599,
    },
    LmsdEntry {
        family: "Fatty Acyls",
        code: "FA05",
        description: "Fatty alcohols",
        smarts: r"[CX4][OH]",
        count: 512,
    },
    LmsdEntry {
        family: "Fatty Acyls",
        code: "FA12",
        description: "Oxygenated hydrocarbons",
        smarts: r"[#6]~[#6]~[#6]~[#6]~[#6]~[#6]~[#6]~[#6]~[#6]~[#6]~[O]~[O]",
        count: 363,
    },
    LmsdEntry {
        family: "Fatty Acyls",
        code: "FA06",
        description: "Fatty aldehydes",
        smarts: r"[CX3H1](=O)[#6]",
        count: 270,
    },
    LmsdEntry {
        family: "Fatty Acyls",
        code: "FA13",
        description: "Fatty acyl glycosides",
        smarts: r"[O;$(O[C;R0])][CX3](=[OX1])",
        count: 257,
    },
    LmsdEntry {
        family: "Fatty Acyls",
        code: "FA00",
        description: "Other Fatty Acyls",
        smarts: r"[CX3](=[OX1])[OH]",
        count: 50,
    },
    LmsdEntry {
        family: "Fatty Acyls",
        code: "FA09",
        description: "Fatty nitriles",
        smarts: r"[CX3]#[NX2]",
        count: 28,
    },
    LmsdEntry {
        family: "Fatty Acyls",
        code: "FA10",
        description: "Fatty ethers",
        smarts: r"[CX3][O;$(OC)]",
        count: 18,
    },
    LmsdEntry {
        family: "Glycerolipids",
        code: "GL03",
        description: "Triradylglycerols",
        smarts: r"[CX4]([OX2][CX3](=[OX1])[#6])([OX2][CX3](=[OX1])[#6])[OX2][CX3](=[OX1])[#6]",
        count: 6936,
    },
    LmsdEntry {
        family: "Glycerolipids",
        code: "GL02",
        description: "Diradylglycerols",
        smarts: r"[CX4]([OX2][CX3](=[OX1])[#6])[OX2][CX3](=[OX1])[#6]",
        count: 604,
    },
    LmsdEntry {
        family: "Glycerolipids",
        code: "GL05",
        description: "Glycosyldiradylglycerols",
        smarts: r"[CX4]([OX2][CX3](=[OX1])[#6])[OX2][CX3](=[OX1])[#6][OX2R1]",
        count: 104,
    },
    LmsdEntry {
        family: "Glycerolipids",
        code: "GL01",
        description: "Monoradylglycerols",
        smarts: r"[CH2X4][CHX4][CH2X4][OX2][CX3](=[OX1])[#6]",
        count: 93,
    },
    LmsdEntry {
        family: "Glycerolipids",
        code: "GL04",
        description: "Monoglycosylglycerols",
        smarts: r"[CH2X4][CHX4][CH2X4][OX2][CX3](=[OX1])[#6][OX2R1]",
        count: 25,
    },
    LmsdEntry {
        family: "Glycerolipids",
        code: "GL07",
        description: "Betaine diradylglycerols",
        smarts: r"[NX3+]([CH3])([CH3])[CH2X4][OX2][CX3](=[OX1])[#6][CH2X4][OX2][CX3](=[OX1)]",
        count: 16,
    },
    LmsdEntry {
        family: "Glycerolipids",
        code: "GL00",
        description: "Other Glycerolipids",
        smarts: r"[CX4]([OX2][CX3](=[OX1])[#6])",
        count: 10,
    },
    LmsdEntry {
        family: "Glycerolipids",
        code: "GL06",
        description: "Betaine monoradylglycerols",
        smarts: r"[NX3+]([CH3])([CH3])[CH2X4][OX2][CX3](=[OX1])[#6]",
        count: 7,
    },
    LmsdEntry {
        family: "Glycerophospholipids",
        code: "GP01",
        description: "Glycerophosphocholines",
        smarts: r"[PX4](=[OX1])([OX2])([OX2])[NX4+]([CH3])([CH3])[CH3]",
        count: 1905,
    },
    LmsdEntry {
        family: "Glycerophospholipids",
        code: "GP02",
        description: "Glycerophosphoethanolamines",
        smarts: r"[PX4](=[OX1])([OX2])([OX2])[CH2X4][CH2X4][NX3;H2,H1,H0]",
        count: 1565,
    },
    LmsdEntry {
        family: "Glycerophospholipids",
        code: "GP04",
        description: "Glycerophosphoglycerols",
        smarts: r"[PX4](=[OX1])([OX2])([OX2])[CH2X4][CHX4]([OX2H,OX1-])[CH2X4][OX2H,OX1-]",
        count: 1351,
    },
    LmsdEntry {
        family: "Glycerophospholipids",
        code: "GP03",
        description: "Glycerophosphoserines",
        smarts: r"[PX4](=[OX1])([OX2])([OX2])[CH2X4][CHX4]([CX3](=[OX1])[OX2H,OX1-])[NX3]",
        count: 1231,
    },
    LmsdEntry {
        family: "Glycerophospholipids",
        code: "GP10",
        description: "Glycerophosphates",
        smarts: r"[PX4](=[OX1])([OX2])([OX2])[CH2X4][CHX4][CH2X4][OX2H,OX1-]",
        count: 1205,
    },
    LmsdEntry {
        family: "Glycerophospholipids",
        code: "GP06",
        description: "Glycerophosphoinositols",
        smarts: r"[PX4](=[OX1])([OX2])([OX2])[CH2X4][CHX4][CH2X4][O!R]1[CH1X4]2[CH1X4][O!R][CH1X4][CH1X4][CH1X4][CH1X4]2[CH1X4]1",
        count: 1199,
    },
    LmsdEntry {
        family: "Glycerophospholipids",
        code: "GP15",
        description: "Glycerophosphoinositolglycans",
        smarts: r"[PX4](=[OX1])([OX2])([OX2])",
        count: 338,
    },
    LmsdEntry {
        family: "Glycerophospholipids",
        code: "GP20",
        description: "Oxidized glycerophospholipids",
        smarts: r"[PX4](=[OX1])([OX2])([OX2])[CH2X4][CHX4]([OH])",
        count: 273,
    },
    LmsdEntry {
        family: "Sphingolipids",
        code: "SP05",
        description: "Neutral glycosphingolipids",
        smarts: r"[NX3][CX3](=[OX1])[CX4][CH1X4][CH1X4][OX2][CH1X4][CH1X4]",
        count: 2117,
    },
    LmsdEntry {
        family: "Sphingolipids",
        code: "SP06",
        description: "Acidic glycosphingolipids",
        smarts: r"[NX3][CX3](=[OX1])[CX4][CH1X4][CH1X4][OX2][S(=O)(=O)[O-]]",
        count: 1393,
    },
    LmsdEntry {
        family: "Sphingolipids",
        code: "SP02",
        description: "Ceramides",
        smarts: r"[NX3][CX3](=[OX1])[CX4]",
        count: 612,
    },
    LmsdEntry {
        family: "Sphingolipids",
        code: "SP03",
        description: "Phosphosphingolipids",
        smarts: r"[NX4+][CX4][CX4][OX2][PX4](=[OX1])[OX2]",
        count: 353,
    },
    LmsdEntry {
        family: "Sphingolipids",
        code: "SP01",
        description: "Sphingoid bases",
        smarts: r"[NX3][CX3]",
        count: 129,
    },
    LmsdEntry {
        family: "Sphingolipids",
        code: "SP00",
        description: "Other Sphingolipids",
        smarts: r"[NX3][CX3](=[OX1])",
        count: 11,
    },
    LmsdEntry {
        family: "Sphingolipids",
        code: "SP04",
        description: "Phosphonosphingolipids",
        smarts: r"[NX3][CX3](=[OX1])[CX4][OX2][PX4](=[OX1])",
        count: 9,
    },
    LmsdEntry {
        family: "Sphingolipids",
        code: "SP08",
        description: "Amphoteric glycosphingolipids",
        smarts: r"[NX3][CX3](=[OX1])[CX4][CH1X4][CH1X4][OX2][S(=O)(=O)]",
        count: 1,
    },
    LmsdEntry {
        family: "Sterol Lipids",
        code: "ST01",
        description: "Sterols",
        smarts: r"[#6]1[#6][#6][#6]2[#6]([#6]1)[#6][#6][#6]2([#6])[#6]",
        count: 1923,
    },
    LmsdEntry {
        family: "Sterol Lipids",
        code: "ST04",
        description: "Bile acids and derivatives",
        smarts: r"[#6]1[#6][#6][#6]2[#6]([#6]1)[#6][#6][#6]2([#6])[#6][CX3](=[OX1])[OH]",
        count: 795,
    },
    LmsdEntry {
        family: "Sterol Lipids",
        code: "ST03",
        description: "Secosteroids",
        smarts: r"[#6]1[#6][#6][#6]2[#6]([#6]1)[#6][#6]",
        count: 761,
    },
    LmsdEntry {
        family: "Sterol Lipids",
        code: "ST02",
        description: "Steroids",
        smarts: r"[#6;R1]1[#6;R1][#6;R1][#6;R1]2[#6;R1]([#6;R1]1)[#6;R1][#6;R1][#6;R1][#6;R1]2",
        count: 402,
    },
    LmsdEntry {
        family: "Sterol Lipids",
        code: "ST05",
        description: "Steroid conjugates",
        smarts: r"[#6]1[#6][#6][#6]2[#6]([#6]1)[#6][#6][#6]2([#6])[#6][CX3](=[OX1])",
        count: 232,
    },
    LmsdEntry {
        family: "Prenol Lipids",
        code: "PR01",
        description: "Isoprenoids",
        smarts: r"[#6]=[#6][#6]=[#6][#6]",
        count: 2475,
    },
    LmsdEntry {
        family: "Prenol Lipids",
        code: "PR02",
        description: "Quinones and hydroquinones",
        smarts: r"[CX3](=O)[CX3](=O)",
        count: 82,
    },
    LmsdEntry {
        family: "Prenol Lipids",
        code: "PR04",
        description: "Hopanoids",
        smarts: r"[#6]1[#6][#6]2[#6][#6][#6]1[#6][#6]3[#6]([#6]2)[#6][#6]4[#6]([#6]3)[#6][#6][#6][#6]4",
        count: 50,
    },
    LmsdEntry {
        family: "Prenol Lipids",
        code: "PR03",
        description: "Polyprenols",
        smarts: r"[#6]=[#6][#6]1[#6]=[CH]",
        count: 37,
    },
    LmsdEntry {
        family: "Saccharolipids",
        code: "SL03",
        description: "Acyltrehaloses",
        smarts: r"[O!R][C;R0][C;R1]1[O!R][C;R1][C;R1][C;R1][O!R][C;R0]1",
        count: 1305,
    },
    LmsdEntry {
        family: "Saccharolipids",
        code: "SL05",
        description: "Other acyl sugars",
        smarts: r"[O!R][C;R0][C;R1]",
        count: 25,
    },
    LmsdEntry {
        family: "Saccharolipids",
        code: "SL01",
        description: "Acylaminosugars",
        smarts: r"[N;!R][C;R0][N;!R]",
        count: 18,
    },
    LmsdEntry {
        family: "Saccharolipids",
        code: "SL02",
        description: "Acylaminosugar glycans",
        smarts: r"[N;!R][C;R0][N;!R][C;R1]1[O!R][C;R1][C;R1][C;R1]1",
        count: 3,
    },
    LmsdEntry {
        family: "Polyketides",
        code: "PK12",
        description: "Flavonoids",
        smarts: r"[C;R1]1[CH;R1][C;R1][C;R1]2[C;R1]([C;R1]1)[C;R1][C;R1][C;R1][C;R1]2[CX3](=[OX1])",
        count: 6602,
    },
    LmsdEntry {
        family: "Polyketides",
        code: "PK13",
        description: "Aromatic polyketides",
        smarts: r"[C;R1]1[CH;R1][C;R1][C;R1][C;R1][C;R1]1",
        count: 199,
    },
    LmsdEntry {
        family: "Polyketides",
        code: "PK15",
        description: "Phenolic lipids",
        smarts: r"[CX3](=[OX1])[O;$(OC)][CH3]",
        count: 127,
    },
    LmsdEntry {
        family: "Polyketides",
        code: "PK03",
        description: "Annonaceae acetogenins",
        smarts: r"[CH2X4]([CX3](=[OX1])[#6])[CX3](=[OX1])O[C;R0]",
        count: 99,
    },
    LmsdEntry {
        family: "Polyketides",
        code: "PK04",
        description: "Macrolides and lactone polyketides",
        smarts: r"[CX3](=[OX1])O[C;R0]1[CH2X4]",
        count: 46,
    },
    LmsdEntry {
        family: "Polyketides",
        code: "PK09",
        description: "Polyether antibiotics",
        smarts: r"[C;R1]1[O!R][C;R1][C;R1][O!R]1",
        count: 41,
    },
    LmsdEntry {
        family: "Polyketides",
        code: "PK11",
        description: "Cytochalasins",
        smarts: r"[C;R1]1[C;R1][C;R1][C;R1][C;R1][C;R1]1[NX2]=[CX3]",
        count: 38,
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

/// Generate all LMSD subclass classes, each with a **specific** SMARTS pattern.
///
/// Each LMSD subclass (FA01-FA13, GL01-GL07, GP01-GP20, SP01-SP08,
/// ST01-ST05, PR01-PR04, SL01-SL05, PK11-PK15) gets its own distinct
/// SMARTS pattern so that [`ChemicalClass::matches`] identifies the
/// correct subclass directly — no post-hoc family filtering needed.
///
/// Colors are assigned using microshades palettes:
/// - First 4 classes per family get shades 0-3
/// - All remaining classes get shade 4 (the lightest)
///
/// Names use the full LMSD format: "Description [CODE]"
/// (e.g., "Fatty Acids and Conjugates [FA01]").
/// Within each family, classes are sorted by LMSD database count (descending).
#[must_use]
pub fn lmsd_all() -> Vec<ChemicalClass> {
    let mut result = Vec::new();
    for (family_name, palette) in LMSD_FAMILY_PALETTES {
        let mut matching: Vec<&LmsdEntry> = LMSD_ENTRIES
            .iter()
            .filter(|e| e.family == *family_name)
            .collect();
        matching.sort_by_key(|e| std::cmp::Reverse(e.count));
        for (idx, entry) in matching.iter().enumerate() {
            let shade_idx = if idx < 4 { idx } else { 4 };
            let full_name = format!("{} [{}]", entry.description, entry.code);
            result.push(ChemicalClass::new(
                full_name,
                entry.smarts,
                palette[shade_idx],
                (*family_name).to_string(),
            ));
        }
    }
    result
}
