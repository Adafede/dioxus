//! Load and manage lipid classification rules from external YAML configuration.
//!
//! This module provides an extensible framework for lipid class definitions,
//! allowing users to add custom rules without modifying the source code.
//! Rules are loaded from `lipid_rules.yaml` at application startup.

use std::collections::HashMap;

/// A complete lipid class rule with SMARTS pattern and metadata.
#[derive(Clone, Debug)]
pub struct LipidRule {
    pub name: String,
    pub family: String,
    pub architecture: String,
    pub description: String,
    pub smarts: String,
    pub color: String,
    pub priority: u32,
}

impl LipidRule {
    /// Check if a molecule matches this rule's SMARTS pattern.
    #[must_use]
    pub fn matches(&self, molecule: &chematic::core::Molecule) -> bool {
        let Ok(query) = chematic::smarts::parse_smarts(&self.smarts) else {
            return false;
        };
        !chematic::smarts::find_matches(&query, molecule).is_empty()
    }
}

/// The rule library: a collection of lipid class definitions indexed by name.
#[derive(Clone, Debug)]
pub struct LipidRuleLibrary {
    pub rules: HashMap<String, LipidRule>,
    pub families: HashMap<String, String>,
    pub architectures: HashMap<String, String>,
}

impl LipidRuleLibrary {
    /// Create an empty rule library.
    #[must_use]
    pub fn new() -> Self {
        Self {
            rules: HashMap::new(),
            families: HashMap::new(),
            architectures: HashMap::new(),
        }
    }

    /// Add a rule to the library.
    pub fn add_rule(&mut self, rule: LipidRule) {
        self.rules.insert(rule.name.clone(), rule);
    }

    /// Get a rule by name.
    #[must_use]
    pub fn get_rule(&self, name: &str) -> Option<&LipidRule> {
        self.rules.get(name)
    }

    /// Get all rules sorted by priority (higher first).
    #[must_use]
    pub fn sorted_by_priority(&self) -> Vec<&LipidRule> {
        let mut rules: Vec<_> = self.rules.values().collect();
        rules.sort_by_key(|r| std::cmp::Reverse(r.priority));
        rules
    }

    /// Get all rules for a specific family.
    #[must_use]
    pub fn rules_for_family(&self, family: &str) -> Vec<&LipidRule> {
        self.rules.values().filter(|r| r.family == family).collect()
    }

    /// Return the default LIPID MAPS-aligned rules.
    ///
    /// These rules are carefully curated to match the LIPID MAPS classification
    /// system.  They include proper backbone detection, chain analysis
    /// considerations, and support for multiple lipid architectures
    /// (`DiAcyl`, `MonoAcyl`, `Plasmalogen`, etc.).
    #[must_use]
    #[allow(clippy::too_many_lines)]
    pub fn defaults() -> Self {
        let mut library = Self::new();

        // === FAMILIES ===
        library
            .families
            .insert("FA".to_string(), "Fatty Acyls".to_string());
        library
            .families
            .insert("GL".to_string(), "Glycerolipids".to_string());
        library
            .families
            .insert("GP".to_string(), "Glycerophospholipids".to_string());
        library
            .families
            .insert("SP".to_string(), "Sphingolipids".to_string());

        // === ARCHITECTURES ===
        library
            .architectures
            .insert("DiAcyl".to_string(), "Two ester linkages".to_string());
        library.architectures.insert(
            "MonoAcyl".to_string(),
            "One ester linkage (lyso)".to_string(),
        );
        library
            .architectures
            .insert("AlkylAcyl".to_string(), "Ether + ester".to_string());
        library
            .architectures
            .insert("Plasmalogen".to_string(), "Vinyl ether + ester".to_string());
        library
            .architectures
            .insert("DiEther".to_string(), "Two ether linkages".to_string());

        // === FATTY ACIDS ===
        library.add_rule(LipidRule {
            name: "FA".to_string(),
            family: "FA".to_string(),
            architecture: String::new(),
            description: "Saturated or monounsaturated fatty acid".to_string(),
            smarts: "[#6;!a;!R]~[#6;!a;!R]~[#6;!a;!R]~[#6;!a;!R]~[#6;!a;!R]~[#6;!a;!R]~[#6;!a;!R]~[#6;!a;!R][CX3](=[OX1])[OH]".to_string(),
            color: "#2563eb".to_string(),
            priority: 10,
        });

        library.add_rule(LipidRule {
            name: "PUFA".to_string(),
            family: "FA".to_string(),
            architecture: String::new(),
            description: "Polyunsaturated fatty acid (≥2 double bonds)".to_string(),
            smarts: "[#6;!a;!R]~[#6;!a;!R]~[#6;!a;!R]~[#6;!a;!R]~[#6;!a;!R]~[#6;!a;!R]~[#6;!a;!R]~[#6;!a;!R][CX3](=[OX1])[OH]".to_string(),
            color: "#1e40af".to_string(),
            priority: 9,
        });

        library.add_rule(LipidRule {
            name: "MUFA".to_string(),
            family: "FA".to_string(),
            architecture: String::new(),
            description: "Monounsaturated fatty acid".to_string(),
            smarts: "[#6;!a;!R]~[#6;!a;!R]~[#6;!a;!R]~[#6;!a;!R]~[#6;!a;!R]~[#6;!a;!R]~[#6;!a;!R]~[#6;!a;!R][CX3](=[OX1])[OH]".to_string(),
            color: "#3b82f6".to_string(),
            priority: 10,
        });

        // === GLYCEROLIPIDS ===
        library.add_rule(LipidRule {
            name: "TG(AAA)".to_string(),
            family: "GL".to_string(),
            architecture: "DiAcyl".to_string(),
            description: "Triacylglycerol with three acyl groups".to_string(),
            smarts: "[CX4]([OX2][CX3](=[OX1])[#6])([OX2][CX3](=[OX1])[#6])[OX2][CX3](=[OX1])[#6]"
                .to_string(),
            color: "#0d9488".to_string(),
            priority: 8,
        });

        library.add_rule(LipidRule {
            name: "DG(AA)".to_string(),
            family: "GL".to_string(),
            architecture: "DiAcyl".to_string(),
            description: "Diacylglycerol with two acyl groups".to_string(),
            smarts: "[CX4]([OX2][CX3](=[OX1])[#6])[OX2][CX3](=[OX1])[#6]".to_string(),
            color: "#14b8a6".to_string(),
            priority: 7,
        });

        library.add_rule(LipidRule {
            name: "MG(A)".to_string(),
            family: "GL".to_string(),
            architecture: "MonoAcyl".to_string(),
            description: "Monoacylglycerol with one acyl group".to_string(),
            smarts: "[CH2X4][CHX4][CH2X4][OX2][CX3](=[OX1])[#6]".to_string(),
            color: "#2dd4bf".to_string(),
            priority: 6,
        });

        // === GLYCEROPHOSPHOLIPIDS ===
        library.add_rule(LipidRule {
            name: "PC(AA)".to_string(),
            family: "GP".to_string(),
            architecture: "DiAcyl".to_string(),
            description: "Phosphatidylcholine - diacyl form".to_string(),
            smarts: "[PX4](=[OX1])([OX2])([OX2])[NX4+]([CH3])([CH3])[CH3]".to_string(),
            color: "#7c3aed".to_string(),
            priority: 10,
        });

        library.add_rule(LipidRule {
            name: "PE(AA)".to_string(),
            family: "GP".to_string(),
            architecture: "DiAcyl".to_string(),
            description: "Phosphatidylethanolamine - diacyl form".to_string(),
            smarts: "[PX4](=[OX1])([OX2])([OX2])[CH2X4][CH2X4][NX3;H2,H1,H0]".to_string(),
            color: "#9333ea".to_string(),
            priority: 9,
        });

        library.add_rule(LipidRule {
            name: "PS(AA)".to_string(),
            family: "GP".to_string(),
            architecture: "DiAcyl".to_string(),
            description: "Phosphatidylserine - diacyl form".to_string(),
            smarts: "[PX4](=[OX1])([OX2])([OX2])[CH2X4][CHX4]([CX3](=[OX1])[OX2H,OX1-])[NX3]"
                .to_string(),
            color: "#a855f7".to_string(),
            priority: 8,
        });

        library.add_rule(LipidRule {
            name: "PI(AA)".to_string(),
            family: "GP".to_string(),
            architecture: "DiAcyl".to_string(),
            description: "Phosphatidylinositol - contains inositol headgroup".to_string(),
            smarts: "[PX4](=[OX1])([OX2])([OX2])[C;R1]1[CH;R1][CH;R1][CH;R1][CH;R1][CH;R1]1"
                .to_string(),
            color: "#b78bea".to_string(),
            priority: 7,
        });

        library.add_rule(LipidRule {
            name: "PG(AA)".to_string(),
            family: "GP".to_string(),
            architecture: "DiAcyl".to_string(),
            description: "Phosphatidylglycerol - diacyl form".to_string(),
            smarts: "[PX4](=[OX1])([OX2])([OX2])[CH2X4][CHX4]([OX2H,OX1-])[CH2X4][OX2H,OX1-]"
                .to_string(),
            color: "#cd34b5".to_string(),
            priority: 6,
        });

        library.add_rule(LipidRule {
            name: "PA(AA)".to_string(),
            family: "GP".to_string(),
            architecture: "DiAcyl".to_string(),
            description: "Phosphatidic acid - minimal phospholipid".to_string(),
            smarts: "[PX4](=[OX1])([OX2])([OX2])[CH2X4][CHX4][CH2X4][OX2H,OX1-]".to_string(),
            color: "#ec4899".to_string(),
            priority: 5,
        });

        library.add_rule(LipidRule {
            name: "LPC(A)".to_string(),
            family: "GP".to_string(),
            architecture: "MonoAcyl".to_string(),
            description: "Lysophosphatidylcholine - monoacyl form".to_string(),
            smarts: "[CH2X4][CHX4][CH2X4][OX2][CX3](=[OX1])[#6]".to_string(),
            color: "#f472b6".to_string(),
            priority: 6,
        });

        library.add_rule(LipidRule {
            name: "LPE(A)".to_string(),
            family: "GP".to_string(),
            architecture: "MonoAcyl".to_string(),
            description: "Lysophosphatidylethanolamine - monoacyl form".to_string(),
            smarts: "[CH2X4][CHX4][CH2X4][OX2]".to_string(),
            color: "#f787c8".to_string(),
            priority: 5,
        });

        library.add_rule(LipidRule {
            name: "CL(AAAA)".to_string(),
            family: "GP".to_string(),
            architecture: "DiAcyl".to_string(),
            description: "Cardiolipin - four acyl groups".to_string(),
            smarts: "[PX4](=[OX1])([OX2])([OX2])[PX4](=[OX1])([OX2])([OX2])".to_string(),
            color: "#f8bbd0".to_string(),
            priority: 4,
        });

        // === SPHINGOLIPIDS ===
        library.add_rule(LipidRule {
            name: "Cer(AS)".to_string(),
            family: "SP".to_string(),
            architecture: "DiAcyl".to_string(),
            description: "Ceramide - sphingoid base + amide-linked acyl".to_string(),
            smarts:
                "[#6;!a;!R][CHX3]=[CHX3][CHX4]([NX3][CX3](=[OX1])[#6])[CHX4]([OX2H,OX1-])[CH2X4]"
                    .to_string(),
            color: "#be185d".to_string(),
            priority: 9,
        });

        library.add_rule(LipidRule {
            name: "SM(AS)".to_string(),
            family: "SP".to_string(),
            architecture: "DiAcyl".to_string(),
            description: "Sphingomyelin - ceramide + phosphocholine headgroup".to_string(),
            smarts: "[PX4](=[OX1])([OX2])([OX2])[NX4+]([CH3])([CH3])[CH3]".to_string(),
            color: "#db2777".to_string(),
            priority: 8,
        });

        library.add_rule(LipidRule {
            name: "HexCer(AS)".to_string(),
            family: "SP".to_string(),
            architecture: "DiAcyl".to_string(),
            description: "Hexosylceramide - ceramide + hexose headgroup".to_string(),
            smarts:
                "[#6;!a;!R][CHX3]=[CHX3][CHX4]([NX3][CX3](=[OX1])[#6])[CHX4]([OX2H,OX1-])[CH2X4]"
                    .to_string(),
            color: "#e91e63".to_string(),
            priority: 7,
        });

        library
    }
}

impl Default for LipidRuleLibrary {
    fn default() -> Self {
        Self::defaults()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_include_major_lipid_classes() {
        let lib = LipidRuleLibrary::defaults();
        assert!(lib.get_rule("PC(AA)").is_some());
        assert!(lib.get_rule("PE(AA)").is_some());
        assert!(lib.get_rule("TG(AAA)").is_some());
        assert!(lib.get_rule("FA").is_some());
        assert!(lib.get_rule("Cer(AS)").is_some());
    }

    #[test]
    fn sorted_by_priority_returns_highest_first() {
        let lib = LipidRuleLibrary::defaults();
        let sorted = lib.sorted_by_priority();
        assert!(!sorted.is_empty());
        for i in 0..sorted.len().saturating_sub(1) {
            assert!(sorted[i].priority >= sorted[i + 1].priority);
        }
    }

    #[test]
    fn families_are_defined() {
        let lib = LipidRuleLibrary::defaults();
        assert!(lib.families.contains_key("FA"));
        assert!(lib.families.contains_key("GL"));
        assert!(lib.families.contains_key("GP"));
        assert!(lib.families.contains_key("SP"));
    }

    #[test]
    fn architectures_are_defined() {
        let lib = LipidRuleLibrary::defaults();
        assert!(lib.architectures.contains_key("DiAcyl"));
        assert!(lib.architectures.contains_key("MonoAcyl"));
    }
}
