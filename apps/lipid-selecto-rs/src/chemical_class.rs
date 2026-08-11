//! User-defined chemical classes with SMARTS pattern matching.

use chematic::smarts;
use std::collections::HashMap;

/// A chemical class defined by name, SMARTS pattern, display color, and family.
#[derive(Clone, Debug)]
pub struct ChemicalClass {
    pub name: String,
    pub smarts: String,
    pub color: String,
    pub family: String,
}

impl ChemicalClass {
    /// Create a new chemical class.
    pub fn new(
        name: impl Into<String>,
        smarts: impl Into<String>,
        color: impl Into<String>,
        family: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            smarts: smarts.into(),
            color: color.into(),
            family: family.into(),
        }
    }

    /// Check if a molecule matches this class's SMARTS pattern.
    ///
    /// Returns `true` if the molecule contains at least one match, `false` otherwise
    /// or if the SMARTS pattern cannot be parsed.
    pub fn matches(&self, molecule: &chematic::core::Molecule) -> bool {
        let Ok(query) = smarts::parse_smarts(&self.smarts) else {
            return false;
        };
        !smarts::find_matches(&query, molecule).is_empty()
    }

    /// Return the default lipid classes.
    ///
    /// These match the LIPID MAPS classification system with proper family and
    /// architecture designations. Uses CVD-friendly microshades color palettes (1-5).
    pub fn defaults() -> Vec<ChemicalClass> {
        vec![
            // === FATTY ACYLS (FA) - cvd_orange palette ===
            // FA: saturated, no double bonds
            ChemicalClass::new(
                "FA",
                "[#6][#6][#6][#6][#6][#6][#6][#6][CX3](=[OX1])[OH]",
                "#9D654C", // cvd_orange[1]
                "Fatty Acyls",
            ),
            // MUFA: exactly one C=C double bond
            ChemicalClass::new(
                "MUFA",
                "[#6][#6]=[#6][#6][#6][#6][#6][#6][#6][CX3](=[OX1])[OH]",
                "#C17754", // cvd_orange[2]
                "Fatty Acyls",
            ),
            // PUFA: two or more C=C double bonds (separated by single bond)
            ChemicalClass::new(
                "PUFA",
                "[#6][#6]=[#6][#6][#6]=[#6][#6][#6][#6][CX3](=[OX1])[OH]",
                "#F09163", // cvd_orange[3]
                "Fatty Acyls",
            ),
            // === GLYCEROLIPIDS (GL) - cvd_blue palette ===
            ChemicalClass::new(
                "TG(AAA)",
                "[CX4]([OX2][CX3](=[OX1])[#6])([OX2][CX3](=[OX1])[#6])[OX2][CX3](=[OX1])[#6]",
                "#098BD9", // cvd_blue[1]
                "Glycerolipids",
            ),
            ChemicalClass::new(
                "DG(AA)",
                "[CX4]([OX2][CX3](=[OX1])[#6])[OX2][CX3](=[OX1])[#6]",
                "#56B4E9", // cvd_blue[2]
                "Glycerolipids",
            ),
            ChemicalClass::new(
                "MG(A)",
                "[CH2X4][CHX4][CH2X4][OX2][CX3](=[OX1])[#6]",
                "#7DCCFF", // cvd_blue[3]
                "Glycerolipids",
            ),
            // === GLYCEROPHOSPHOLIPIDS (GP) - cvd_green and cvd_turquoise palettes ===
            ChemicalClass::new(
                "PC(AA)",
                "[PX4](=[OX1])([OX2])([OX2])[NX4+]([CH3])([CH3])[CH3]",
                "#4E7705", // cvd_green[1]
                "Glycerophospholipids",
            ),
            ChemicalClass::new(
                "PE(AA)",
                "[PX4](=[OX1])([OX2])([OX2])[CH2X4][CH2X4][NX3;H2,H1,H0]",
                "#6D9F06", // cvd_green[2]
                "Glycerophospholipids",
            ),
            ChemicalClass::new(
                "PS(AA)",
                "[PX4](=[OX1])([OX2])([OX2])[CH2X4][CHX4]([CX3](=[OX1])[OX2H,OX1-])[NX3]",
                "#97CE2F", // cvd_green[3]
                "Glycerophospholipids",
            ),
            ChemicalClass::new(
                "PG(AA)",
                "[PX4](=[OX1])([OX2])([OX2])[CH2X4][CHX4]([OX2H,OX1-])[CH2X4][OX2H,OX1-]",
                "#BDEC6F", // cvd_green[4]
                "Glycerophospholipids",
            ),
            ChemicalClass::new(
                "PI(AA)",
                "[PX4](=[OX1])([OX2])([OX2])[C;R1]1[CH;R1][CH;R1][CH;R1][CH;R1][CH;R1]1",
                "#DDFFA0", // cvd_green[5]
                "Glycerophospholipids",
            ),
            ChemicalClass::new(
                "PA(AA)",
                "[PX4](=[OX1])([OX2])([OX2])[CH2X4][CHX4][CH2X4][OX2H,OX1-]",
                "#DDFFA0", // cvd_green[5]
                "Glycerophospholipids",
            ),
            // LPC: single acyl + phosphocholine head
            ChemicalClass::new(
                "LPC(A)",
                "[CH2X4][CHX4]([OX2][CX3](=[OX1])[#6])[CH2X4][OX2][PX4](=[OX1])[OX2][CH2X4][N+;X4]",
                "#148F77", // cvd_turquoise[1]
                "Glycerophospholipids",
            ),
            // LPE: single acyl + phosphoethanolamine head
            ChemicalClass::new(
                "LPE(A)",
                "[CH2X4][CHX4]([OX2][CX3](=[OX1])[#6])[CH2X4][OX2][PX4](=[OX1])[OX2][CH2X4][NX3]",
                "#009E73", // cvd_turquoise[2]
                "Glycerophospholipids",
            ),
            ChemicalClass::new(
                "CL(AAAA)",
                "[PX4](=[OX1])([OX2])([OX2])[CH2X4][CHX4]([OX2])[CH2X4][OX2]",
                "#43BA8F", // cvd_turquoise[3]
                "Glycerophospholipids",
            ),
            // === SPHINGOLIPIDS (SP) - cvd_purple palette ===
            ChemicalClass::new(
                "Cer(AS)",
                "[NX3][CX3](=[OX1])[CX4]",
                "#7D3560", // cvd_purple[1]
                "Sphingolipids",
            ),
            ChemicalClass::new(
                "SM(AS)",
                "[NX4+][CX4][CX4][OX2][PX4](=[OX1])[OX2]",
                "#A1527F", // cvd_purple[2]
                "Sphingolipids",
            ),
            ChemicalClass::new(
                "HexCer(AS)",
                "[NX3][CX3](=[OX1])[CX4][CH1X4][CH1X4][OX2][CH1X4][CH1X4]",
                "#CC79A7", // cvd_purple[3]
                "Sphingolipids",
            ),
            // === STEROL LIPIDS (ST) - purple palette ===
            ChemicalClass::new(
                "ST",
                "[#6]1[#6][#6][#6]2[#6]([#6]1)[#6][#6][#6]2([#6])[#6]",
                "#6a51a3", // purple[1]
                "Sterol Lipids",
            ),
            // === PRENOL LIPIDS (PR) - orange palette ===
            ChemicalClass::new(
                "PR",
                "[#6]=[#6][#6]=[#6][#6]",
                "#ff7f00", // orange[1]
                "Prenol Lipids",
            ),
            // === SACCHARIPOLIPIDS (SL) - blue palette ===
            // Lipid A has multiple ester-linked acyls and phosphate
            ChemicalClass::new(
                "SL",
                "[#6][OX2][PX4](=[OX1])[OX2][#6]",
                "#4292c6", // blue[3]
                "Saccharipolipids",
            ),
            // === POLYKETIDES (PK) - green palette ===
            // Macrolide ring: large cyclic polyketide
            ChemicalClass::new(
                "PK",
                "[#6;R]1[#6]([#6](=[OX1])[#6])[#6;R][#6;R][#6;R][#6;R][#6;R][#6;R][#6;R][#6;R][#6;R][#6;R][#6;R][#6;R]1",
                "#238b45", // green[1]
                "Polyketides",
            ),
        ]
    }

    /// Convert defaults into a map for quick lookup by name.
    pub fn defaults_map() -> HashMap<String, ChemicalClass> {
        Self::defaults()
            .into_iter()
            .map(|c| (c.name.clone(), c))
            .collect()
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
}
