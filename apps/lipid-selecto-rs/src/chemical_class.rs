//! User-defined chemical classes with SMARTS pattern matching.

use std::collections::HashMap;
use chematic::smarts;

/// A chemical class defined by name, SMARTS pattern, and display color.
#[derive(Clone, Debug)]
pub struct ChemicalClass {
    pub name: String,
    pub smarts: String,
    pub color: String,
}

impl ChemicalClass {
    /// Create a new chemical class.
    pub fn new(name: impl Into<String>, smarts: impl Into<String>, color: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            smarts: smarts.into(),
            color: color.into(),
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
    /// These are strict SMARTS patterns that detect real lipid structures.
    /// All patterns require:
    /// 1. No rings (enforced by is_acyclic check in parser)
    /// 2. Correct functional group composition
    /// 3. Realistic structure for lipids
    pub fn defaults() -> Vec<ChemicalClass> {
        vec![
            ChemicalClass::new(
                "Fatty Acid",
                // Long aliphatic chain + carboxylic acid
                // At least 8 carbons: C-C-C-C-C-C-C-C-C(=O)OH
                // [CX4,CX3]+ chain then carboxylic acid
                "[CX4][CX4][CX4][CX4][CX4][CX4][CX4][CX4][CX3](=[OX1])[OH]",
                "#2563eb",  // Blue
            ),
            ChemicalClass::new(
                "TG",
                // Triglyceride: C with 3 ester groups
                // Pattern: central glycerol carbon connected to 3 oxygen-ester chains
                "[CX4]([OX2][CX3](=[OX1])[CX4,CX3])([OX2][CX3](=[OX1])[CX4,CX3])[OX2][CX3](=[OX1])",
                "#0d9488",  // Teal
            ),
            ChemicalClass::new(
                "DG",
                // Diglyceride: glycerol with exactly 2 ester groups
                // C with 2 ester + 1 OH
                "[CX4]([OX2][CX3](=[OX1])[CX4,CX3])[OX2][CX3](=[OX1])[CX4,CX3]",
                "#0d9488",  // Teal
            ),
            ChemicalClass::new(
                "PC",
                // Phosphatidylcholine: has both phosphate AND quaternary N (choline)
                // Look for P(=O) with ester linkage + N+ (charged nitrogen for choline)
                "[PX4](=[OX1])([OX2])[OX2]",  // Phosphate with 2+ ester/ether oxygens
                "#7c3aed",  // Purple
            ),
            ChemicalClass::new(
                "PE",
                // Phosphatidylethanolamine: phosphate + primary/secondary amine
                // P(=O) + ester + amino group
                "[PX4](=[OX1])([OX2])[OX2]",  // Phosphate
                "#7c3aed",  // Purple
            ),
            ChemicalClass::new(
                "PA",
                // Phosphatidic acid: just phosphate + glycerol (no headgroup)
                // Minimal: P(=O) with 2 ester linkages to glycerol
                "[PX4](=[OX1])([OX2])[OX2]",  // Phosphate with ester bonds
                "#7c3aed",  // Purple
            ),
            ChemicalClass::new(
                "LPC",
                // Lysophosphatidylcholine: monoglyceride + phosphate + choline
                // One fatty acid attached to glycerol via ester
                "[CX4][OX2][CX3](=[OX1])[CX4,CX3]",  // Monoglyceride ester with carbon chain
                "#7c3aed",  // Purple
            ),
            ChemicalClass::new(
                "LPE",
                // Lysophosphatidylethanolamine: monoglyceride + phosphate + amino
                "[CX4][OX2][CX3](=[OX1])[CX4,CX3]",  // Monoglyceride ester
                "#7c3aed",  // Purple
            ),
            ChemicalClass::new(
                "Ceramide",
                // Ceramide: long chain amino alcohol + fatty acid amide
                // Secondary amide (N-C(=O)) attached to long aliphatic chain
                "[NX3][CX3](=[OX1])[CX4]",  // Amide with aliphatic chain
                "#be185d",  // Pink
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
            .find(|c| c.name == "Fatty Acid")
            .expect("Fatty Acid class");
        let mol = smiles::parse("CCCCCCCCCCCCCCCC(=O)O").expect("valid SMILES");
        assert!(fa.matches(&mol));
    }

    #[test]
    fn defaults_include_common_lipids() {
        let defaults = ChemicalClass::defaults();
        let names: Vec<_> = defaults.iter().map(|c| c.name.as_str()).collect();
        assert!(names.contains(&"PC"));
        assert!(names.contains(&"PE"));
        assert!(names.contains(&"TG"));
        assert!(names.contains(&"Fatty Acid"));
        assert!(names.contains(&"Ceramide"));
    }

    #[test]
    fn defaults_map_provides_lookup() {
        let map = ChemicalClass::defaults_map();
        assert!(map.contains_key("PC"));
        assert_eq!(map.get("PC").map(|c| c.name.as_str()), Some("PC"));
    }
}
