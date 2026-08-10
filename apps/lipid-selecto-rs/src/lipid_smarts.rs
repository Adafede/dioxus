//! Modular SMARTS fragment library for lipid classification.
//!
//! This module defines reusable SMARTS fragments that can be composed
//! to build robust lipid classifiers. Rather than hardcoding full patterns,
//! we define the chemical building blocks and combine them intelligently.

use std::collections::HashMap;

/// Core SMARTS fragments for lipid chemistry.
/// These are reusable building blocks that represent actual chemical structures.
#[derive(Debug)]
pub struct SmartsCores;

impl SmartsCores {
    /// Return a map of core structural motifs.
    pub fn cores() -> HashMap<&'static str, &'static str> {
        let mut map = HashMap::new();

        // === FUNCTIONAL GROUPS ===
        map.insert("acyl", "[CX3](=[OX1])[#6]");              // C(=O)-R
        map.insert("ester", "[OX2][CX3](=[OX1])[#6]");        // O-C(=O)-R
        map.insert("amide", "[NX3][CX3](=[OX1])[#6]");        // N-C(=O)-R

        // === GLYCEROL-BASED BACKBONES ===
        map.insert("glycerol_3C", "[CH2X4][CHX4][CH2X4]");    // 3-carbon glycerol core

        // === PHOSPHATE ===
        map.insert("phospho", "[P;X4](=[OX1])");              // P(=O) phosphate core

        // === HEADGROUPS ===
        map.insert(
            "choline",
            "[CH2X4][CH2X4][N+;X4]([CH3])([CH3])[CH3]",
        );
        map.insert("ethanolamine", "[CH2X4][CH2X4][NX3;H2,H1,H0]");
        map.insert("serine_head", "[CH2X4][CHX4]([CX3](=O)[OX2H,OX1-])[NX3]");
        map.insert("glycerol_head", "[CH2X4][CHX4]([OX2H,OX1-])[CH2X4][OX2H,OX1-]");

        // === RING SYSTEMS (CYCLIZATION) ===
        map.insert("inositol", "[C;R1]1[CH;R1][CH;R1][CH;R1][CH;R1][CH;R1]1");

        // === SPHINGOID BASES ===
        // Long-chain amino alcohol with unsaturation
        map.insert(
            "sphingoid_base",
            "[CH2X4][CHX4]([OX2H,OX1-])[CHX4]([NX3])[CHX4]=[CHX4][#6]",
        );
        map.insert(
            "dihydro_sphingoid",
            "[CH2X4][CHX4]([OX2H,OX1-])[CHX4]([NX3])[CH2X4][CHX4][#6]",
        );

        // === ETHER/PLASMALOGEN LINKAGES ===
        // Ether: C-O-C (no carbonyl)
        map.insert("ether", "[#6][OX2][#6]");
        // Plasmalogen: C-O-C=C (1Z-alkenyl ether, vinylether)
        map.insert("plasmalogen_ether", "[#6][OX2][CHX3]=[CHX3][#6]");

        map
    }
}

/// Structural family level in the lipid hierarchy.
/// Corresponds to LIPID MAPS top-level categories.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum StructuralFamily {
    /// Fatty Acyls (FA)
    FattyAcyl,
    /// Glycerolipids (GL)
    Glycerolipid,
    /// Glycerophospholipids (GP)
    Glycerophospholipid,
    /// Sphingolipids (SP)
    Sphingolipid,
    /// Sterol Lipids (ST)
    SterolLipid,
    /// Prenol Lipids (PR)
    PrenolLipid,
}

/// Lipid class level: the main categorical distinction.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum LipidClass {
    // === FATTY ACYLS ===
    FA,    // Fatty Acid
    PUFA,  // Polyunsaturated Fatty Acid (2+ C=C)
    MUFA,  // Monounsaturated Fatty Acid (1 C=C)

    // === GLYCEROLIPIDS ===
    MG, // Monoacylglycerol
    DG, // Diacylglycerol
    TG, // Triacylglycerol

    // === GLYCEROPHOSPHOLIPIDS ===
    PC,  // Phosphatidylcholine
    PE,  // Phosphatidylethanolamine
    PS,  // Phosphatidylserine
    PI,  // Phosphatidylinositol
    PG,  // Phosphatidylglycerol
    PA,  // Phosphatidic Acid
    CL,  // Cardiolipin
    LPC, // Lysophosphatidylcholine
    LPE, // Lysophosphatidylethanolamine

    // === SPHINGOLIPIDS ===
    Cer,     // Ceramide
    SM,      // Sphingomyelin
    HexCer,  // Hexosylceramide
}

/// Lipid subclass: the molecular architecture level.
/// Distinguishes diacyl, alkyl-acyl, plasmalogen, etc.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum Architecture {
    /// Two fatty acyl groups (standard)
    DiAcyl,
    /// One ether (alkyl) + one acyl
    AlkylAcyl,
    /// One acyl + one ether
    AcylAlkyl,
    /// Plasmalogen: 1Z-alkenyl ether + acyl
    Plasmalogen,
    /// Both ether-linked
    DiEther,
    /// Single acyl (lyso compounds)
    MonoAcyl,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cores_include_essential_motifs() {
        let cores = SmartsCores::cores();
        assert!(cores.contains_key("acyl"));
        assert!(cores.contains_key("ester"));
        assert!(cores.contains_key("phospho"));
        assert!(cores.contains_key("choline"));
        assert!(cores.contains_key("glycerol_3C"));
    }

    #[test]
    fn structural_families_are_distinct() {
        assert_ne!(StructuralFamily::FattyAcyl, StructuralFamily::Glycerolipid);
        assert_ne!(
            StructuralFamily::Glycerophospholipid,
            StructuralFamily::Sphingolipid
        );
    }

    #[test]
    fn lipid_classes_include_major_types() {
        let _classes = vec![
            LipidClass::FA,
            LipidClass::PC,
            LipidClass::PE,
            LipidClass::TG,
            LipidClass::Cer,
        ];
        assert_eq!(_classes.len(), 5);
    }
}
