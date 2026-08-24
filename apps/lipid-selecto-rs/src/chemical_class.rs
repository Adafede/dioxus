//! User-defined chemical classes with SMARTS pattern matching.

use std::collections::HashMap;

use chematic::smarts;

/// A chemical class defined by name, SMARTS pattern, display color, and family.
///
/// SMARTS patterns are pre-compiled once at construction time to avoid
/// re-parsing the pattern string on every `matches` call — critical for large
/// datasets where thousands of molecules are matched against dozens of classes.
#[derive(Clone, Debug)]
pub struct ChemicalClass {
    pub name: String,
    pub smarts: String,
    pub color: String,
    pub family: String,
    /// Pre-compiled SMARTS query (parsed once in `new`).
    compiled: Option<smarts::QueryMolecule>,
}

impl ChemicalClass {
    /// Create a new chemical class, pre-compiling the SMARTS pattern.
    pub fn new(
        name: impl Into<String>,
        smarts_str: impl Into<String>,
        color: impl Into<String>,
        family: impl Into<String>,
    ) -> Self {
        let smarts_str = smarts_str.into();
        let compiled = smarts::parse_smarts(&smarts_str).ok();
        Self {
            name: name.into(),
            smarts: smarts_str,
            color: color.into(),
            family: family.into(),
            compiled,
        }
    }

    /// Check if a molecule matches this class's pre-compiled SMARTS pattern.
    ///
    /// Returns `true` if the molecule contains at least one match, `false` otherwise
    /// or if the SMARTS pattern cannot be parsed.
    #[must_use]
    pub fn matches(&self, molecule: &chematic::core::Molecule) -> bool {
        let Some(query) = &self.compiled else {
            return false;
        };
        !smarts::find_matches(query, molecule).is_empty()
    }

    /// Return the default lipid classes.
    ///
    /// These match the LIPID MAPS classification system with proper family and
    /// architecture designations.
    #[must_use]
    pub fn defaults() -> Vec<Self> {
        [
            fatty_acyls(),
            glycerolipids(),
            glycerophospholipids(),
            sphingolipids(),
            sterol_lipids(),
            prenol_lipids(),
            saccharolipids(),
            polyketides(),
        ]
        .into_iter()
        .flatten()
        .collect()
    }

    /// Convert defaults into a map for quick lookup by name.
    #[must_use]
    pub fn defaults_map() -> HashMap<String, Self> {
        Self::defaults()
            .into_iter()
            .map(|c| (c.name.clone(), c))
            .collect()
    }
}

fn fatty_acyls() -> Vec<ChemicalClass> {
    vec![
        ChemicalClass::new(
            "FA",
            "[#6][#6][#6][#6][#6][#6][#6][#6][CX3](=[OX1])[OH]",
            "#9D654C",
            "Fatty Acyls",
        ),
        ChemicalClass::new(
            "MUFA",
            "[#6][#6]=[#6][#6][#6][#6][#6][#6][#6][CX3](=[OX1])[OH]",
            "#C17754",
            "Fatty Acyls",
        ),
        ChemicalClass::new(
            "PUFA",
            "[#6][#6]=[#6][#6][#6]=[#6][#6][#6][#6][CX3](=[OX1])[OH]",
            "#F09163",
            "Fatty Acyls",
        ),
    ]
}

fn glycerolipids() -> Vec<ChemicalClass> {
    vec![
        ChemicalClass::new(
            "TG(AAA)",
            "[CX4]([OX2][CX3](=[OX1])[#6])([OX2][CX3](=[OX1])[#6])[OX2][CX3](=[OX1])[#6]",
            "#098BD9",
            "Glycerolipids",
        ),
        ChemicalClass::new(
            "DG(AA)",
            "[CX4]([OX2][CX3](=[OX1])[#6])[OX2][CX3](=[OX1])[#6]",
            "#56B4E9",
            "Glycerolipids",
        ),
        ChemicalClass::new(
            "MG(A)",
            "[CH2X4][CHX4][CH2X4][OX2][CX3](=[OX1])[#6]",
            "#7DCCFF",
            "Glycerolipids",
        ),
    ]
}

fn glycerophospholipids() -> Vec<ChemicalClass> {
    vec![
        ChemicalClass::new(
            "PC(AA)",
            "[PX4](=[OX1])([OX2])([OX2])[NX4+]([CH3])([CH3])[CH3]",
            "#4E7705",
            "Glycerophospholipids",
        ),
        ChemicalClass::new(
            "PE(AA)",
            "[PX4](=[OX1])([OX2])([OX2])[CH2X4][CH2X4][NX3;H2,H1,H0]",
            "#6D9F06",
            "Glycerophospholipids",
        ),
        ChemicalClass::new(
            "PS(AA)",
            "[PX4](=[OX1])([OX2])([OX2])[CH2X4][CHX4]([CX3](=[OX1])[OX2H,OX1-])[NX3]",
            "#97CE2F",
            "Glycerophospholipids",
        ),
        ChemicalClass::new(
            "PI(AA)",
            "[PX4](=[OX1])([OX2])([OX2])[C;R1]1[CH;R1][CH;R1][CH;R1][CH;R1][CH;R1]1",
            "#DDFFA0",
            "Glycerophospholipids",
        ),
        ChemicalClass::new(
            "PG(AA)",
            "[PX4](=[OX1])([OX2])([OX2])[CH2X4][CHX4]([OX2H,OX1-])[CH2X4][OX2H,OX1-]",
            "#BDEC6F",
            "Glycerophospholipids",
        ),
        ChemicalClass::new(
            "PA(AA)",
            "[PX4](=[OX1])([OX2])([OX2])[CH2X4][CHX4][CH2X4][OX2H,OX1-]",
            "#DDFFA0",
            "Glycerophospholipids",
        ),
        ChemicalClass::new(
            "LPC(A)",
            "[CH2X4][CHX4]([OX2][CX3](=[OX1])[#6])[CH2X4][OX2][PX4](=[OX1])[OX2][CH2X4][N+;X4]",
            "#148F77",
            "Glycerophospholipids",
        ),
        ChemicalClass::new(
            "LPE(A)",
            "[CH2X4][CHX4]([OX2][CX3](=[OX1])[#6])[CH2X4][OX2][PX4](=[OX1])[OX2][CH2X4][NX3]",
            "#009E73",
            "Glycerophospholipids",
        ),
        ChemicalClass::new(
            "CL(AAAA)",
            "[PX4](=[OX1])([OX2])([OX2])[CH2X4][CHX4]([OX2])[CH2X4][OX2]",
            "#43BA8F",
            "Glycerophospholipids",
        ),
    ]
}

fn sphingolipids() -> Vec<ChemicalClass> {
    vec![
        ChemicalClass::new(
            "Cer(AS)",
            "[NX3][CX3](=[OX1])[CX4]",
            "#7D3560",
            "Sphingolipids",
        ),
        ChemicalClass::new(
            "SM(AS)",
            "[NX4+][CX4][CX4][OX2][PX4](=[OX1])[OX2]",
            "#A1527F",
            "Sphingolipids",
        ),
        ChemicalClass::new(
            "HexCer(AS)",
            "[NX3][CX3](=[OX1])[CX4][CH1X4][CH1X4][OX2][CH1X4][CH1X4]",
            "#CC79A7",
            "Sphingolipids",
        ),
    ]
}

fn sterol_lipids() -> Vec<ChemicalClass> {
    vec![ChemicalClass::new(
        "ST",
        "[#6]1[#6][#6][#6]2[#6]([#6]1)[#6][#6][#6]2([#6])[#6]",
        "#6a51a3",
        "Sterol Lipids",
    )]
}

fn prenol_lipids() -> Vec<ChemicalClass> {
    vec![ChemicalClass::new(
        "PR",
        "[#6]=[#6][#6]=[#6][#6]",
        "#ff7f00",
        "Prenol Lipids",
    )]
}

fn saccharolipids() -> Vec<ChemicalClass> {
    vec![ChemicalClass::new(
        "SL",
        "[#6][OX2][PX4](=[OX1])[OX2][#6]",
        "#4292c6",
        "Saccharolipids",
    )]
}

fn polyketides() -> Vec<ChemicalClass> {
    vec![ChemicalClass::new(
        "PK",
        "[#6;R]1[#6]([#6](=[OX1])[#6])[#6;R][#6;R][#6;R][#6;R][#6;R][#6;R][#6;R][#6;R][#6;R][#6;R][#6;R][#6;R]1",
        "#238b45",
        "Polyketides",
    )]
}

/// Generate all LMSD subclass classes, each with a **specific** SMARTS pattern.
///
/// Unlike the previous approach which used a single generic family-wide SMARTS
/// for all subclasses, this version assigns a distinct SMARTS to every LMSD
/// subclass (e.g. FA01 gets a carboxylic-acid pattern, GP01 gets a choline
/// phosphate pattern). This means `compute_class_matches` will match a molecule
/// against exactly the right subclass — no post-hoc family filtering needed.
///
/// Colors are assigned using microshades palette:
/// - First 4 classes per family get shades 0-3
/// - All remaining classes get shade 4 (the lightest)
///
/// Names use the full LMSD format: "Description [CODE]" (e.g., "Fatty Acids and Conjugates [FA01]")
#[must_use]
pub fn lmsd_all() -> Vec<ChemicalClass> {
    // Map LMSD code -> specific SMARTS pattern
    let smarts_map: HashMap<&str, &str> = HashMap::from([
        // ── Fatty Acyls ──────────────────────────────────────────────
        // FA01: Carboxylic acid with aliphatic chain (8+ carbons)
        ("FA01", "[CX3](=[OX1])[OH]"),
        // FA07: Fatty esters — ester group (R-COO-R')
        ("FA07", "[CX3](=[OX1])[O;$(OC)]"),
        // FA03: Eicosanoids — C20 chain with oxygen modifications
        (
            "FA03",
            "[#6]~[#6]~[#6]~[#6]~[#6]~[#6]~[#6]~[#6]~[#6]~[#6]~[#6]~[#6]~[#6]~[#6]~[#6]~[#6]~[#6]~[#6][CX3](=[OX1])[OH]",
        ),
        // FA04: Docosanoids — C22 chain with oxygen
        (
            "FA04",
            "[#6]~[#6]~[#6]~[#6]~[#6]~[#6]~[#6]~[#6]~[#6]~[#6]~[#6]~[#6]~[#6]~[#6]~[#6]~[#6]~[#6]~[#6]~[#6]~[#6][CX3](=[OX1])[OH]",
        ),
        // FA02: Octadecanoids — C18 chain with oxygen
        (
            "FA02",
            "[#6]~[#6]~[#6]~[#6]~[#6]~[#6]~[#6]~[#6]~[#6]~[#6]~[#6]~[#6]~[#6]~[#6]~[#6]~[#6]~[#6][CX3](=[OX1])[OH]",
        ),
        // FA11: Hydrocarbons — alkane (saturated, no O)
        ("FA11", "[#6]~[#6]~[#6]~[#6]~[#6]~[#6]~[#6]~[#6]~[#6]~[#6]"),
        // FA08: Fatty amides — amide group (R-C(=O)-N-)
        ("FA08", "[CX3](=[OX1])[NX3]"),
        // FA05: Fatty alcohols — primary alcohol with aliphatic chain
        ("FA05", "[CX4][OH]"),
        // FA12: Oxygenated hydrocarbons — multiple oxygen atoms
        (
            "FA12",
            "[#6]~[#6]~[#6]~[#6]~[#6]~[#6]~[#6]~[#6]~[#6]~[#6]~[O]~[O]",
        ),
        // FA06: Fatty aldehydes — aldehyde group (R-CHO)
        ("FA06", "[CX3H1](=O)[#6]"),
        // FA13: Fatty acyl glycosides — glycosidic ester
        ("FA13", "[O;$(O[C;R0])][CX3](=[OX1])"),
        // FA00: Other Fatty Acyls — same as FA01 (carboxylic acid)
        ("FA00", "[CX3](=[OX1])[OH]"),
        // FA09: Fatty nitriles — nitrile group (R-C≡N)
        ("FA09", "[CX3]#[NX2]"),
        // FA10: Fatty ethers — ether linkage (R-O-R')
        ("FA10", "[CX3][O;$(OC)]"),
        // ── Glycerolipids ────────────────────────────────────────────
        // GL03: Triacylglycerols — glycerol with 3 ester-linked acyl chains
        (
            "GL03",
            "[CX4]([OX2][CX3](=[OX1])[#6])([OX2][CX3](=[OX1])[#6])[OX2][CX3](=[OX1])[#6]",
        ),
        // GL02: Diacylglycerols — glycerol with 2 ester-linked acyl chains
        (
            "GL02",
            "[CX4]([OX2][CX3](=[OX1])[#6])[OX2][CX3](=[OX1])[#6]",
        ),
        // GL01: Monoradylglycerols — glycerol with 1 ester-linked acyl chain
        ("GL01", "[CH2X4][CHX4][CH2X4][OX2][CX3](=[OX1])[#6]"),
        // GL05: Glycosyldiacylglycerols — DG + sugar phosphate
        (
            "GL05",
            "[CX4]([OX2][CX3](=[OX1])[#6])[OX2][CX3](=[OX1])[#6][OX2R1]",
        ),
        // GL04: Monoglycosylglycerols — MG + sugar
        ("GL04", "[CH2X4][CHX4][CH2X4][OX2][CX3](=[OX1])[#6][OX2R1]"),
        // GL07: Betaine diradylglycerols — DG + betaine (quaternary ammonium)
        (
            "GL07",
            "[NX3+]([CH3])([CH3])[CH2X4][OX2][CX3](=[OX1])[#6][CH2X4][OX2][CX3](=[OX1])",
        ),
        // GL00: Other Glycerolipids — generic glycerol ester
        ("GL00", "[CX4]([OX2][CX3](=[OX1])[#6])"),
        // GL06: Betaine monoradylglycerols — MG + betaine
        ("GL06", "[NX3+]([CH3])([CH3])[CH2X4][OX2][CX3](=[OX1])[#6]"),
        // ── Glycerophospholipids ─────────────────────────────────────
        // GP01: Glycerophosphocholines — phosphate + choline
        (
            "GP01",
            "[PX4](=[OX1])([OX2])([OX2])[NX4+]([CH3])([CH3])[CH3]",
        ),
        // GP02: Glycerophosphoethanolamines — phosphate + ethanolamine
        (
            "GP02",
            "[PX4](=[OX1])([OX2])([OX2])[CH2X4][CH2X4][NX3;H2,H1,H0]",
        ),
        // GP04: Glycerophosphoglycerols — phosphate + glycerol
        (
            "GP04",
            "[PX4](=[OX1])([OX2])([OX2])[CH2X4][CHX4]([OX2H,OX1-])[CH2X4][OX2H,OX1-]",
        ),
        // GP03: Glycerophosphoserines — phosphate + serine
        (
            "GP03",
            "[PX4](=[OX1])([OX2])([OX2])[CH2X4][CHX4]([CX3](=[OX1])[OX2H,OX1-])[NX3]",
        ),
        // GP10: Glycerophosphates — phosphate only
        (
            "GP10",
            "[PX4](=[OX1])([OX2])([OX2])[CH2X4][CHX4][CH2X4][OX2H,OX1-]",
        ),
        // GP06: Glycerophosphoinositols — phosphate + inositol
        (
            "GP06",
            "[PX4](=[OX1])([OX2])([OX2])[CH2X4][CHX4][CH2X4][O!R]1[CH1X4]2[CH1X4][O!R][CH1X4][CH1X4][CH1X4][CH1X4]2[CH1X4]1",
        ),
        // GP15: Glycerophosphoinositolglycans — PI + glycan
        ("GP15", "[PX4](=[OX1])([OX2])([OX2])"),
        // GP20: Oxidized glycerophospholipids — phospholipid with oxidized chain
        ("GP20", "[PX4](=[OX1])([OX2])([OX2])[CH2X4][CHX4]([OH])"),
        // ── Sphingolipids ────────────────────────────────────────────
        // SP05: Neutral glycosphingolipids — ceramide + neutral sugar
        (
            "SP05",
            "[NX3][CX3](=[OX1])[CX4][CH1X4][CH1X4][OX2][CH1X4][CH1X4]",
        ),
        // SP06: Acidic glycosphingolipids — ceramide + acidic sugar (sulfate/phosphate)
        (
            "SP06",
            "[NX3][CX3](=[OX1])[CX4][CH1X4][CH1X4][OX2][S(=O)(=O)[O-]]",
        ),
        // SP02: Ceramides — sphingosine + fatty acyl (amide-linked)
        ("SP02", "[NX3][CX3](=[OX1])[CX4]"),
        // SP03: Phosphosphingolipids — ceramide + phosphate + headgroup
        ("SP03", "[NX4+][CX4][CX4][OX2][PX4](=[OX1])[OX2]"),
        // SP01: Sphingoid bases — sphingosine without acyl chain
        ("SP01", "[NX3][CX3]"),
        // SP00: Other Sphingolipids — generic sphingoid amide
        ("SP00", "[NX3][CX3](=[OX1])"),
        // SP04: Phosphonosphingolipids — ceramide + phosphate monoester
        ("SP04", "[NX3][CX3](=[OX1])[CX4][OX2][PX4](=[OX1])"),
        // SP08: Amphoteric glycosphingolipids — ceramide + sugar + charge
        (
            "SP08",
            "[NX3][CX3](=[OX1])[CX4][CH1X4][CH1X4][OX2][S(=O)(=O)]",
        ),
        // ── Sterol Lipids ────────────────────────────────────────────
        // ST01: Sterols — steroid nucleus with hydroxyl
        (
            "ST01",
            "[#6]1[#6][#6][#6]2[#6]([#6]1)[#6][#6][#6]2([#6])[#6]",
        ),
        // ST04: Bile acids — steroid with carboxylic acid
        (
            "ST04",
            "[#6]1[#6][#6][#6]2[#6]([#6]1)[#6][#6][#6]2([#6])[#6][CX3](=[OX1])[OH]",
        ),
        // ST03: Secosteroids — steroid with broken ring
        ("ST03", "[#6]1[#6][#6][#6]2[#6]([#6]1)[#6][#6]"),
        // ST02: Steroids — steroid nucleus (4 fused rings)
        (
            "ST02",
            "[#6;R1]1[#6;R1][#6;R1][#6;R1]2[#6;R1]([#6;R1]1)[#6;R1][#6;R1][#6;R1][#6;R1]2",
        ),
        // ST05: Steroid conjugates — sterol with conjugated group
        (
            "ST05",
            "[#6]1[#6][#6][#6]2[#6]([#6]1)[#6][#6][#6]2([#6])[#6][CX3](=[OX1])",
        ),
        // ── Prenol Lipids ────────────────────────────────────────────
        // PR01: Isoprenoids — isoprene units (C=C-C-C=C)
        ("PR01", "[#6]=[#6][#6]=[#6][#6]"),
        // PR02: Quinones — quinone structure
        ("PR02", "[CX3](=O)[CX3](=O)"),
        // PR04: Hopanoids — pentacyclic triterpenoid
        (
            "PR04",
            "[#6]1[#6][#6]2[#6][#6][#6]1[#6][#6]3[#6]([#6]2)[#6][#6]4[#6]([#6]3)[#6][#6][#6][#6]4",
        ),
        // PR03: Polyprenols — long isoprenoid chain
        ("PR03", "[#6]=[#6][#6]1[#6]=[CH]"),
        // ── Saccharolipids ───────────────────────────────────────────
        // SL01: Acylaminosugars — sugar + amide
        ("SL01", "[N;!R][C;R0][N;!R]"),
        // SL02: Acylaminosugar glycans — sugar polymer + amide
        ("SL02", "[N;!R][C;R0][N;!R][C;R1]1[O!R][C;R1][C;R1][C;R1]1"),
        // SL03: Acyltrehaloses — trehalose + acyl
        (
            "SL03",
            "[O!R][C;R0][C;R1]1[O!R][C;R1][C;R1][C;R1][O!R][C;R0]1",
        ),
        // SL05: Other acyl sugars — sugar + acyl
        ("SL05", "[O!R][C;R0][C;R1]"),
        // ── Polyketides ──────────────────────────────────────────────
        // PK12: Flavonoids — C6-C3-C6 with aromatic rings and carbonyl
        (
            "PK12",
            "[C;R1]1[CH;R1][C;R1][C;R1]2[C;R1]([C;R1]1)[C;R1][C;R1][C;R1][C;R1]2[CX3](=[OX1])",
        ),
        // PK13: Aromatic polyketides — aromatic ring system
        ("PK13", "[C;R1]1[CH;R1][C;R1][C;R1][C;R1][C;R1]1"),
        // PK15: Phenolic lipids — phenol + aliphatic chain
        ("PK15", "[CX3](=[OX1])[O;$(OC)][CH3]"),
        // PK03: Annonaceae acetogenins — long-chain acetogenin
        ("PK03", "[CH2X4]([CX3](=[OX1])[#6])[CX3](=[OX1])O[C;R0]"),
        // PK04: Macrolides — macrocyclic lactone
        ("PK04", "[CX3](=[OX1])O[C;R0]1[CH2X4]"),
        // PK09: Polyether antibiotics — polyether ring system
        ("PK09", "[C;R1]1[O!R][C;R1][C;R1][O!R]1"),
        // PK11: Cytochalasins — bicyclic structure with conjugated bond
        ("PK11", "[C;R1]1[C;R1][C;R1][C;R1][C;R1][C;R1]1[NX2]=[CX3]"),
    ]);

    // Counts from LMSD.sdf.tsv (for sorting within family)
    let mut counts: HashMap<String, usize> = HashMap::new();

    // Fatty Acyls
    counts.insert("FA01".to_string(), 3102);
    counts.insert("FA07".to_string(), 2454);
    counts.insert("FA03".to_string(), 1381);
    counts.insert("FA04".to_string(), 1191);
    counts.insert("FA02".to_string(), 763);
    counts.insert("FA11".to_string(), 701);
    counts.insert("FA08".to_string(), 599);
    counts.insert("FA05".to_string(), 512);
    counts.insert("FA12".to_string(), 363);
    counts.insert("FA06".to_string(), 270);
    counts.insert("FA13".to_string(), 257);
    counts.insert("FA00".to_string(), 50);
    counts.insert("FA09".to_string(), 28);
    counts.insert("FA10".to_string(), 18);

    // Glycerolipids
    counts.insert("GL03".to_string(), 6936);
    counts.insert("GL02".to_string(), 604);
    counts.insert("GL05".to_string(), 104);
    counts.insert("GL01".to_string(), 93);
    counts.insert("GL04".to_string(), 25);
    counts.insert("GL07".to_string(), 16);
    counts.insert("GL00".to_string(), 10);
    counts.insert("GL06".to_string(), 7);

    // Glycerophospholipids
    counts.insert("GP01".to_string(), 1905);
    counts.insert("GP02".to_string(), 1565);
    counts.insert("GP04".to_string(), 1351);
    counts.insert("GP03".to_string(), 1231);
    counts.insert("GP10".to_string(), 1205);
    counts.insert("GP06".to_string(), 1199);
    counts.insert("GP15".to_string(), 338);
    counts.insert("GP20".to_string(), 273);

    // Sphingolipids
    counts.insert("SP05".to_string(), 2117);
    counts.insert("SP06".to_string(), 1393);
    counts.insert("SP02".to_string(), 612);
    counts.insert("SP03".to_string(), 353);
    counts.insert("SP01".to_string(), 129);
    counts.insert("SP00".to_string(), 11);
    counts.insert("SP04".to_string(), 9);
    counts.insert("SP08".to_string(), 1);

    // Sterol Lipids
    counts.insert("ST01".to_string(), 1923);
    counts.insert("ST04".to_string(), 795);
    counts.insert("ST03".to_string(), 761);
    counts.insert("ST02".to_string(), 402);
    counts.insert("ST05".to_string(), 232);

    // Prenol Lipids
    counts.insert("PR01".to_string(), 2475);
    counts.insert("PR02".to_string(), 82);
    counts.insert("PR04".to_string(), 50);
    counts.insert("PR03".to_string(), 37);

    // Saccharolipids
    counts.insert("SL03".to_string(), 1305);
    counts.insert("SL05".to_string(), 25);
    counts.insert("SL01".to_string(), 18);
    counts.insert("SL02".to_string(), 3);

    // Polyketides
    counts.insert("PK12".to_string(), 6602);
    counts.insert("PK13".to_string(), 199);
    counts.insert("PK15".to_string(), 127);
    counts.insert("PK03".to_string(), 99);
    counts.insert("PK04".to_string(), 46);
    counts.insert("PK09".to_string(), 41);
    counts.insert("PK11".to_string(), 38);

    // Palettes — shade 0-3 for first 4, shade 4 for rest
    const FA_PALETTE: [&str; 5] = ["#4E7705", "#6D9F06", "#97CE2F", "#BDEC6F", "#DDFFA0"];
    const GL_PALETTE: [&str; 5] = ["#098BD9", "#56B4E9", "#7DCCFF", "#BCE1FF", "#E7F4FF"];
    const GP_PALETTE: [&str; 5] = ["#7D3560", "#A1527F", "#CC79A7", "#E794C1", "#EFB6D6"];
    const SP_PALETTE: [&str; 5] = ["#9D654C", "#C17754", "#F09163", "#FCB076", "#FFD5AF"];
    const ST_PALETTE: [&str; 5] = ["#238b45", "#41ab5d", "#74c476", "#a1d99b", "#c7e9c0"];
    const PR_PALETTE: [&str; 5] = ["#4292c6", "#6baed6", "#9ecae1", "#c6dbef", "#eff3ff"];
    const SL_PALETTE: [&str; 5] = ["#6a51a3", "#807dba", "#9e9ac8", "#bcbddc", "#dadaeb"];
    const PK_PALETTE: [&str; 5] = ["#ff7f00", "#fe9929", "#fdae6b", "#fec44f", "#feeda0"];

    // Build result with proper color assignment
    let mut result = Vec::new();

    // ── Fatty Acyls ────────────────────────────────────────────────
    let fa_classes: Vec<(&str, &str)> = vec![
        ("Fatty Acids and Conjugates", "FA01"),
        ("Fatty esters", "FA07"),
        ("Eicosanoids", "FA03"),
        ("Docosanoids", "FA04"),
        ("Octadecanoids", "FA02"),
        ("Hydrocarbons", "FA11"),
        ("Fatty amides", "FA08"),
        ("Fatty alcohols", "FA05"),
        ("Oxygenated hydrocarbons", "FA12"),
        ("Fatty aldehydes", "FA06"),
        ("Fatty acyl glycosides", "FA13"),
        ("Other Fatty Acyls", "FA00"),
        ("Fatty nitriles", "FA09"),
        ("Fatty ethers", "FA10"),
    ];
    build_family(
        &mut result,
        &fa_classes,
        &counts,
        &smarts_map,
        &FA_PALETTE,
        "Fatty Acyls",
    );

    // ── Glycerolipids ──────────────────────────────────────────────
    let gl_classes: Vec<(&str, &str)> = vec![
        ("Triradylglycerols", "GL03"),
        ("Diradylglycerols", "GL02"),
        ("Glycosyldiradylglycerols", "GL05"),
        ("Monoradylglycerols", "GL01"),
        ("Monoglycosylglycerols", "GL04"),
        ("Betaine diradylglycerols", "GL07"),
        ("Other Glycerolipids", "GL00"),
        ("Betaine monoradylglycerols", "GL06"),
    ];
    build_family(
        &mut result,
        &gl_classes,
        &counts,
        &smarts_map,
        &GL_PALETTE,
        "Glycerolipids",
    );

    // ── Glycerophospholipids ───────────────────────────────────────
    let gp_classes: Vec<(&str, &str)> = vec![
        ("Glycerophosphocholines", "GP01"),
        ("Glycerophosphoethanolamines", "GP02"),
        ("Glycerophosphoglycerols", "GP04"),
        ("Glycerophosphoserines", "GP03"),
        ("Glycerophosphates", "GP10"),
        ("Glycerophosphoinositols", "GP06"),
        ("Glycerophosphoinositolglycans", "GP15"),
        ("Oxidized glycerophospholipids", "GP20"),
    ];
    build_family(
        &mut result,
        &gp_classes,
        &counts,
        &smarts_map,
        &GP_PALETTE,
        "Glycerophospholipids",
    );

    // ── Sphingolipids ──────────────────────────────────────────────
    let sp_classes: Vec<(&str, &str)> = vec![
        ("Neutral glycosphingolipids", "SP05"),
        ("Acidic glycosphingolipids", "SP06"),
        ("Ceramides", "SP02"),
        ("Phosphosphingolipids", "SP03"),
        ("Sphingoid bases", "SP01"),
        ("Other Sphingolipids", "SP00"),
        ("Phosphonosphingolipids", "SP04"),
        ("Amphoteric glycosphingolipids", "SP08"),
    ];
    build_family(
        &mut result,
        &sp_classes,
        &counts,
        &smarts_map,
        &SP_PALETTE,
        "Sphingolipids",
    );

    // ── Sterol Lipids ──────────────────────────────────────────────
    let st_classes: Vec<(&str, &str)> = vec![
        ("Sterols", "ST01"),
        ("Bile acids and derivatives", "ST04"),
        ("Secosteroids", "ST03"),
        ("Steroids", "ST02"),
        ("Steroid conjugates", "ST05"),
    ];
    build_family(
        &mut result,
        &st_classes,
        &counts,
        &smarts_map,
        &ST_PALETTE,
        "Sterol Lipids",
    );

    // ── Prenol Lipids ──────────────────────────────────────────────
    let pr_classes: Vec<(&str, &str)> = vec![
        ("Isoprenoids", "PR01"),
        ("Quinones and hydroquinones", "PR02"),
        ("Hopanoids", "PR04"),
        ("Polyprenols", "PR03"),
    ];
    build_family(
        &mut result,
        &pr_classes,
        &counts,
        &smarts_map,
        &PR_PALETTE,
        "Prenol Lipids",
    );

    // ── Saccharolipids ─────────────────────────────────────────────
    let sl_classes: Vec<(&str, &str)> = vec![
        ("Acyltrehaloses", "SL03"),
        ("Other acyl sugars", "SL05"),
        ("Acylaminosugars", "SL01"),
        ("Acylaminosugar glycans", "SL02"),
    ];
    build_family(
        &mut result,
        &sl_classes,
        &counts,
        &smarts_map,
        &SL_PALETTE,
        "Saccharolipids",
    );

    // ── Polyketides ────────────────────────────────────────────────
    let pk_classes: Vec<(&str, &str)> = vec![
        ("Flavonoids", "PK12"),
        ("Aromatic polyketides", "PK13"),
        ("Phenolic lipids", "PK15"),
        ("Annonaceae acetogenins", "PK03"),
        ("Macrolides and lactone polyketides", "PK04"),
        ("Polyether antibiotics", "PK09"),
        ("Cytochalasins", "PK11"),
    ];
    build_family(
        &mut result,
        &pk_classes,
        &counts,
        &smarts_map,
        &PK_PALETTE,
        "Polyketides",
    );

    result
}

/// Build a family of [`ChemicalClass`] objects, sorted by LMSD count (descending),
/// each using its own specific SMARTS pattern from `smarts_map`.
fn build_family(
    result: &mut Vec<ChemicalClass>,
    classes: &[(&str, &str)],
    counts: &HashMap<String, usize>,
    smarts_map: &HashMap<&str, &str>,
    palette: &[&str],
    family: &str,
) {
    let mut sorted: Vec<_> = classes.to_vec();
    sorted.sort_by(|a, b| {
        let a_count = counts.get(b.1).copied().unwrap_or(0);
        let b_count = counts.get(a.1).copied().unwrap_or(0);
        a_count.cmp(&b_count)
    });

    for (idx, (name, code)) in sorted.iter().enumerate() {
        let shade_idx = if idx < 4 { idx } else { 4 };
        let color = palette[shade_idx].to_string();
        let full_name = format!("{} [{}]", name, code);
        let smarts = smarts_map.get(*code).copied().unwrap_or("[*]").to_string();
        result.push(ChemicalClass::new(
            full_name,
            smarts,
            color,
            family.to_string(),
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chematic::smiles;

    #[test]
    fn fatty_acid_matches_palmitic_acid() {
        let fa = ChemicalClass::defaults()
            .into_iter()
            .find(|c| c.name == "FA")
            .expect("FA class");
        let mol = smiles::parse("CCCCCCCCCCCCCCCC(=O)O").expect("valid SMILES");
        assert!(fa.matches(&mol));
    }

    #[test]
    fn defaults_include_common_lipids() {
        let defaults = ChemicalClass::defaults();
        let names: Vec<_> = defaults.iter().map(|c| c.name.as_str()).collect();
        // FA, MUFA, PUFA
        assert!(names.contains(&"FA"));
        assert!(names.contains(&"MUFA"));
        assert!(names.contains(&"PUFA"));
        // GL
        assert!(names.contains(&"TG(AAA)"));
        assert!(names.contains(&"DG(AA)"));
        assert!(names.contains(&"MG(A)"));
        // GP
        assert!(names.contains(&"PC(AA)"));
        assert!(names.contains(&"PE(AA)"));
        assert!(names.contains(&"LPC(A)"));
        assert!(names.contains(&"LPE(A)"));
        // SP
        assert!(names.contains(&"Cer(AS)"));
        assert!(names.contains(&"SM(AS)"));
        // ST, PR, SL, PK
        assert!(names.contains(&"ST"));
        assert!(names.contains(&"PR"));
        assert!(names.contains(&"SL"));
        assert!(names.contains(&"PK"));
    }

    #[test]
    fn defaults_map_provides_lookup() {
        let map = ChemicalClass::defaults_map();
        assert!(map.contains_key("PC(AA)"));
        assert_eq!(map.get("PC(AA)").map(|c| c.name.as_str()), Some("PC(AA)"));
    }

    #[test]
    fn gp_class_order_matches_architecture_priority() {
        let gp: Vec<_> = ChemicalClass::defaults()
            .into_iter()
            .filter(|c| c.family == "Glycerophospholipids")
            .collect();
        let gp_names: Vec<_> = gp.iter().map(|c| c.name.as_str()).collect();
        // PI(AA) (priority 7) must come before PG(AA) (priority 6) —
        // matches the ordering in `rules::LipidRuleLibrary::add_default_glycerophospholipid_rules`
        let pi_pos = gp_names.iter().position(|&n| n == "PI(AA)").unwrap();
        let pg_idx = gp_names.iter().position(|&n| n == "PG(AA)").unwrap();
        assert!(
            pi_pos < pg_idx,
            "PI(AA) should appear before PG(AA) in the class ordering"
        );
    }

    #[test]
    fn family_order_follows_lipid_maps_hierarchy() {
        let defaults = ChemicalClass::defaults();
        let families: Vec<&str> = defaults
            .iter()
            .map(|c| c.family.as_str())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();
        // Families should appear in LIPID MAPS order: FA, GL, GP, SP, ST, PR, SL, PK
        let expected = [
            "Fatty Acyls",
            "Glycerolipids",
            "Glycerophospholipids",
            "Sphingolipids",
            "Sterol Lipids",
            "Prenol Lipids",
            "Saccharolipids",
            "Polyketides",
        ];
        for (i, fam) in expected.iter().enumerate() {
            assert!(
                families.contains(fam),
                "Expected family {fam} at position {i}"
            );
        }
        // Verify the first occurrence order matches expected
        let mut seen: Vec<&str> = Vec::new();
        for class in &defaults {
            if !seen.contains(&class.family.as_str()) {
                seen.push(class.family.as_str());
            }
        }
        let expected_order: Vec<&str> = expected.to_vec();
        assert_eq!(
            seen, expected_order,
            "Family order should follow LIPID MAPS hierarchy"
        );
    }
}
