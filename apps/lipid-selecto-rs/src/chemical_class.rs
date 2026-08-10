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
    /// These are the standard lipid categories with SMARTS patterns that detect
    /// the key structural features. All patterns exclude rings ([!R]) to ensure
    /// we're only matching acyclic lipids.
    pub fn defaults() -> Vec<ChemicalClass> {
        vec![
            ChemicalClass::new(
                "PC",
                "[!R][PX4](=[OX1])[!R]",  // Phosphate + acyclic context
                "#7c3aed",  // Purple
            ),
            ChemicalClass::new(
                "PE",
                "[!R][NX3][!R][PX4](=[OX1])[!R]",  // Ethanolamine + phosphate + acyclic
                "#7c3aed",  // Purple
            ),
            ChemicalClass::new(
                "TG",
                "[CX4]([OX2][CX3](=[OX1])[#6])([OX2][CX3](=[OX1])[#6])[OX2][CX3](=[OX1])",  // Triglyceride
                "#0d9488",  // Teal
            ),
            ChemicalClass::new(
                "DG",
                "[CX4]([OX2][CX3](=[OX1])[#6])[OX2][CX3](=[OX1])",  // Diglyceride
                "#0d9488",  // Teal
            ),
            ChemicalClass::new(
                "PA",
                "[!R][PX4](=[OX1])[!R]",  // Phosphate + acyclic
                "#7c3aed",  // Purple
            ),
            ChemicalClass::new(
                "LPC",
                "[CX4][OX2][CX3](=[OX1])[#6]",  // Monoglyceride
                "#7c3aed",  // Purple
            ),
            ChemicalClass::new(
                "LPE",
                "[CX4][NX3][#6]",  // Amino + acyclic
                "#7c3aed",  // Purple
            ),
            ChemicalClass::new(
                "Ceramide",
                "[CX3](=[OX1])[NX3]",  // Amide linkage
                "#be185d",  // Pink
            ),
            ChemicalClass::new(
                "Fatty Acid",
                "[#6;!a;!R][CX3](=[OX1])[OH]",  // Carboxylic acid + acyclic
                "#2563eb",  // Blue
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
