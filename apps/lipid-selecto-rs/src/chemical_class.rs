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
