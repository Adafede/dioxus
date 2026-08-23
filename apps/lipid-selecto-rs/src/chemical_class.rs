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

/// Generate all LMSD subclass classes, sorted by LMSD count within each family.
///
/// Colors are assigned using microshades palette:
/// - First 4 classes per family get shades 0-3
/// - All remaining classes get shade 4 (the lightest)
///
/// Names use the full LMSD format: "Description [CODE]" (e.g., "Fatty Acids and Conjugates [FA01]")
#[must_use]
pub fn lmsd_all() -> Vec<ChemicalClass> {
    use std::collections::HashMap;

    // Counts from LMSD.sdf.tsv
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
    counts.insert("GL01".to_string(), 93);
    counts.insert("GL05".to_string(), 104);
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

    // Palettes - shade 0-3 for first 4, shade 4 for rest
    const FA_PALETTE: [&str; 5] = ["#4E7705", "#6D9F06", "#97CE2F", "#BDEC6F", "#DDFFA0"];
    const GL_PALETTE: [&str; 5] = ["#098BD9", "#56B4E9", "#7DCCFF", "#BCE1FF", "#E7F4FF"];
    const GP_PALETTE: [&str; 5] = ["#7D3560", "#A1527F", "#CC79A7", "#E794C1", "#EFB6D6"];
    const SP_PALETTE: [&str; 5] = ["#9D654C", "#C17754", "#F09163", "#FCB076", "#FFD5AF"];
    const ST_PALETTE: [&str; 5] = ["#238b45", "#41ab5d", "#74c476", "#a1d99b", "#c7e9c0"];
    const PR_PALETTE: [&str; 5] = ["#4292c6", "#6baed6", "#9ecae1", "#c6dbef", "#eff3ff"];
    const SL_PALETTE: [&str; 5] = ["#6a51a3", "#807dba", "#9e9ac8", "#bcbddc", "#dadaeb"];
    const PK_PALETTE: [&str; 5] = ["#ff7f00", "#fe9929", "#fdae6b", "#fec44f", "#feeda0"];

    // Define all LMSD subclasses (name, code, count)
    let mut fa_classes: Vec<(String, String)> = Vec::new();
    fa_classes.push(("Fatty Acids and Conjugates".to_string(), "FA01".to_string()));
    fa_classes.push(("Fatty esters".to_string(), "FA07".to_string()));
    fa_classes.push(("Eicosanoids".to_string(), "FA03".to_string()));
    fa_classes.push(("Docosanoids".to_string(), "FA04".to_string()));
    fa_classes.push(("Octadecanoids".to_string(), "FA02".to_string()));
    fa_classes.push(("Hydrocarbons".to_string(), "FA11".to_string()));
    fa_classes.push(("Fatty amides".to_string(), "FA08".to_string()));
    fa_classes.push(("Fatty alcohols".to_string(), "FA05".to_string()));
    fa_classes.push(("Oxygenated hydrocarbons".to_string(), "FA12".to_string()));
    fa_classes.push(("Fatty aldehydes".to_string(), "FA06".to_string()));
    fa_classes.push(("Fatty acyl glycosides".to_string(), "FA13".to_string()));
    fa_classes.push(("Other Fatty Acyls".to_string(), "FA00".to_string()));
    fa_classes.push(("Fatty nitriles".to_string(), "FA09".to_string()));
    fa_classes.push(("Fatty ethers".to_string(), "FA10".to_string()));

    let mut gl_classes: Vec<(String, String)> = Vec::new();
    gl_classes.push(("Triradylglycerols".to_string(), "GL03".to_string()));
    gl_classes.push(("Diradylglycerols".to_string(), "GL02".to_string()));
    gl_classes.push(("Monoradylglycerols".to_string(), "GL01".to_string()));
    gl_classes.push(("Glycosyldiradylglycerols".to_string(), "GL05".to_string()));
    gl_classes.push(("Monoglycosylglycerols".to_string(), "GL04".to_string()));
    gl_classes.push(("Betaine diradylglycerols".to_string(), "GL07".to_string()));
    gl_classes.push(("Other Glycerolipids".to_string(), "GL00".to_string()));
    gl_classes.push(("Betaine monoradylglycerols".to_string(), "GL06".to_string()));

    let mut gp_classes: Vec<(String, String)> = Vec::new();
    gp_classes.push(("Glycerophosphocholines".to_string(), "GP01".to_string()));
    gp_classes.push((
        "Glycerophosphoethanolamines".to_string(),
        "GP02".to_string(),
    ));
    gp_classes.push(("Glycerophosphoglycerols".to_string(), "GP04".to_string()));
    gp_classes.push(("Glycerophosphoserines".to_string(), "GP03".to_string()));
    gp_classes.push(("Glycerophosphates".to_string(), "GP10".to_string()));
    gp_classes.push(("Glycerophosphoinositols".to_string(), "GP06".to_string()));
    gp_classes.push((
        "Glycerophosphoinositolglycans".to_string(),
        "GP15".to_string(),
    ));
    gp_classes.push((
        "Oxidized glycerophospholipids".to_string(),
        "GP20".to_string(),
    ));

    let mut sp_classes: Vec<(String, String)> = Vec::new();
    sp_classes.push(("Neutral glycosphingolipids".to_string(), "SP05".to_string()));
    sp_classes.push(("Acidic glycosphingolipids".to_string(), "SP06".to_string()));
    sp_classes.push(("Ceramides".to_string(), "SP02".to_string()));
    sp_classes.push(("Phosphosphingolipids".to_string(), "SP03".to_string()));
    sp_classes.push(("Sphingoid bases".to_string(), "SP01".to_string()));
    sp_classes.push(("Other Sphingolipids".to_string(), "SP00".to_string()));
    sp_classes.push(("Phosphonosphingolipids".to_string(), "SP04".to_string()));
    sp_classes.push((
        "Amphoteric glycosphingolipids".to_string(),
        "SP08".to_string(),
    ));

    let mut st_classes: Vec<(String, String)> = Vec::new();
    st_classes.push(("Sterols".to_string(), "ST01".to_string()));
    st_classes.push(("Bile acids and derivatives".to_string(), "ST04".to_string()));
    st_classes.push(("Secosteroids".to_string(), "ST03".to_string()));
    st_classes.push(("Steroids".to_string(), "ST02".to_string()));
    st_classes.push(("Steroid conjugates".to_string(), "ST05".to_string()));

    let mut pr_classes: Vec<(String, String)> = Vec::new();
    pr_classes.push(("Isoprenoids".to_string(), "PR01".to_string()));
    pr_classes.push(("Quinones and hydroquinones".to_string(), "PR02".to_string()));
    pr_classes.push(("Hopanoids".to_string(), "PR04".to_string()));
    pr_classes.push(("Polyprenols".to_string(), "PR03".to_string()));

    let mut sl_classes: Vec<(String, String)> = Vec::new();
    sl_classes.push(("Acyltrehaloses".to_string(), "SL03".to_string()));
    sl_classes.push(("Other acyl sugars".to_string(), "SL05".to_string()));
    sl_classes.push(("Acylaminosugars".to_string(), "SL01".to_string()));
    sl_classes.push(("Acylaminosugar glycans".to_string(), "SL02".to_string()));

    let mut pk_classes: Vec<(String, String)> = Vec::new();
    pk_classes.push(("Flavonoids".to_string(), "PK12".to_string()));
    pk_classes.push(("Aromatic polyketides".to_string(), "PK13".to_string()));
    pk_classes.push(("Phenolic lipids".to_string(), "PK15".to_string()));
    pk_classes.push(("Annonaceae acetogenins".to_string(), "PK03".to_string()));
    pk_classes.push((
        "Macrolides and lactone polyketides".to_string(),
        "PK04".to_string(),
    ));
    pk_classes.push(("Polyether antibiotics".to_string(), "PK09".to_string()));
    pk_classes.push(("Cytochalasins".to_string(), "PK11".to_string()));

    // Build result with proper color assignment
    let mut result = Vec::new();

    // Fatty Acyls - use fatty acid SMARTS pattern (carboxylic acid with acyl chain)
    let fa_smarts = "[#6][#6][#6][#6][#6][#6][#6][#6][CX3](=[OX1])[OH]";
    fa_classes.sort_by(|a, b| {
        let a_count = counts.get(&b.1).copied().unwrap_or(0);
        let b_count = counts.get(&a.1).copied().unwrap_or(0);
        a_count.cmp(&b_count)
    });
    for (idx, (name, code)) in fa_classes.drain(..).enumerate() {
        let shade_idx = if idx < 4 { idx } else { 4 };
        let color = FA_PALETTE[shade_idx].to_string();
        let full_name = format!("{} [{}]", name, code);
        result.push(ChemicalClass::new(
            full_name,
            fa_smarts.to_string(),
            color,
            "Fatty Acyls".to_string(),
        ));
    }

    // Glycerolipids - use TG (triacylglycerol) pattern as family proxy
    let gl_smarts = "[CX4]([OX2][CX3](=[OX1])[#6])([OX2][CX3](=[OX1])[#6])[OX2][CX3](=[OX1])[#6]";
    gl_classes.sort_by(|a, b| {
        let a_count = counts.get(&b.1).copied().unwrap_or(0);
        let b_count = counts.get(&a.1).copied().unwrap_or(0);
        a_count.cmp(&b_count)
    });
    for (idx, (name, code)) in gl_classes.drain(..).enumerate() {
        let shade_idx = if idx < 4 { idx } else { 4 };
        let color = GL_PALETTE[shade_idx].to_string();
        let full_name = format!("{} [{}]", name, code);
        result.push(ChemicalClass::new(
            full_name,
            gl_smarts.to_string(),
            color,
            "Glycerolipids".to_string(),
        ));
    }

    // Glycerophospholipids - use PC pattern as family proxy
    let gp_smarts = "[PX4](=[OX1])([OX2])([OX2])[NX4+]([CH3])([CH3])[CH3]";
    gp_classes.sort_by(|a, b| {
        let a_count = counts.get(&b.1).copied().unwrap_or(0);
        let b_count = counts.get(&a.1).copied().unwrap_or(0);
        a_count.cmp(&b_count)
    });
    for (idx, (name, code)) in gp_classes.drain(..).enumerate() {
        let shade_idx = if idx < 4 { idx } else { 4 };
        let color = GP_PALETTE[shade_idx].to_string();
        let full_name = format!("{} [{}]", name, code);
        result.push(ChemicalClass::new(
            full_name,
            gp_smarts.to_string(),
            color,
            "Glycerophospholipids".to_string(),
        ));
    }

    // Sphingolipids - use Cer (ceramide) pattern as family proxy
    let sp_smarts = "[NX3][CX3](=[OX1])[CX4]";
    sp_classes.sort_by(|a, b| {
        let a_count = counts.get(&b.1).copied().unwrap_or(0);
        let b_count = counts.get(&a.1).copied().unwrap_or(0);
        a_count.cmp(&b_count)
    });
    for (idx, (name, code)) in sp_classes.drain(..).enumerate() {
        let shade_idx = if idx < 4 { idx } else { 4 };
        let color = SP_PALETTE[shade_idx].to_string();
        let full_name = format!("{} [{}]", name, code);
        result.push(ChemicalClass::new(
            full_name,
            sp_smarts.to_string(),
            color,
            "Sphingolipids".to_string(),
        ));
    }

    // Sterol Lipids - use sterol pattern
    let st_smarts = "[#6]1[#6][#6][#6]2[#6]([#6]1)[#6][#6][#6]2([#6])[#6]";
    st_classes.sort_by(|a, b| {
        let a_count = counts.get(&b.1).copied().unwrap_or(0);
        let b_count = counts.get(&a.1).copied().unwrap_or(0);
        a_count.cmp(&b_count)
    });
    for (idx, (name, code)) in st_classes.drain(..).enumerate() {
        let shade_idx = if idx < 4 { idx } else { 4 };
        let color = ST_PALETTE[shade_idx].to_string();
        let full_name = format!("{} [{}]", name, code);
        result.push(ChemicalClass::new(
            full_name,
            st_smarts.to_string(),
            color,
            "Sterol Lipids".to_string(),
        ));
    }

    // Prenol Lipids - use isoprenoid pattern
    let pr_smarts = "[#6]=[#6][#6]=[#6][#6]";
    pr_classes.sort_by(|a, b| {
        let a_count = counts.get(&b.1).copied().unwrap_or(0);
        let b_count = counts.get(&a.1).copied().unwrap_or(0);
        a_count.cmp(&b_count)
    });
    for (idx, (name, code)) in pr_classes.drain(..).enumerate() {
        let shade_idx = if idx < 4 { idx } else { 4 };
        let color = PR_PALETTE[shade_idx].to_string();
        let full_name = format!("{} [{}]", name, code);
        result.push(ChemicalClass::new(
            full_name,
            pr_smarts.to_string(),
            color,
            "Prenol Lipids".to_string(),
        ));
    }

    // Saccharolipids - use saccharolipid pattern
    let sl_smarts = "[#6][OX2][PX4](=[OX1])[OX2][#6]";
    sl_classes.sort_by(|a, b| {
        let a_count = counts.get(&b.1).copied().unwrap_or(0);
        let b_count = counts.get(&a.1).copied().unwrap_or(0);
        a_count.cmp(&b_count)
    });
    for (idx, (name, code)) in sl_classes.drain(..).enumerate() {
        let shade_idx = if idx < 4 { idx } else { 4 };
        let color = SL_PALETTE[shade_idx].to_string();
        let full_name = format!("{} [{}]", name, code);
        result.push(ChemicalClass::new(
            full_name,
            sl_smarts.to_string(),
            color,
            "Saccharolipids".to_string(),
        ));
    }

    // Polyketides - use aromatic polyketide pattern
    let pk_smarts = "[#6;R]1[#6]([#6](=[OX1])[#6])[#6;R][#6;R][#6;R][#6;R][#6;R][#6;R][#6;R][#6;R][#6;R][#6;R][#6;R][#6;R]1";
    pk_classes.sort_by(|a, b| {
        let a_count = counts.get(&b.1).copied().unwrap_or(0);
        let b_count = counts.get(&a.1).copied().unwrap_or(0);
        a_count.cmp(&b_count)
    });
    for (idx, (name, code)) in pk_classes.drain(..).enumerate() {
        let shade_idx = if idx < 4 { idx } else { 4 };
        let color = PK_PALETTE[shade_idx].to_string();
        let full_name = format!("{} [{}]", name, code);
        result.push(ChemicalClass::new(
            full_name,
            pk_smarts.to_string(),
            color,
            "Polyketides".to_string(),
        ));
    }

    result
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
