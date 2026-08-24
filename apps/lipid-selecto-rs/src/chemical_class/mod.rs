//! User-defined chemical classes with SMARTS pattern matching.
//!
//! This module provides two complementary classification systems:
//!
//! - **Broad classes** ([`ChemicalClass::defaults`]): coarse-grained family-level
//!   SMARTS patterns (e.g. "FA", "PC(AA)") used for color attribution and
//!   simple lipid detection.
//! - **LMSD subclasses** ([`lmsd_all`]): the 58 LIPID MAPS Structure Database
//!   subclasses (FA01–FA13, GL01–GL07, …) with specific SMARTS patterns, used
//!   for precise class assignment in exports and gallery matching.
//!
//! Both systems share the [`ChemicalClass`] type and the same pre-compiled
//! SMARTS matching engine.

mod defaults;
mod lmsd;

use std::collections::HashMap;

use chematic::smarts;

use defaults::{
    fatty_acyls, glycerolipids, glycerophospholipids, polyketides, prenol_lipids, saccharolipids,
    sphingolipids, sterol_lipids,
};
pub use lmsd::lmsd_all;

// === Palette constants ===

// Microshades palettes — shade 0-3 for first 4 classes per family,
// shade 4 for the rest.
pub(super) const FA_PALETTE: [&str; 5] = ["#4E7705", "#6D9F06", "#97CE2F", "#BDEC6F", "#DDFFA0"];
pub(super) const GL_PALETTE: [&str; 5] = ["#098BD9", "#56B4E9", "#7DCCFF", "#BCE1FF", "#E7F4FF"];
pub(super) const GP_PALETTE: [&str; 5] = ["#7D3560", "#A1527F", "#CC79A7", "#E794C1", "#EFB6D6"];
pub(super) const SP_PALETTE: [&str; 5] = ["#9D654C", "#C17754", "#F09163", "#FCB076", "#FFD5AF"];
pub(super) const ST_PALETTE: [&str; 5] = ["#238b45", "#41ab5d", "#74c476", "#a1d99b", "#c7e9c0"];
pub(super) const PR_PALETTE: [&str; 5] = ["#4292c6", "#6baed6", "#9ecae1", "#c6dbef", "#eff3ff"];
pub(super) const SL_PALETTE: [&str; 5] = ["#6a51a3", "#807dba", "#9e9ac8", "#bcbddc", "#dadaeb"];
pub(super) const PK_PALETTE: [&str; 5] = ["#ff7f00", "#fe9929", "#fdae6b", "#fec44f", "#feeda0"];

/// A chemical class defined by name, SMARTS pattern, display color, and family.
///
/// SMARTS patterns are pre-compiled once at construction time to avoid
/// re-parsing the pattern string on every `matches` call — critical for large
/// datasets where thousands of molecules are matched against dozens of classes.
#[derive(Clone, Debug)]
pub struct ChemicalClass {
    /// Display name of the lipid class (e.g. "FA", "Cer(AS)").
    pub name: String,
    /// SMARTS pattern string (as defined in the constructor).
    pub smarts: String,
    /// Hex color code for UI rendering.
    pub color: String,
    /// LIPID MAPS broad family name (e.g. "Fatty Acyls", "Sphingolipids").
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
